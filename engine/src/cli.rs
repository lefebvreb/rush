use std::time::Duration;
use std::{env, io, thread};
use std::io::Write;
use std::str::FromStr;

use chess::prelude::*;
use engine::Engine;

const USAGE: &str = r#"A cli to the engine and chess library, providing an interactive interface for testing and playing the engine.

Usage:
  $ ./cli <fen>
    <fen> : the fen string to be parsed, put into quotes.

The default position's fen string is: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1""#;

const HELP: &str = r#"Available commands:
  help            : prints this message.
  play <move>     : plays the given <move>, encoded in pure algebraic coordinate notation.
  think <seconds> : starts the engine for <seconds> seconds.
  auto <seconds>  : plays the engine against itself, with <seconds> seconds to think for each move.
  exit            : exits the cli.
"#;

// The global state of the cli.
struct State {
    engine: Engine,
    buffer: String,
    history: Vec<Move>,
}

// ================================ Utils

impl State {
    // Print the board and it's fen representation.
    fn print_board(&self) {
        // Clear the screen.
        print!("\x1B[2J\x1B[1;1H");
        
        let board = self.engine.read_board();
        let history_string = self.history.iter().map(|mv| format!("{}", mv)).collect::<Vec<_>>().join(", ");
        println!("fen string: \"{b}\"\nMove history: {h}\n{b:#}", b=board, h=history_string);
    }

    // Print what the engine think is best.
    fn print_engine(&self) {
        if let Some(mv) = self.engine.get_best_move() {
            println!("Engine's preferred move: {}.\nFurthest depth searched: {}.", mv, self.engine.get_current_depth());
        } else {
            println!("Engine hasn't had time to think yet.")
        }
    }

    // Read a line from the terminal.
    fn read_tokens(&mut self) -> Vec<String> {
        // The prompt.
        print!(">>> ");

        io::stdout().flush().ok();
        self.buffer.clear();
        io::stdin().read_line(&mut self.buffer).expect("Cannot read line");

        // Tokenize.
        self.buffer.split_ascii_whitespace().map(|s| s.to_string()).collect()
    }

    // Ask the user to press enter before continuing.
    fn ask_ok(&mut self) {
        println!("Press enter to continue...");
        io::stdin().read_line(&mut self.buffer).expect("Cannot read line");
    }

    // Makes the engine think for duration seconds.
    fn think_for(&mut self, duration: Duration) {
        self.engine.start();
        thread::sleep(duration);
        self.engine.stop();
    }

    // Plays the given move.
    fn play_move(&mut self, mv: Move) {
        let mut board = self.engine.write_board();

        assert!(board.is_pseudo_legal(mv) && board.is_legal(mv), "Tried to play illegal move");

        self.history.push(mv);

        board.do_move(mv);
    }
}

// ================================ Commands

impl State {
    fn play(&mut self, args: &mut impl Iterator<Item = String>) {
        match args.next().map(|s| self.engine.read_board().parse_move(&s)) {
            Some(Ok(mv)) => self.play_move(mv),
            Some(Err(msg)) => {
                println!("{}", msg);
                self.ask_ok();
            },
            None => {
                println!("Invalid usage of the \"play\" command, type \"help\" to get correct usage.");
                self.ask_ok();
            },
        }
    }

    fn back(&mut self) {
        if self.history.is_empty() {
            println!("No move to undo.");
            self.ask_ok();
            return;
        }

        let mv = self.history.pop().unwrap();
        let mut board = self.engine.write_board();
        board.undo_move(mv);
    }

    fn think(&mut self, args: &mut impl Iterator<Item = String>) {
        match args.next().map(|s| f64::from_str(&s).map(|n| Duration::from_secs_f64(n))) {
            Some(Ok(duration)) => {
                self.think_for(duration);
            },
            Some(Err(msg)) => {
                println!("Could not parse duration: {}.", msg);
                self.ask_ok();
            },
            None => {
                println!("Invalid usage of the \"think\" command, type \"help\" to get correct usage.");
                self.ask_ok();
            }
        }
    }

    fn auto(&mut self, args: &mut impl Iterator<Item = String>) {
        match args.next().map(|s| f64::from_str(&s).map(|n| Duration::from_secs_f64(n))) {
            Some(Ok(duration)) => {
                loop {
                    // Print the board.
                    self.print_board();

                    // Get the engine's preferred move.
                    self.think_for(duration);
                    let mv = self.engine.get_best_move().expect("Engine found nothing");
                    
                    // Play the move.
                    self.play_move(mv);
                }
            },
            Some(Err(msg)) => {
                println!("Could not parse duration: {}.", msg);
                self.ask_ok();
            },
            None => {
                println!("Invalid usage of the \"auto\" command, type \"help\" to get correct usage.");
                self.ask_ok();
            }
        }
    }
}

fn main() {
    // Initializes the chess library.
    chess::init();

    let mut state = {
        // Get the arguments.
        let mut args = env::args();

        // Executable path.
        args.next().expect("Can't get executable path.");

        // Get and parse fen.
        let board = match args.next().map(|fen| Board::new(&fen)) {
            Some(Ok(board)) => board,
            Some(Err(msg)) => {
                println!("{}", msg);
                return;
            },
            None => {
                println!("{}", USAGE);
                return;
            },
        };

        // Construct the state.
        State {
            engine: Engine::new(board),
            buffer: String::new(),
            history: Vec::new(),
        }
    };

    loop {
        // Print the state of the board and of the engine.
        state.print_board();
        state.print_engine();

        let mut args = state.read_tokens().into_iter();

        if let Some(command) = args.next() {
            match command.as_str() {
                "help" => {
                    println!("{}", HELP);
                    state.ask_ok();
                },
                "play" => state.play(&mut args),
                "back" => state.back(),
                "think" => state.think(&mut args),
                "auto" => state.auto(&mut args),
                "exit" => return,
                unknown => {
                    println!("Unknown command: \"{}\". Type \"help\" to get a list of available commands.", unknown);
                    state.ask_ok();
                },
            }
        }
    }
}