use std::fmt;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Arc, Barrier, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use chess::board::Board;
use chess::book::Book;
use chess::moves::{AtomicMove, Move};

use crate::{params, utils};
use crate::search::Search;
use crate::table::TranspositionTable;

//#################################################################################################
//
//                                       struct GlobalInfo
//
//#################################################################################################

/// The shared info between threads.
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
    /// Returns a reference to the TranspositionTable.
    #[inline]
    pub(crate) fn get_table(&self) -> &TranspositionTable {
        &self.table
    }

    /// Returns a clone of the current board, the root of the tree to explore.
    #[inline]
    pub(crate) fn board(&self) -> Board {
        self.board.read().unwrap().clone()
    }

    /// Returns true if the engine is currently searching.
    #[inline]
    pub(crate) fn is_searching(&self) -> bool {
        self.searching.load(Ordering::Relaxed)
    }

    /// Atomically looks for the stop signal.
    #[inline]
    pub(crate) fn should_stop(&self) -> bool {
        self.stop.load(Ordering::Acquire)
    }

    /// Wait at the barrier for every other thread.
    #[inline]
    pub(crate) fn wait(&self) {
        self.barrier.wait();
    }

    /// Returns the current search depth.
    #[inline]
    pub(crate) fn search_depth(&self) -> u8 {
        self.search_depth.load(Ordering::Relaxed)
    }

    /// Returns the search depth a thread should search to next.
    /// This is computed as 1 + the current base depth + the id,
    /// where the id is a number such that at any given time,
    /// one thread searches to log2(params::NUM_SEARCH_THREAD),
    /// two at log2(params::NUM_SEARCH_THREAD)-1, four at 
    /// log2(params::NUM_SEARCH_THREAD)-2, etc.
    /// This allow for a flexible work distribution, and makes threads
    /// not all search at the same thing at the same time.
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

    /// Report back a move, stores if it was searched at a deeper depth
    /// than the current one, and subsequently increase the base search depth.
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

// ================================ impl

impl GlobalInfo {
    /// Loads the best move found as of now.
    #[inline]
    fn get_best_move(&self) -> Option<Move> {
        self.best_move.load()
    }
}

//#################################################################################################
//
//                                       enum EngineResult
//
//#################################################################################################

/// Represents the result of an engine think() call.
#[derive(Debug)]
pub enum EngineStatus {
    /// When no call to think() was done yet.
    Idling,
    /// When the engine is currently thinking.
    Thinking,
    /// When a move was probed in a book.
    BookMove(Move),
    /// When the engine actually thought for an amount of time.
    Preferred {
        mv: Move,
        depth: u8,
    }
}

// ================================ pub impl

impl EngineStatus {
    /// Returns the move the engine has found, or None if it is currently thinking or has not thought yet.
    pub fn get_move(&self) -> Option<Move> {
        match *self {
            EngineStatus::BookMove(mv) | EngineStatus::Preferred {mv, ..} => Some(mv),
            _ => None,
        }
    }

    /// Returns true if the engine is thinking.
    pub fn is_thinking(&self) -> bool {
        matches!(self, EngineStatus::Thinking)
    }
}

// ================================ traits impl

impl fmt::Display for EngineStatus {
    /// Displays the result associated with this engine.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EngineStatus::Idling => write!(f, "Engine has has no time to think yet."),
            EngineStatus::Thinking => write!(f, "Engine is currently thinking."),
            EngineStatus::BookMove(mv) => write!(f, "Engine has found a book move {}.", mv),
            EngineStatus::Preferred {mv, depth} => write!(f, "Engine's preferred move is: {}.\nFurthest depth reached: {}.", mv, depth),
        }
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
    book: Option<Book>,
    status: EngineStatus,
    seed: u32,
}

// ================================ pub impl

