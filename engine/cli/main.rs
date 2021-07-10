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
  exit            : exits the cli.
  play  <move>    : plays the given <move>, encoded in pure algebraic coordinate notation.
  think <seconds> : starts the engine for <seconds> seconds.
"#;

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

        // Print the engine's choice.
        println!("Engine's preferred move: {}.", engine.get_best_move().map(|mv| format!("{}", mv)).unwrap_or("-".to_string()));

        // Print the prompt.
        print!("Type \"help\" for a list of all commands.\n>>> ");

        // Read a line from the terminal.
        io::stdout().flush().ok();
        buffer.clear();
        io::stdin().read_line(&mut buffer).expect("Cannot read line");

        // Get the user input.
        let mut input = buffer.trim_end().split(' ');

        match input.next() {
            Some(command) => match command {
                "help" => println!("{}", HELP),
                "exit" => return,
                "play" => {
                    let mut board = engine.write_board();
                    match input.next() {
                        Some(s) => {
                            match board.parse_move(s) {
                                Ok(mv) => {
                                    board.do_move(mv);
                                    continue;
                                },
                                _ => println!("Invalid move litteral, make sure that you used pure algebraic coordinate notation and that the move is legal in this position."),
                            }
                        },
                        _ => println!("Invalid usage of the \"play\" command, type \"help\" to get the correct usage."),
                    }
                },
                "think" => {
                    match input.next() {
                        Some(s) => {
                            match u64::from_str(s) {
                                Ok(seconds) => {
                                    println!("Engine is thinking...");
                                    engine.start();
                                    thread::sleep(Duration::from_secs(seconds));
                                    engine.stop();
                                    continue;
                                },
                                _ => println!("Could not parse thinking duration, it must be a positive integer."),
                            }
                        },
                        _ => println!("Invalid usage of the \"think\" command, type \"help\" to get the correct usage."),
                    }
                }
                unknown => println!("Unknown command: \"{}\"", unknown),
            }
            _ => (),
        }

        println!("Press enter to continue...");
        io::stdin().read_line(&mut buffer).expect("Cannot read line");
    }
}