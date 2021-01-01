// cargo run --bin perft -- <depth> "<fen>" "(<moves> )+"?
// cargo build --bin perft --release

use std::env::args;
use std::str::FromStr;

use chess::*;

// The perft algorithm, counting the number of leaf nodes
fn perft(game: &mut SearchGame<15>, depth: usize) -> u64 {
    if depth == 0 {
        return 1;
    }

    let mut nodes = 0;
    let mut move_gen = game.legals();

    loop {
        let mv = move_gen.next();
        if mv.is_none() {break}

        game.do_move(mv);
        nodes += perft(game, depth - 1);
        game.undo_move();
    }

    nodes
}

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
    let mut game = FullGame::from_fen(&fen).expect("Cannot parse FEN");

    // Moves to apply
    if args.len() != 0 {
        for s in args.next().unwrap().split(" ") {
            game.do_move(game.parse_move(&s).expect("Couldn't parse move"));
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