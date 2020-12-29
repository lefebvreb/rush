// cargo test --package chess --test perft

use chess::prelude::*;

// The depth at which the perft test is carried, may be changed
const DEPTH: usize = 2;
// min: 0, max: 13

// Pre-computed perft results, taken from https://www.chessprogramming.org/Perft_Results
const PERFT_RESULTS: &[u64] = &[
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
    2_097_651_003_696_806,
    62_854_969_236_701_747,
    1_981_066_775_000_396_239,
];

// The perft algorithm, counting the number of leaf nodes
fn perft(game: &mut Game, depth: usize) -> u64 {
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

// The test: run perft and compare with correct results
#[test]
fn run_perft() {
    let mut game = Game::default();

    assert_eq!(perft(&mut game, DEPTH), PERFT_RESULTS[DEPTH]);
}