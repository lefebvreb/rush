// The goal of this binary is to be used by perftree (https://github.com/agausmann/perftree)
// to help debug the move generator. 
// It can also be used for profiling.
//
// Usage: 
//   $ ./perft <depth> <fen> <moves>
//     <depth> : The depth at which the perft needs to be carried.
//     <fen>   : the fen string to be used, put it into quotes.
//     <moves> : (optional) a list of space seperated moves, in pure algebraic
//               coordinates notation, to be performed before node counting.
//               Needs to be a single arguments, use quotes.
//
// Ex:
//   $ ./perft -- 3 "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
//
// For profiling with perf:
//   $ cargo build --bin perft --release
//   $ perf record --call-graph dwarf target/release/perft 3 "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
//   $ perf report

use std::env;
use std::str::FromStr;
use std::thread;

use chess::prelude::*;

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
    args.next().unwrap();

    // Perft depth.
    let depth = usize::from_str(&args.next().expect("Cannot find depth argument")).expect("Cannot parse depth");
    assert!(depth > 0, "Depth should be at least one");
    assert!(depth <= 12, "Exceeded maximum depth of twelve");

    // fen position.
    let fen = args.next().expect("Cannot find fen argument");
    let mut board = Board::new(&fen).expect("Cannot parse fen");

    // Moves to apply
    if args.len() != 0 {
        for s in args.next().unwrap().split(' ') {
            let mv = board.parse_move(s).expect("Could not parse move");
            board.do_move(mv);
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
