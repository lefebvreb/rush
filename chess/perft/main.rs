// The goal of this binary is to be used by perftree (https://github.com/agausmann/perftree)
// to help debug the move generator. 
// It can also be used for profiling.
//
// Usage: 
//   $ ./perft <depth> <fen> <moves>
//     <depth> : The depth at which the perft needs to be carried
//     <fen>   : the fen string to be used, put it into quotes
//     <moves> : (optional) a list of space seperated moves, in pure algebraic
//               coordinates notation, to be performed before node counting.
//               Needs to be a single arguments, use quotes
//
// Ex:
//   $ cargo run --bin perft -- 3 "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
//
// For profiling with perf:
//   $ cargo build --bin perft --release
//   $ perf record --call-graph dwarf target/release/perft 3 "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
//   $ perf report

use std::env::args;
use std::str::FromStr;

use chess::prelude::*;

// The perft algorithm, counting the number of leaf nodes.
fn perft(board: &mut Board, depth: usize) -> u64 {
    if depth == 0 {
        return 1;
    }

    let mut nodes = 0;
    
    let mut list = movegen::MoveList::new();
    movegen::legals(&board, &mut list);

    for &mv in list.iter() {
        board.do_move(mv);
        nodes += perft(board, depth - 1);
        board.undo_move(mv);
    }

    nodes
}

fn main() {
    let mut args = args();
    
    // Executable path.
    args.next().unwrap();

    // Perft depth.
    let depth = usize::from_str(&args.next().expect("Cannot find depth argument")).expect("Cannot parse depth");
    assert!(depth <= 10, "Exceeded maximum depth of 10");

    // fen position.
    let fen = args.next().expect("Cannot find fen argument");
    let mut board = Board::from_str(&fen).expect("Cannot parse fen");

    // Moves to apply
    if args.len() != 0 {
        for s in args.next().unwrap().split(" ") {
            let mv = board.parse_move(&s).expect("Could not parse move");
            board.do_move(mv);
        }
    }

    // Total number of nodes found.
    let mut total = 0;

    // Compute the legal moves of the starting position.
    let mut list = movegen::MoveList::new();
    movegen::legals(&board, &mut list);

    // Do perft and count nodes.
    for &mv in list.iter() {
        board.do_move(mv);
        let count = perft(&mut board, depth - 1);
        board.undo_move(mv);
        println!("{} {}", mv, count);
        total += count;
    }

    // Print the total after an empty line.
    println!("\n{}", total);
}
