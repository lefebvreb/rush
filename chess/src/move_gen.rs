use std::ops::{Generator, GeneratorState};
use std::pin::Pin;

use crate::attacks::*;
use crate::bitboard::BitBoard;
use crate::castle_rights::CastleAvailability;
use crate::game::Game;
use crate::moves::Move;
use crate::piece::Piece;

// A table containing the promotions of a pawn, in the ordered
// in which there are generated
const PROMOTIONS: &[Piece] = &[
    Piece::Queen,
    Piece::Rook,
    Piece::Bishop,
    Piece::Knight,
];

/// A trait used to provide an iterator-like interface for
/// dealing with move generators
pub trait MoveGenerator {
    /// Generate and return the next move of the generator. Panics
    /// if the last move returned was `Move::None`
    fn next(&mut self) -> Move;

    /// Collect the generator into a `Vec` of `Move`
    #[cold]
    fn collect(&mut self) -> Vec<Move> {
        (0..)
            .map(|_| self.next())
            .take_while(|mv| !mv.is_none())
            .collect()
    }
}

// Implementation of the MoveGenerator trait for rust's native Generator traits
#[doc(hidden)]
impl<G: Generator<(), Yield=Move, Return=()> + Unpin> MoveGenerator for G {
    #[inline(always)]
    fn next(&mut self) -> Move {
        match Pin::new(self).resume(()) {
            GeneratorState::Yielded(mv) => mv,
            GeneratorState::Complete(_) => Move::None,
        }
    }
}

