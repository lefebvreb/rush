use std::env;
use std::str::FromStr;
use std::thread;

use anyhow::{Error, Result};
use clap::App;

use chess::prelude::*;
use clap::Arg;

/* 
 * For the default position:
 * $ cargo build --bin perft --release 
 * $ target/release/perft 6 "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
 *
 * For profiling using perf:
 * $ cargo build --bin perft --release
 * $ perf record --call-graph dwarf target/release/perft 4 "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
 * $ perf report
 * Don't forget this also benchmarks initialization costs, as well as argument parsing.
 *
 * For a quick and dirty benchmark:
 * $ cargo build --bin perft --release 
 * $ time target/release/perft 6 "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
 */

fn main() -> Result<()> {
    // Get the args to the program.
    let args = App::new("Rush chess engine perft debugger")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Benjamin Lefebvre")
        .about("A binary to be used along perftree (https://github.com/agausmann/perftree), designed to help debug the move generator. It can also be used for profiling.")
        .arg(Arg::with_name("depth")
            .index(1)
            .value_name("DEPTH")
            .help("The maximum depth at which to expand the game tree.")
            .required(true))
        .arg(Arg::with_name("fen")
            .index(2)
            .value_name("FEN")
            .help("The fen of the starting position. Use double quotes.")
            .required(true))
        .arg(Arg::with_name("moves")
            .index(3)
            .value_name("MOVES")
            .help("A space seperated serie of moves to perform before beginning game tree expansion."))
        .get_matches();

    // Parse depth.
    let depth = usize::from_str(args.value_of("depth").unwrap()).map_err(|_| Error::msg("Unable to parse depth."))?;
    if !(0..=12).contains(&depth) {
        return Err(Error::msg("Invalid depth, depth must be between 1 and 12."));
    }
    
    // Initialize the chess library.
    chess::init();

    // Parse the board.
    let mut board = Board::from_str(args.value_of("fen").unwrap())?;

    // Parse and do the moves to apply.
    if let Some(arg) = args.value_of("moves") {
        for s in arg.split(' ') {
            let mv = board.parse_move(s)?;
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

        // Bulk-count the number of nodes.
        total = list.len() as u64;
    } else {
        // Launch a thread for each move.
        let mut handles = Vec::new();

        // For each thread, assign it a move to perform before perft.
        for &mv in list.iter() {
            let mut board = board.clone();

            handles.push(thread::spawn(move || {
                board.do_move(mv);
                movegen::perft(&mut board, depth - 1)
            }));
        }

        // Join all thread handles and get results.
        for (handle, mv) in handles.into_iter().zip(list) {
            let count = handle.join().unwrap();
            println!("{} {}", mv, count);
            total += count;
        }
    }

    // Print the total after an empty line.
    println!("\n{}", total);

    // Successfully return.
    Ok(())
}
