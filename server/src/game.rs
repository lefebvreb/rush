use std::time::Duration;

use anyhow::{Error, Result};
use chess::prelude::*;
use engine::Engine;
use warp::ws::Message;

use crate::protocol::ServerMessage;

// The fen used for the default position.
const DEFAULT_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

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

    msg: ServerMessage,
}

// ================================ pub impl

impl Game {
    // Creates a new game with the default position.
    pub fn new() -> Game {
        let board = Board::new(DEFAULT_FEN).unwrap();

        Game {
            engine: Engine::new(board),
            history: Vec::new(),
            cursor: 0,

            msg: ServerMessage {
                fen: DEFAULT_FEN.to_string(),
                draw: false,
                lastMove: String::new(),
                thinking: false,
                engineMove: String::new(),
                engineDepth: 0,
            }
        }
    }

    // Tries to parse and play the given move.
    pub fn on_play(&mut self, s: &str) -> Result<()> {
        let mv = self.engine.read_board()
            .parse_move(s)
            .map_err(|e| Error::msg(format!("failed to parse move: {}", e)))?;
        self.do_move(mv);

        Ok(())
    }

    // Starts the engine for the given amount of seconds.
    pub async fn on_think(&self, seconds: f32) {
        if self.engine.start() {
            tokio::time::sleep(Duration::from_secs_f32(seconds)).await;
            self.engine.stop();
        }
    }

    // Tries to get the engine's favorite move and play it.
    pub fn on_do(&mut self) -> Result<()> {
        let mv = self.engine.get_best_move()
            .ok_or(Error::msg("Engine has no best move yet"))?;
        self.do_move(mv);

        Ok(())
    }

    // Undo the last move.
    pub fn on_undo(&mut self) -> Result<()> {
        todo!()
    }

    // Redo the last move.
    pub fn on_redo(&mut self) -> Result<()> {
        todo!()
    }
}

// ================================ impl

impl Game {
    // Performs the given move, assumed to be legal, and updates the state.
    fn do_move(&mut self, mv: Move) {
        if self.cursor != self.history.len() {
            self.history.truncate(self.cursor);
        }
        self.history.push(mv);
        self.cursor += 1;

        let mut board = self.engine.write_board();
        board.do_move(mv);

        self.msg.fen = board.to_string();
        self.msg.draw = matches!(board.status(), Status::Draw);
        self.msg.lastMove = mv.to_string();
        self.msg.thinking = false;
        self.msg.engineMove.clear();
        self.msg.engineDepth = 0;
    }

    // Produces a server message from the current state of the game.
    fn make_msg(&self) -> Message {
        self.msg.to_warp_msg()
    }
}