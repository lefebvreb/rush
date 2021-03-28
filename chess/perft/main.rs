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
// For profiling with perf:
//   $ cargo build --bin perft --release
//   $ perf record --call-graph dwarf target/release/perft 3 "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
//   $ perf report

use std::env::args;
use std::str::FromStr;

use chess::*;

// The perft algorithm, counting the number of leaf nodes
fn perft(game: Game, depth: usize) -> u64 {
    if depth == 0 {
        return 1;
    }

    let mut nodes = 0;
    let mut move_gen = game.legals();

    while let Some(mv) = move_gen.next() {
        nodes += perft(game.do_move(mv), depth - 1);
    }

    nodes
}

fn main() {
    let mut args = args();
    
    // Executable path
    args.next().unwrap();

    // Perft depth
    let depth = usize::from_str(&args.next().expect("Cannot find depth argument")).expect("Cannot parse depth");
    assert!(depth < 10, "A depth of {} is way too much, 10 is max", depth);

    // FEN position
    let fen = args.next().expect("Cannot find FEN argument");
    let mut game = Game::from_fen(&fen).expect("Cannot parse FEN");

    // Moves to apply
    if args.len() != 0 {
        for s in args.next().unwrap().split(" ") {
            let mv = game.parse_move(&s).expect("Could not parse move");
            game = game.do_move(mv);
        }
    }

    // Total number of positions found
    let mut total = 0;

    // Do perft and count nodes
    for (s, mv) in game.legals().to_map() {
        let count = perft(game.do_move(mv), depth-1);
        println!("{} {}", s, count);
        total += count;
    }

    // Prints the total after an empty line
    println!("\n{}", total);
}
