#![allow(dead_code, unused_variables, unused_macros)]

use chess::{Game, Move, MoveGenerator, ParseFenError};

mod eval;
mod params;
mod search;
mod shared;
mod threads;

pub fn initialize(fen: &str) -> Result<(), ParseFenError> {
    let game = Game::from_fen(fen)?;

    shared::initialize(game);
    threads::start_threads();

    Ok(())
}

pub fn do_move(mv: Move) {
    shared::do_move(mv);
}

pub fn compute() -> Move {
    threads::launch_search()
        .unwrap_or_else(|| shared::game().legals().next().unwrap())
}