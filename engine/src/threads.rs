use std::sync::{Arc, Barrier};
use std::thread;

use chess::{Game, Move};

use crate::params;
use crate::search::Search;
use crate::shared;

// The loop a worker thread is running
fn worker_loop(sync: Arc<Barrier>) {
    let mut search = Search::default();

    loop {
        sync.wait();
        search.search_position();
        sync.wait();
        search = Search::default();
    }
}

// Start the threads
pub(crate) fn start_threads() -> Arc<Barrier> {
    let sync = Arc::new(Barrier::new(1 + params::NUM_SEARCH_THREADS as usize));

    for i in 0..params::NUM_SEARCH_THREADS {
        let sync = sync.clone();
        thread::spawn(move || worker_loop(sync));
    }

    sync
}

// Launch the threads and get the result
pub(crate) fn launch_search(game: &Game, sync: &Arc<Barrier>) -> Option<Move> {
    shared::reset_infos(game.clone());

    sync.wait();
    thread::sleep(params::SEARCH_DURATION);
    shared::stop_search();
    sync.wait();
    
    shared::best_move()
}