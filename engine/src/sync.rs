use std::sync::{Arc, Barrier};
use std::thread;

use crate::parameters::*;

//#################################################################################################
//
//                                       struct Sync
//
//#################################################################################################

pub(crate) struct Sync {
    start: Barrier,
    end: Barrier,
}

// ================================ pub(crate) impl

impl Sync {
    pub(crate) fn start_threads(f: fn(Arc<Sync>)) -> Arc<Sync> {
        let sync = Arc::new(Sync {
            start: Barrier::new(1 + NUM_THREADS as usize),
            end:   Barrier::new(1 + NUM_THREADS as usize),
        });

        for i in 0..NUM_THREADS {
            let sync = sync.clone();
            thread::spawn(move || f(sync));
        }

        sync
    }

    #[inline(always)]
    pub(crate) fn wait_start(self: Arc<Self>) {
        self.start.wait();
    }

    #[inline(always)]
    pub(crate) fn wait_end(self: Arc<Self>) {
        self.end.wait();
    }
}