use std::time::Duration;
use std::{env, io, thread};
use std::io::Write;
use std::str::FromStr;

use chess::prelude::*;
use engine::Engine;

const USAGE: &str = r#"
A cli to the engine and chess library, providing an interactive interface for testing and playing the engine.

Usage:
  $ ./cli <fen>
    <fen> : the fen string to be parsed, put into quotes.

The default positon's fen string is: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".
"#;

const HELP: &str = r#"
Available commands:
  help            : prints this message.
  play <move>     : plays the given <move>, encoded in pure algebraic coordinate notation.
  think <seconds> : starts the engine for <seconds> seconds.
  auto <seconds>  : plays the engine against itself, with <seconds> seconds to think for each move.
  exit            : exits the cli.
"#;

fn autoplay(engine: &mut Engine, duration: Duration) {
    loop {
        // Clear the screen.
        print!("\x1B[2J\x1B[1;1H");

        { // Print the board.
            let board = engine.read_board();
            println!("{b}\n{b:#}", b=board);
        }

        // Think for the given duration.
        println!("Engine is thinking...");
        engine.start();
        thread::sleep(duration);
        engine.stop();

        { // Perform the move the engine think is best.
            let mut board = engine.write_board();
            board.do_move(engine.get_best_move().unwrap());
        }
    }
}

fn main() {
    // Initializes the chess library.
    chess::init();

    // Get the arguments.
    let mut args = env::args();

    // Executable path.
    args.next().unwrap();

    // Get and parse fen.
    let fen = match args.next() {
        Some(s) => s,
        _ => {
            println!("{}", USAGE);
            return;
        },
    };
    let board = Board::new(&fen).expect("Cannot parse fen");

    // Create the engine.
    let mut engine = Engine::new(board);

    // The read buffer.
    let mut buffer = String::new();

    loop {
        // Clear the screen.
        print!("\x1B[2J\x1B[1;1H");

        { // Print the board.
            let board = engine.read_board();
            println!("{b}\n{b:#}", b=board);
        }

        // Get infos about the engine's state.
        let best_move = engine.get_best_move().map(|mv| format!("{}", mv)).unwrap_or("-".to_string());
        let depth = engine.get_current_depth();

        // Print the prompt.
        print!("Engine's preferred move: {}.\nFurthest depth searched: {}\nType \"help\" for a list of all commands.\n>>> ", best_move, depth);

        // Read a line from the terminal.
        io::stdout().flush().ok();
        buffer.clear();
        io::stdin().read_line(&mut buffer).expect("Cannot read line");

        // Get the user input.
        let mut input = buffer.trim_end().split(' ');

        match input.next() {
            Some(command) => match command {
                "help" => println!("{}", HELP),
                "play" => {
                    let mut board = engine.write_board();
                    match input.next().map(|s| board.parse_move(s)) {
                        Some(Ok(mv)) => {
                            board.do_move(mv);
                            continue;
                        }
                        Some(Err(_)) => println!("Invalid move litteral, make sure that you used pure algebraic coordinate notation and that the move is legal in this position."),
                        None => println!("Invalid usage of the \"play\" command, type \"help\" to get the correct usage."),
                    }
                },
                "think" | "auto" => {
                    match input.next().map(|s| u64::from_str(s).map(|seconds| Duration::from_secs(seconds))) {
                        Some(Ok(duration)) => {
                            if command == "think" {
                                println!("Engine is thinking...");
                                engine.start();
                                thread::sleep(duration);
                                engine.stop();
                                continue;
                            } else {
                                autoplay(&mut engine, duration);
                                return;
                            }
                        },
                        Some(Err(_)) => println!("Could not parse thinking duration, it must be a positive integer."),
                        None => println!("Invalid usage of the \"{}\" command, type \"help\" to get the correct usage.", command),
                    }
                },
                "exit" => return,
                "" => continue,
                unknown => println!("Unknown command: \"{}\"", unknown),
            }
            _ => (),
        }

        println!("Press enter to continue...");
        io::stdin().read_line(&mut buffer).expect("Cannot read line");
    }
}