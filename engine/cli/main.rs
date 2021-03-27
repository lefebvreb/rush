// A command line interface tool to invoke the engine for a given FEN position.
// Primarily intended for profiling the engine.
//
// Usage:
//   $ ./cli <fen>
//     <fen> : the fen string to be used, put it into quotes
//
// For profiling with perf:
//   $ perf record --call-graph dwarf target/debug/cli "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
//   $ perf report

use std::env::args;

use engine;

fn main() {
    let mut args = args();

    // Executable path
    args.next().unwrap();

    // FEN position
    let fen = args.next().expect("Cannot find FEN argument");

    // Initialize the engine
    engine::initialize(&fen).expect("Cannot parse FEN");

    // Compute the move for the given position
    let mv = engine::compute();

    // Print the formatted move
    println!("{}", mv.to_string());
}