use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Arc, Barrier, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::thread::{self, JoinHandle};

use chess::board::Board;
use chess::moves::{AtomicMove, Move};

use crate::params;
use crate::table::TranspositionTable;

//#################################################################################################
//
//                                       struct GlobalInfo
//
//#################################################################################################

// The shared info between threads.
#[derive(Debug)]
pub(crate) struct GlobalInfo {
    table: TranspositionTable,
    barrier: Barrier,
    stop: AtomicBool,

    searching: AtomicBool,
    search_depth: AtomicU8,
    search_id: AtomicU8,

    best_mv: AtomicMove,
    board: RwLock<Board>,
}

// ================================ pub(crate) impl

impl GlobalInfo {
    // Returns a reference to the TranspositionTable.
    #[inline]
    pub(crate) fn get_table(&self) -> &TranspositionTable {
        &self.table
    }

    // Returns true if the engine is currently searching.
    #[inline]
    pub(crate) fn is_searching(&self) -> bool {
        self.searching.load(Ordering::Acquire)
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
        self.search_depth.load(Ordering::Acquire)
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

        1 + depth + id.trailing_zeros() as u8 
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
                    self.best_mv.store(mv);
                    Some(depth)
                }
            }
        ).ok();
    }
}

//#################################################################################################
//
//                                      fn worker_thread_main
//
//#################################################################################################

// The function that all worker threads will run forever.
fn worker_thread_main(info: Arc<GlobalInfo>) {
    // TODO: Initialize here

    loop {
        info.wait();
    
        // The stop flag is set, return from this function, the thread will be joined.
        if info.should_stop() {
            return;
        }
    
        // TODO: Search here while !info.searching()
    
        info.wait();
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
        Engine {
            info: Arc::new(GlobalInfo {
                table: TranspositionTable::new(),
                barrier: Barrier::new(params::NUM_SEARCH_THREAD + 1),
                stop: AtomicBool::new(false),
    
                searching: AtomicBool::new(false),
                search_depth: AtomicU8::new(0),
                search_id: AtomicU8::new(0),
    
                best_mv: AtomicMove::new(),
                board: RwLock::new(board),
            }),
            handles: Vec::new(),
        }
    }

    /// Starts the engine and begins thinking for the next best move.
    pub fn start(&mut self) {
        // If already searching, return.
        if self.info.is_searching() {
            return;
        }

        // If not done already, spawn the threads.
        // This make thread spawning the threads a lazy operation.
        if self.handles.is_empty() {
            for _ in 0..params::NUM_SEARCH_THREAD {
                let info = self.info.clone();
    
                self.handles.push(thread::spawn(move || {
                    worker_thread_main(info);
                }));
            }
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

        // Unset the searching flag and wait at the barrier for
        // the other threads to all stop working.
        self.info.searching.store(false, Ordering::Release);
        self.info.wait();
    }

    /// Returns the current best move.
    pub fn get_best_move(&self) -> Option<Move> {
        self.info.best_mv.load()
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
        self.info.best_mv.reset();

        self.info.board.write().unwrap()
    }
}

// ================================ traits impl

impl Drop for Engine {
    // On dropping the engine, make sure that all threads are joined.
    fn drop(&mut self) {
        self.stop();

        self.info.stop.store(true, Ordering::Release);
        self.info.wait();

        for handle in self.handles.drain(..) {
            handle.join().unwrap();
        }
    }
}