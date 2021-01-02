use chess::*;

// The perft algorithm, counting the number of leaf nodes
pub fn perft(game: &mut SearchGame<15>, depth: usize) -> u64 {
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