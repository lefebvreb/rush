use std::thread;

use chess::Move;

use crate::params;
use crate::search::Search;
use crate::shared;

// The loop a worker thread is running
fn worker_loop() {
    loop {
        let mut search = Search::default();
        shared::wait();
        search.search_position();
        shared::wait();
    }
}

// Start the threads
pub(crate) fn start_threads() {
    for _ in 0..params::NUM_SEARCH_THREADS {
        thread::spawn(worker_loop);
    }
}

// Launch the threads and get the result
pub(crate) fn launch_search() -> Option<Move> {
    shared::reset_infos();

    shared::wait();
    thread::sleep(params::SEARCH_DURATION);
    shared::stop_search();
    shared::wait();
    
    shared::best_move()
}