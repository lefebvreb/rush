use std::sync::{Arc, Barrier};
use std::thread;

use crate::parameters::*;

//#################################################################################################
//
//                                       struct Sync
//
//#################################################################################################

// A struct to facilitate synchronization between threads
pub(crate) struct Sync {
    barrier: Barrier,
}

// ================================ pub(crate) impl

impl Sync {
    // Start the threads and give them an access to the Sync object
    pub(crate) fn start_threads(f: fn(Arc<Sync>)) -> Arc<Sync> {
        let sync = Arc::new(Sync {
            barrier: Barrier::new(1 + NUM_SEARCH_THREADS as usize),
        });

        for i in 0..NUM_SEARCH_THREADS {
            let sync = sync.clone();
            thread::spawn(move || f(sync));
        }

        sync
    }

    // Wait for all the threads to reach the barrier
    #[inline(always)]
    pub(crate) fn wait(self: Arc<Self>) {
        self.barrier.wait();
    }
}