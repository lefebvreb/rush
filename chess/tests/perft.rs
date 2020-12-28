// cargo test --package chess --test perft -- perft_test --exact --nocapture

use chess::prelude::*;

const PERFT_RESULTS: &'static [u64] = &[
    1,
    20,
    400,
    8_902,
    197_281,
    4_865_609,
    119_060_324,
    3_195_901_860,
    84_998_978_956,
    2_439_530_234_167,
    69_352_859_712_417,
];

fn perft(game: &mut Game, depth: usize) -> u64 {
    if depth == 0 {
        return 1;
    }

    let mut nodes = 0;
    let mut move_gen = MoveGenerator::legals(unsafe {&mut *(game as *mut Game)}); // ugly

    loop {
        let mv = move_gen.next();
        if mv.is_none() {break}
        game.do_move(mv);
        nodes += perft(game, depth-1);
        game.undo_move();
    }

    nodes
}

#[test]
fn perft_test() {
    let mut game = Game::default();

    let depth = 3;

    assert_eq!(perft(&mut game, depth), PERFT_RESULTS[depth]);
}