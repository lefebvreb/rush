use std::str::FromStr;

use chess::*;

// The depth at which the test is carried
const DEPTH: usize = 3;

// Explore the game tree and test if the zobrist key incremental change is
// correctly following
fn zobrist(game: Game, depth: usize) {
    if depth == 0 {
        let recalc = Game::from_str(&game.to_string()).unwrap();
        assert_eq!(game.get_zobrist(), recalc.get_zobrist());
        return;
    }

    let mut move_gen = game.legals();

    loop {
        let mv = move_gen.next();
        if mv.is_none() {break}

        zobrist(game.do_move(mv), depth - 1);
    }
}

#[test]
fn zobrist1() {
    let game = Game::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
    zobrist(game, DEPTH);
}

#[test]
fn zobrist2() {
    let game = Game::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1").unwrap();
    zobrist(game, DEPTH);
}

#[test]
fn zobrist3() {
    let game = Game::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").unwrap();
    zobrist(game, DEPTH);
}

#[test]
fn zobrist4() {
    let game = Game::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1").unwrap();
    zobrist(game, DEPTH);
}

#[test]
fn zobrist5() {
    let game = Game::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8").unwrap();
    zobrist(game, DEPTH);
}

#[test]
fn zobrist6() {
    let game = Game::from_fen("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10").unwrap();
    zobrist(game, DEPTH);
}