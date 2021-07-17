use anyhow::{Error, Result};
use chess::prelude::*;
use engine::Engine;
use warp::ws::Message;

// The fen used for the default position.
const DEFAULT_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

// Makes a warp::ws::Message from a serde_json::json! input.
macro_rules! msg {
    ($($json:tt)+) => {
        Message::text(serde_json::json!($($json)+).to_string())
    }
}

//#################################################################################################
//
//                                         struct Game
//
//#################################################################################################

// Manages the state of the game.
#[derive(Debug)]
pub struct Game {
    engine: Engine,
    history: Vec<Move>,
    cursor: usize,
}

// ================================ pub impl

impl Game {
    // Creates a new game with the default position.
    pub fn new() -> Self {
        Self {
            engine: Engine::new(Board::new(DEFAULT_FEN).unwrap()),
            history: Vec::new(),
            cursor: 0,
        }
    }

    // Returns a global message with all the relevant informations of a given state.
    pub fn on_all(&self) -> Message {
        if let Some(mv) = self.engine.get_best_move() {
            msg!({
                "fen": self.fen(),
                "history": self.history(),
                "draw": self.draw(),
                "thinking": self.engine.is_thinking(),
                "engineMove": mv.to_string(),
                "engineDepth": self.engine.get_current_depth(),
            })
        } else {
            msg!({
                "fen": self.fen(),
                "history": self.history(),
                "draw": self.draw(),
                "thinking": false,
                "engineMove": null,
                "engineDepth": 0,
            })
        }
    }

    // Stops the engine.
    pub fn on_stop(&self) -> Result<Message> {
        if self.engine.is_thinking() {
            self.engine.stop();
            Ok(msg!({
                "thinking": false,
            }))
        } else {
            Err(Error::msg("Engine isn't thinking."))
        }        
    }

    // Tries to parse and play the given move.
    pub fn on_play(&mut self, s: &str) -> Result<Message> {
        let mv = self.engine.read_board()
            .parse_move(s)
            .map_err(|e| Error::msg(format!("Failed to parse move: \"{}\"", s)))?;

        Ok(self.do_move(mv))
    }

    // Starts the engine for the given amount of seconds.
    pub fn on_think(&self) -> Result<Message> {
        if self.engine.is_thinking() {
            Err(Error::msg("Engine is already thinking."))
        } else {
            self.engine.start();
            Ok(msg!({
                "thinking": true,
            }))
        }
    }

    // Tries to get the engine's favorite move and play it.
    pub fn on_do(&mut self) -> Result<Message> {
        let mv = self.engine.get_best_move()
            .ok_or(Error::msg("Engine has no best move yet"))?;
        self.do_move(mv);

        Ok(self.do_move(mv))
    }

    // Undo the last move.
    pub fn on_undo(&mut self) -> Result<Message> {
        if self.cursor == 0 {
            return Err(Error::msg("There is nothing to undo."));
        }

        self.cursor -= 1;
        let mv = self.history[self.cursor];

        self.engine.write_board().undo_move(mv);

        Ok(msg!({
            "undoMove": self.fen(),
        }))
    }

    // Redo the last move.
    pub fn on_redo(&mut self) -> Result<Message> {
        if self.cursor == self.history.len() {
            return Err(Error::msg("There is nothing to redo."));
        }

        let mv = self.history[self.cursor];
        self.cursor += 1;

        self.engine.write_board().do_move(mv);

        Ok(self.do_move(mv))
    }
}

// ================================ impl

impl Game {
    // Performs the given move, assumed to be legal, and updates the state.
    fn do_move(&mut self, mv: Move) -> Message {
        if self.cursor != self.history.len() {
            self.history.truncate(self.cursor);
        }
        self.history.push(mv);
        self.cursor += 1;

        let mut board = self.engine.write_board();
        board.do_move(mv);

        msg!({
            "doMove": mv.to_string(),
        })
    }

    fn fen(&self) -> String {
        self.engine.read_board().to_string()
    }

    fn draw(&self) -> bool {
        matches!(self.engine.read_board().status(), Status::Draw)
    }

    fn history(&self) -> Vec<String> {
        self.history.iter().map(|mv| mv.to_string()).collect::<Vec<_>>()
    }
}