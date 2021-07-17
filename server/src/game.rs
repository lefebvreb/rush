use chess::prelude::*;
use engine::Engine;

// Manages the state of the game.
#[derive(Debug)]
pub struct Game {
    engine: Engine,
    history: Vec<Move>,
}

impl Game {
    // Creates a new game with the default position.
    pub fn new() -> Game {
        let board = Board::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();

        Game {
            engine: Engine::new(board),
            history: Vec::new(),
        }
    }
}