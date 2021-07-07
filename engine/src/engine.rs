use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Arc, Barrier};
use std::thread;

use chess::prelude::*;

use crate::errors::EngineError;
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

    search_depth: AtomicU8,
    search_id: AtomicU8,
    stop: AtomicBool,

    // Todo: store best move and current board
}

// ================================ pub(crate) impl

impl GlobalInfo {
    // Creates a new GlonalInfo struct with default parameters.
    pub(crate) fn new() -> GlobalInfo {
        GlobalInfo {
            table: TranspositionTable::new(),
            barrier: Barrier::new(params::NUM_SEARCH_THREAD + 1),

            search_depth: AtomicU8::new(0),
            search_id: AtomicU8::new(0),
            stop: AtomicBool::new(true),
        }
    }

    // Atomically looks for the stop signal.
    #[inline]
    pub(crate) fn is_stop(self: &Arc<Self>) -> bool {
        self.stop.load(Ordering::Acquire)
    }
}

// ================================ impl

impl GlobalInfo {
    // Sets the new value of the stop flag atomically.
    #[inline]
    fn set_stop(self: &mut Arc<Self>, val: bool) {
        self.stop.store(val, Ordering::Release);
    }
}

//#################################################################################################
//
//                                      fn worker_thread_main
//
//#################################################################################################

// The function that all worker threads will run forever.
fn worker_thread_main(info: Arc<GlobalInfo>) -> ! {
    todo!()
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
}

// ================================ pub impl

impl Engine {
    pub fn new() -> Engine {
        let info = Arc::new(GlobalInfo::new());

        for i in 0..params::NUM_SEARCH_THREAD {
            let info = info.clone();

            thread::spawn(move || {
                worker_thread_main(info);
            });
        }

        Engine {
            info,
        }
    }

    pub fn start(&mut self) -> Result<(), EngineError> {
        if !self.info.is_stop() {
            return Err(EngineError::new("engine already started"));
        }
        self.info.set_stop(false);

        todo!()
    }

    pub fn stop(&mut self) -> Result<Move, EngineError> {
        if self.info.is_stop() {
            return Err(EngineError::new("engine not started"));
        }
        self.info.set_stop(true);

        todo!()
    }

    pub fn set_position(&self, board: &Board) {

    }
}