impl Game {
    /// Return a generator able to produce the legal moves associated
    /// to a specific position. Keeps a reference to `self`, for
    /// generation correctness, the value of `self` should not change
    /// between each call to `next()`
    #[inline(always)]
    pub fn legals(&self) -> impl MoveGenerator {
        // Second immutable reference to safe, needed to be able to do and undo moves
        // between each call to resume(())
        let game = unsafe {& *(self as *const Game)};
        
        move || {
            // Board and colors
            let board = game.get_board();
            let color = game.get_color();
            let color_inv = color.invert();

            // Occupancy bitboards
            let occ = board.get_occupancy();
            let them = board.get_color_occupancy(color_inv);
            let free = board.get_free();

            // King square and squares attacking the king
            let king_sq = board.get_bitboard(color, Piece::King).as_square_unchecked();
            let king_attacks = board.get_attacks(king_sq) & them;

            // King's danger and safety bitboards
            let danger = king_attacks.iter_squares()
                .fold(BitBoard::EMPTY, |danger, sq| {
                    danger | board.get_defend_unchecked(sq)
                });

            // Pinned pieces bitboard
            let pinned = get_pinned(color, board);

            // Macro to give, if needed, the pin mask
            macro_rules! pin {
                ($from: expr) => {
                    if pinned.contains($from) {
                        get_pin_mask(king_sq, $from)
                    } else {
                        BitBoard::FULL
                    }
                };
            }

            // Count how many checkers there are
            let check_mask = match king_attacks.count_bits() {
                0 => { // No checkers: may castle
                    match game.get_castle_rights().get_availability(color, occ, danger) {
                        CastleAvailability::KingSide => yield Move::KingCastle,
                        CastleAvailability::QueenSide => yield Move::QueenCastle,
                        CastleAvailability::Both => {
                            yield Move::KingCastle;
                            yield Move::QueenCastle;
                        }
                        _ => (),
                    };

                    BitBoard::FULL
                }
                1 => { // One checker: extra mask to apply to during move generation
                    let checker_sq = king_attacks.as_square_unchecked();
                    squares_between(king_sq, checker_sq)
                }
                2 => { // Two checkers: can only capture with king or move with king
                    let king_defend = board.get_defend_unchecked(king_sq) & !danger;

                    // King captures
                    for to in (king_defend & them).iter_squares() {
                        yield Move::Capture {
                            from: king_sq,
                            to,
                            capture: board.get_piece_unchecked(to),
                        };
                    }

                    // King quiets
                    for to in (king_defend & free).iter_squares() {
                        yield Move::Quiet {
                            from: king_sq,
                            to,
                        };
                    }

                    return;
                }
                _ => unreachable!(),
            };

            // Capture mask
            let mask = check_mask & them;

            // Pawn captures and promote-captures
            for from in board.get_bitboard(color, Piece::Pawn).iter_squares() {
                let defend = board.get_defend_unchecked(from) & mask & pin!(from);

                if defend.is_last_rank(color) {
                    for to in defend.iter_squares() {
                        for promote in PROMOTIONS {
                            yield Move::PromoteCapture {
                                from, 
                                to, 
                                capture: board.get_piece_unchecked(to),
                                promote: *promote,
                            };
                        }
                    }
                } else {
                    for to in defend.iter_squares() {
                        yield Move::Capture {
                            from, 
                            to, 
                            capture: board.get_piece_unchecked(to),
                        };
                    }                    
                }
            }

            // Rook, Bishop, Knight and Queen captures
            for from in (
                board.get_bitboard(color, Piece::Rook)
                | board.get_bitboard(color, Piece::Bishop)
                | board.get_bitboard(color, Piece::Knight)
                | board.get_bitboard(color, Piece::Queen)
            ).iter_squares() {
                let defend = board.get_defend_unchecked(from) & mask & pin!(from);

                for to in defend.iter_squares() {
                    yield Move::Capture {
                        from, 
                        to, 
                        capture: board.get_piece_unchecked(to),
                    };
                }
            }

            // The defend bitboard of the king
            let king_defend = board.get_defend_unchecked(king_sq) & !danger;

            // King captures
            for to in (king_defend & them).iter_squares() {
                yield Move::Capture {
                    from: king_sq,
                    to,
                    capture: board.get_piece_unchecked(to),
                };
            }

            // En passant
            // TODO

            // Quiets mask
            let mask = check_mask & free;

            // Pawn quiets and double pushes
            for from in board.get_bitboard(color, Piece::Pawn).iter_squares() {
                let pin = pin!(from);

                if let Some(to) = get_pawn_push(color, from) {
                    if board.is_empty(to) & pin.contains(to) {
                        yield Move::Quiet {
                            from,
                            to,
                        };

                        if let Some(to) = get_pawn_double_push(color, from) {
                            if board.is_empty(to) & pin.contains(to) {
                                yield Move::DoublePush {
                                    from,
                                    to,
                                };
                            }
                        }
                    }
                }
            }

            // Rook, Bishop, Knight and Queen quiets
            for from in (
                board.get_bitboard(color, Piece::Rook)
                | board.get_bitboard(color, Piece::Bishop)
                | board.get_bitboard(color, Piece::Knight)
                | board.get_bitboard(color, Piece::Queen)
            ).iter_squares() {
                let defend = board.get_defend_unchecked(from) & mask & pin!(from);

                for to in defend.iter_squares() {
                    yield Move::Quiet {
                        from, 
                        to,
                    };
                }
            }

            // King Quiets
            for to in (king_defend & free).iter_squares() {
                yield Move::Quiet {
                    from: king_sq,
                    to,
                };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::square::Square;

    // Quick test to check the correctness of the
    // move generator. For a more in-depth, full test,
    // perft is used. It is implemented in the file
    // chess/tests/perft.rs
    #[test]
    fn openings() {
        let game = Game::default();
        let moves = game.legals().collect();

        assert_eq!(moves, vec![
            Move::Quiet {from: Square::A2, to: Square::A3}, 
            Move::DoublePush {from: Square::A2, to: Square::A4},
            Move::Quiet {from: Square::B2, to: Square::B3}, 
            Move::DoublePush {from: Square::B2, to: Square::B4}, 
            Move::Quiet {from: Square::C2, to: Square::C3}, 
            Move::DoublePush {from: Square::C2, to: Square::C4}, 
            Move::Quiet {from: Square::D2, to: Square::D3}, 
            Move::DoublePush {from: Square::D2, to: Square::D4}, 
            Move::Quiet {from: Square::E2, to: Square::E3}, 
            Move::DoublePush {from: Square::E2, to: Square::E4}, 
            Move::Quiet {from: Square::F2, to: Square::F3}, 
            Move::DoublePush {from: Square::F2, to: Square::F4}, 
            Move::Quiet {from: Square::G2, to: Square::G3}, 
            Move::DoublePush {from: Square::G2, to: Square::G4}, 
            Move::Quiet {from: Square::H2, to: Square::H3}, 
            Move::DoublePush {from: Square::H2, to: Square::H4}, 
            Move::Quiet {from: Square::B1, to: Square::A3},
            Move::Quiet {from: Square::B1, to: Square::C3}, 
            Move::Quiet {from: Square::G1, to: Square::F3}, 
            Move::Quiet {from: Square::G1, to: Square::H3}
        ])
    }
}