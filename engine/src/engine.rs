use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Arc, Barrier, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use chess::board::Board;
use chess::moves::{AtomicMove, Move};

use crate::params;
use crate::search::Search;
use crate::table::TranspositionTable;

//#################################################################################################
//
//                                       struct GlobalInfo
//
//#################################################################################################

// The shared info between threads.
#[derive(Debug)]
pub(crate) struct GlobalInfo {
    barrier: Barrier,
    searching: AtomicBool,
    stop: AtomicBool,
    
    table: TranspositionTable,
    search_depth: AtomicU8,
    search_id: AtomicU8,
    best_move: AtomicMove,

    board: RwLock<Board>,
}

// ================================ pub(crate) impl

impl GlobalInfo {
    // Returns a reference to the TranspositionTable.
    #[inline]
    pub(crate) fn get_table(&self) -> &TranspositionTable {
        &self.table
    }

    // Returns a clone of the current board, the root of the tree to explore.
    #[inline]
    pub(crate) fn board(&self) -> Board {
        self.board.read().unwrap().clone()
    }

    // Returns true if the engine is currently searching.
    #[inline]
    pub(crate) fn is_searching(&self) -> bool {
        self.searching.load(Ordering::Relaxed)
    }

    // Atomically looks for the stop signal.
    #[inline]
    pub(crate) fn should_stop(&self) -> bool {
        self.stop.load(Ordering::Acquire)
    }

    // Wait at the barrier for every other thread.
    #[inline]
    pub(crate) fn wait(&self) {
        self.barrier.wait();
    }

    // Returns the current search depth.
    #[inline]
    pub(crate) fn search_depth(&self) -> u8 {
        self.search_depth.load(Ordering::Relaxed)
    }

    // Returns the search depth a thread should search to next.
    // This is computed as 1 + the current base depth + the id,
    // where the id is a number such that at any given time,
    // one thread searches to log2(params::NUM_SEARCH_THREAD),
    // two at log2(params::NUM_SEARCH_THREAD)-1, four at 
    // log2(params::NUM_SEARCH_THREAD)-2, etc.
    // This allow for a flexible work distribution, and makes threads
    // not all search at the same thing at the same time.
    #[inline]
    pub(crate) fn thread_search_depth(&self) -> u8 {
        let depth = self.search_depth();

        let id = self.search_id.fetch_update(
            Ordering::SeqCst,
            Ordering::SeqCst,
            |id| Some((id + 1) % params::NUM_SEARCH_THREAD as u8)
        ).unwrap();

        1 + depth + (id + 1).trailing_zeros() as u8 
    }

    // Report back a move, stores if it was searched at a deeper depth
    // than the current one, and subsequently increase the base search depth.
    #[inline]
    pub(crate) fn report_move(&self, mv: Move, depth: u8) {
        self.search_depth.fetch_update(
            Ordering::SeqCst,
            Ordering::SeqCst,
            |cur_depth| {
                if depth <= cur_depth {
                    None
                } else {
                    self.best_move.store(mv);
                    Some(depth)
                }
            }
        ).ok();
    }
}

//#################################################################################################
//
//                                        struct Engine
//
//#################################################################################################

/// The struct representing a chess engine.
#[derive(Debug)]
pub struct Engine {
    info: Arc<GlobalInfo>,
    handles: Vec<JoinHandle<()>>,
}

// ================================ pub impl

impl Engine {
    /// Initializes a new chess engine, working on a board.
    pub fn new(board: Board) -> Engine {
        // Construct the initial info object.
        let info = Arc::new(GlobalInfo {
            barrier: Barrier::new(params::NUM_SEARCH_THREAD + 1),
            searching: AtomicBool::new(false),
            stop: AtomicBool::new(false),
            
            table: TranspositionTable::new(),
            search_depth: AtomicU8::new(0),
            search_id: AtomicU8::new(0),
            best_move: AtomicMove::default(),

            board: RwLock::new(board),
        });

        // Initializes the thread pool.
        let handles = (0..params::NUM_SEARCH_THREAD).map(|_| {
            let info = info.clone();

            thread::spawn(move || {
                let mut search = Search::new(info);
                search.thread_main();
            })
        }).collect();

        Engine {
            info,
            handles,
        }
    }

    /// Returns true if the engine is currently thinking.
    pub fn is_thinking(&self) -> bool {
        self.info.is_searching()
    }

    /// Starts the engine and begins thinking for the next best move.
    pub fn start(&self) {
        // If already searching, return.
        if self.info.is_searching() {
            return;
        }

        // Set the searching flag and wait at the barrier with 
        // the other threads that are already waiting.
        self.info.searching.store(true, Ordering::Release);
        self.info.wait();
    }

    /// Stops the engine if it is searching.
    /// Search may be resumed by calling start() again.
    pub fn stop(&self) {
        if !self.info.is_searching() {
            return;
        }

        // Get more time if the engine has found nothing.
        while self.get_best_move().is_none() {
            thread::sleep(Duration::from_millis(50));
        }

        // Unset the searching flag and wait at the barrier for
        // the other threads to all stop working.
        self.info.searching.store(false, Ordering::Release);
        self.info.wait();
    }

    /// Returns the current best move.
    pub fn get_best_move(&self) -> Option<Move> {
        self.info.best_move.load()
    }

    /// Returns the current best depth searched.
    pub fn get_current_depth(&self) -> u8 {
        self.info.search_depth()
    }

    /// Returns a read lock to the board.
    pub fn read_board(&self) -> RwLockReadGuard<'_, Board> {
        self.info.board.read().unwrap()
    }

    /// Stops the search if it is on and resets the search informations.
    /// Then returns a write lock to the board.
    pub fn write_board(&self) -> RwLockWriteGuard<'_, Board> {
        self.stop();

        self.info.search_depth.store(0, Ordering::Release);
        self.info.search_id.store(0, Ordering::Release);
        self.info.best_move.reset();

        self.info.board.write().unwrap()
    }
}

// ================================ traits impl

impl Drop for Engine {
    // On dropping the engine, make sure that all threads are joined.
    fn drop(&mut self) {
        if self.handles.is_empty() {
            return;
        }

        self.stop();

        self.info.stop.store(true, Ordering::Release);
        self.info.wait();

        for handle in self.handles.drain(..) {
            handle.join().ok();
        }
    }
}