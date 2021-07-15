use std::env;
use std::str::FromStr;
use std::thread;

use chess::prelude::*;

/// The usage/help of this binary.
const USAGE: &str = r#"The goal of this binary is to be used by perftree (https://github.com/agausmann/perftree) and to help debug the move generator.
It can also be used for profiling.

Usage: 
  ./target/release/perft <depth> <fen> <moves>
    <depth> : The depth at which the perft needs to be carried.
    <fen>   : the fen string to be used, put it into quotes.
    <moves> : (optional) a list of space seperated moves, in pure algebraic
               coordinates notation, to be performed before node counting.
               Needs to be a single arguments, use quotes.

Example:
  ./target/release/perft 3 "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"

For profiling with perf:
  cargo build --bin perft --release
  perf record --call-graph dwarf target/release/perft 3 "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
  perf report"#;

// The perft algorithm, counting the number of leaf nodes.
fn perft(board: &mut Board, depth: usize) -> u64 {
    let mut list = Vec::new();
    movegen::legals(board, &mut list);
    
    if depth == 1 {
        return list.len() as u64;
    }

    let mut nodes = 0;
    
    for &mv in list.iter() {
        board.do_move(mv);
        nodes += perft(board, depth - 1);
        board.undo_move(mv);
    }

    nodes
}

fn main() {
    // Initialize the chess library.
    chess::init();

    // Get the arguments.
    let mut args = env::args();
    
    // Executable path.
    args.next().expect("Can't get executable path.");

    // Parse depth.
    let arg = args.next();
    if arg.is_none() {
        println!("{}", USAGE);
        return;
    }
    let depth = match usize::from_str(arg.unwrap().as_str()) {
        Ok(n) => n,
        Err(err) => {
            println!("{}", err);
            return;
        }
    };
    if !(1..12).contains(&depth) {
        println!("<depth>, must be comprised between 1 and 12 inclusive.");
        return;
    }

    // fen position.
    let arg = args.next();
    if arg.is_none() {
        println!("{}", USAGE);
        return;
    }
    let fen = arg.unwrap();
    let mut board = match Board::new(&fen) {
        Ok(board) => board,
        Err(err) => {
            println!("{}", err);
            return;
        }
    };

    // Moves to apply.
    if let Some(arg) = args.next() {
        for s in arg.split(' ') {
            match board.parse_move(s) {
                Ok(mv) => board.do_move(mv),
                Err(err) => {
                    println!("{}", err);
                    return;
                }
            };
        }
    }

    // Compute the legal moves of the starting position.
    let mut list = Vec::new();
    movegen::legals(&board, &mut list);

    // The total number of nodes.
    let mut total = 0;
    
    if depth == 1 {
        // Special case if depth is only one.
        for &mv in list.iter() {
            println!("{} 1", mv);
        }

        total = list.len() as u64;
    } else {
        // Launch a thread for each move.
        let mut handles = Vec::new();

        for &mv in list.iter() {
            let mut board = board.clone();

            handles.push(thread::spawn(move || {
                board.do_move(mv);
                perft(&mut board, depth - 1)
            }));
        }

        for (handle, mv) in handles.into_iter().zip(list) {
            let count = handle.join().unwrap();
            println!("{} {}", mv, count);
            total += count;
        }
    }

    // Print the total after an empty line.
    println!("\n{}", total);
}
