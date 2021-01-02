// cargo run --bin perft -- <depth> "<fen>" "(<moves> )+"?
// cargo build --bin perft --release

use std::env::args;
use std::str::FromStr;

use chess::*;

mod perft;
use perft::perft;

// How to use the perft script:
// cargo run --bin perft --release -- <depth> "<fen>" "(<moves> )+"?
fn main() {
    let mut args = args();
    
    // Executable path
    args.next().unwrap();

    // Perft depth
    let depth = usize::from_str(&args.next().expect("Cannot find depth argument")).expect("Cannot parse depth");

    // FEN position
    let fen = args.next().expect("Cannot find FEN argument");
    let mut game = Game::from_fen(&fen).expect("Cannot parse FEN");

    // Moves to apply
    if args.len() != 0 {
        for s in args.next().unwrap().split(" ") {
            game.do_move(game.parse_move(&s).expect("Could not parse move"));
        }
    }

    // Total number of positions found
    let mut total = 0;
    // A search-game
    let mut game = game.search_game();

    // Do perft and count nodes
    for (s, mv) in game.legals().to_map() {
        game.do_move(mv);
        let count = perft(&mut game, depth-1);
        println!("{} {}", s, count);
        total += count;
        game.undo_move();
    }

    // Prints the total after an empty line
    println!("\n{}", total);
}