impl Engine {
    /// Initializes a new chess engine, working on a board.
    pub fn new(board: Board, book: Option<Book>) -> Engine {
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

        // The seed used for all pseudo-random number generation.
        let mut seed = utils::seed();

        // Initializes the thread pool.
        let handles = (0..params::NUM_SEARCH_THREAD).map(|_| {
            let thread_seed = utils::xorshift32(&mut seed).wrapping_mul(0x98FF2E9E);
            let info = info.clone();

            thread::spawn(move || {
                let mut search = Search::new(thread_seed, info);
                search.thread_main();
            })
        }).collect();

        Engine {
            info,
            handles,
            book,
            status: EngineStatus::Idling,
            seed,
        }
    }

    /// Returns the current best move.
    pub fn poll(&self) -> &EngineStatus {
        &self.status
    }

    /// Returns a read lock to the board.
    pub fn read_board(&self) -> RwLockReadGuard<'_, Board> {
        self.info.board.read().unwrap()
    }

    /// Starts the engine and begins thinking for the next best move.
    /// May return false, meaning the engine is already thinking, or
    /// it has found a book move. In either case, the engine must be
    /// polled to get it's status.
    /// May return true, meaning the engine has started thinking and
    /// will need to be stopped and polled whenever we want some results.
    pub fn start(&mut self) -> bool {
        // If already searching, return.
        if self.info.is_searching() {
            return false;
        }

        // If a match is found in a book, return it.
        if let Some(mv) = self.lookup() {
            self.status = EngineStatus::BookMove(mv);
            return false;
        }

        // Set the engine as thinking.
        self.status = EngineStatus::Thinking;

        // Set the searching flag and wait at the barrier with 
        // the other threads that are already waiting.
        self.info.searching.store(true, Ordering::Release);
        self.info.wait();

        return true;
    }

    /// Stops the engine if it is searching.
    /// Search may be resumed by calling start() again.
    pub fn stop(&mut self) {
        if !self.info.is_searching() {
            return;
        }

        // Get more time if the engine has found nothing.
        while self.info.get_best_move().is_none() {
            thread::sleep(Duration::from_millis(50));
        }

        // Unset the searching flag and wait at the barrier for
        // the other threads to all stop working.
        self.info.searching.store(false, Ordering::Release);
        self.info.wait();

        self.status = EngineStatus::Preferred {
            mv: self.info.get_best_move().unwrap(),
            depth: self.info.search_depth(),
        };
    }

    /// Stops the search if it is on and resets the search informations.
    /// Then returns a write lock to the board.
    pub fn write_board(&mut self) -> RwLockWriteGuard<'_, Board> {
        // Stop if thinking.
        if self.info.is_searching() {
            self.info.searching.store(false, Ordering::Release);
            self.info.wait();
        }

        // Sets the engine as idling.
        self.status = EngineStatus::Idling;

        self.info.search_depth.store(0, Ordering::Release);
        self.info.search_id.store(0, Ordering::Release);
        self.info.best_move.reset();

        self.info.board.write().unwrap()
    }
}

// ================================ impl

impl Engine {
    /// Stops the search if it is on.
    /// Probes the book to see if any move may be applied in this situation.
    fn lookup(&mut self) -> Option<Move> {
        if let Some(book) = &self.book {
            let results = book.probe(&self.info.board.read().unwrap());
        
            match results.len() {
                0 => None,
                1 => {
                    let (mv, _) = results[0];
                    Some(mv)
                },
                _ => {
                    let total_weight: u32 = results.iter().map(|&(_, weight)| u32::from(weight)).sum();
                    let rand = utils::xorshift32(&mut self.seed) % total_weight;

                    let mut sum = 0;
                    for &(mv, weight) in results.iter() {
                        let next_sum = sum + u32::from(weight);

                        if (sum..next_sum).contains(&rand) {
                            return Some(mv);
                        }

                        sum = next_sum;
                    }

                    unreachable!()
                },
            }
        } else {
            None
        }        
    }
}

// ================================ traits impl

impl Drop for Engine {
    /// On dropping the engine, make sure that all threads are joined.
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