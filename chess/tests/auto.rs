use chess::*;

mod perft;
use perft::perft;

#[test]
pub fn perft1() {
    let mut game = Game::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
    assert_eq!(perft(&mut game, 5), 4_865_609)
}

#[test]
pub fn perft2() {
    let mut game = Game::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1").unwrap();
    assert_eq!(perft(&mut game, 5), 193_690_690)
}

#[test]
pub fn perft3() {
    let mut game = Game::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").unwrap();
    assert_eq!(perft(&mut game, 7), 178_633_661)
}

#[test]
pub fn perft4() {
    let mut game = Game::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1").unwrap();
    assert_eq!(perft(&mut game, 5), 15_833_292)
}

#[test]
pub fn perft5() {
    let mut game = Game::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8").unwrap();
    assert_eq!(perft(&mut game, 5), 89_941_194)
}

#[test]
pub fn perft6() {
    let mut game = Game::from_fen("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10").unwrap();
    assert_eq!(perft(&mut game, 5), 164_075_551)
}