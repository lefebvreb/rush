use std::time::Duration;

use anyhow::{Error, Result};
use chess::prelude::*;
use engine::Engine;
use serde_json::{json, Value};
use tokio::sync::mpsc::{self, UnboundedSender};
use warp::ws::Message;

use crate::messages::{Command, Response};

// The fen used for the default position.
const DEFAULT_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

// Makes a warp::ws::Message from a serde_json::json! input.
macro_rules! msg {
    {$($json:tt)*} => {
        Message::text(json!({$($json)*}).to_string())
    }
}

//#################################################################################################
//
//                                       struct History
//
//#################################################################################################

// A struct keeping a history of played and/or undoed moves, as well
// as their textual representations.
#[derive(Debug)]
struct History {
    moves: Vec<Move>,
    strings: Vec<String>,
    cursor: usize,
}

// ================================ impl

impl History {
    // Creates a new empty move history.
    fn new() -> Self {
        Self {
            moves: Vec::new(),
            strings: Vec::new(),
            cursor: 0,
        }
    }

    // Pushes a new move to the history, losing all undoed moves.
    fn push(&mut self, mv: Move) {
        // If we are not at the end of the timeline.
        if self.cursor != self.moves.len() {
            // Turns out the move has already been done in the past future, redo it.
            if mv == self.moves[self.cursor] {
                self.redo().ok();
                return;
            }

            // Throw out all future moves, we are changing timeline.
            self.moves.truncate(self.cursor);
            self.strings.truncate(self.cursor);
        }

        // Push a new move.
        self.moves.push(mv);
        self.strings.push(mv.to_string());
        self.cursor += 1;
    }

    // Undo a move.
    fn undo(&mut self) -> Result<Move> {
        // Check there is something to undo.
        if self.cursor == 0 {
            return Err(Error::msg("There is no move to undo"));
        }

        // Decrement the cursor and return that move.
        self.cursor -= 1;
        Ok(self.moves[self.cursor])
    }

    // Redo a move.
    fn redo(&mut self) -> Result<Move> {
        // Check that we are not at the end of the timeline.
        if self.cursor == self.moves.len() {
            return Err(Error::msg("There is no move to redo"));
        }

        // Get the move to redo and increment the cursor.
        let mv = self.moves[self.cursor];
        self.cursor += 1;
        Ok(mv)
    }
}

// ================================ traits impl

impl From<&History> for Value {
    // Converts the history into it's json representation: an array of the 
    // moves currently played.
    fn from(history: &History) -> Self {
        Self::from(&history.strings[..history.cursor])
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
    history: History,
    tx: UnboundedSender<Command>,
}

// ================================ pub impl

impl Game {
    // Creates a new game with the default position.
    // Returns a channel used to pass messages to the game state.
    // Takes a channel in argument, used by the game state to respond
    // to incoming messages.
    pub fn new(tx: UnboundedSender<Result<Response>>) -> UnboundedSender<Command> {
        // Creates the communication channels used to send messages to the game state.
        let (game_tx, mut game_rx) = mpsc::unbounded_channel();
        let self_tx = game_tx.clone();

        // Spawn a new task, reacting to incoming client messages.
        tokio::spawn(async move {
            // The game state itself.
            let mut game = Self {
                engine: Engine::new(Board::new(DEFAULT_FEN).unwrap()),
                history: History::new(),
                tx: self_tx,
            };

            // While there are incoming messages, process them and respond
            // through the given tx channel.
            while let Some(command) = game_rx.recv().await {
                if let Err(e) = tx.send(game.react(command)) {
                    eprintln!("{}", e);
                    break;
                }
            }
        });

        game_tx
    }

    // Reacts to a given command and returns the response.
    pub fn react(&mut self, command: Command) -> Result<Response> {
        match command {
            // On welcoming a new connection, send him the welcome message.
            Command::Welcome(dest) => {
                Ok(Response::Send {
                    dest,
                    msg: msg!{
                        "fen": self.engine.read_board().to_string(),
                        "history": Value::from(&self.history),
                        "thinking": self.engine.is_thinking(),
                        "engineMove": self.engine.get_best_move().map_or(Value::Null, |mv| Value::from(mv.to_string())),
                        "engineDepth": self.engine.get_current_depth(),
                    },
                })
            }
            // Request to play a move.
            Command::Play(s) => {
                // Parses and performs the move.
                let mv = self.engine.read_board().parse_move(s.as_str()).map_err(|_| Error::msg("Unable to parse move."))?;
                self.engine.write_board().do_move(mv);
                self.history.push(mv);

                Ok(Response::Broadcast(msg!{
                    "doMove": mv.to_string(),
                    "thinking": false,
                    "engineMove": null,
                    "engineDepth": 0,
                }))
            },
            // Request to start the engine for a given amount of seconds.
            Command::Think(seconds) => {
                // Starts the engine.
                if self.engine.is_thinking() {
                    return Err(Error::msg("Engine is already thinking."));
                }
                self.engine.start();

                // Starts a task that will stop the engine later.
                let tx = self.tx.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(Duration::from_secs_f64(seconds)).await;
                    tx.send(Command::Stop).ok();
                });

                Ok(Response::Broadcast(msg!{
                    "thinking": true,
                }))
            },
            // Request to stop the engine.
            Command::Stop => {
                self.engine.stop();

                Ok(Response::Broadcast(msg!{
                    "thinking": false,
                    "engineMove": self.engine.get_best_move().ok_or(Error::msg("Engine has no best move."))?.to_string(),
                    "engineDepth": self.engine.get_current_depth(),
                }))
            },
            // Request to perform the engine's preferred move.
            Command::Do => {
                let mv = self.engine.get_best_move().ok_or(Error::msg("Engine has no preferred move."))?;
                self.engine.write_board().do_move(mv);
                self.history.push(mv);

                Ok(Response::Broadcast(msg!{
                    "doMove": mv.to_string(),
                    "thinking": false,
                    "engineMove": null,
                    "engineDepth": 0,
                }))
            },
            // Request to undo move.
            Command::Undo => {
                let mv = self.history.undo()?;
                self.engine.write_board().undo_move(mv);

                Ok(Response::Broadcast(msg!{
                    "undoMove": self.engine.read_board().to_string(),
                    "thinking": false,
                    "engineMove": null,
                    "engineDepth": 0,
                }))
            },
            // Request to redo the last undoed move.
            Command::Redo => {
                let mv = self.history.redo()?;
                self.engine.write_board().do_move(mv);

                Ok(Response::Broadcast(msg!{
                    "doMove": mv.to_string(),
                    "thinking": false,
                    "engineMove": null,
                    "engineDepth": 0,
                }))
            },
        }
    }
}