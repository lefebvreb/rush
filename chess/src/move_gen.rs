use std::ops::{Generator, GeneratorState};
use std::pin::Pin;

use crate::attacks::*;
use crate::bitboard::BitBoard;
use crate::castle_rights::CastleAvailability;
//use crate::color::Color;
use crate::game::Game;
use crate::moves::Move;
use crate::piece::Piece;
//use crate::square::Square;

#[repr(u8)]
#[derive(Debug)]
enum PawnPromote {
    Queen,
    Rook,
    Bishop,
    Knight,
}

impl PawnPromote {
    #[inline(always)]
    pub fn next(&mut self) {
        *self = match self {
            PawnPromote::Queen => PawnPromote::Rook,
            PawnPromote::Rook => PawnPromote::Bishop,
            PawnPromote::Bishop => PawnPromote::Knight,
            PawnPromote::Knight => PawnPromote::Queen,
        };
    }
}

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
        let game = unsafe {& *(self as *const Game)};
        
        move || {
            // Board and colors
            let board = game.get_board();
            let color = game.get_color();
            let color_inv = color.invert();

            // Occupancy bitboards
            let occ = board.get_occupancy();
            let us = board.get_color_occupancy(color);
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
            let safe = !danger;

            // Pinned pieces bitboard
            let pinned = get_pinned(color, board);

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
                    let king_defend = board.get_defend_unchecked(king_sq) & safe;

                    for to in (king_defend & them).iter_squares() {
                        yield Move::Capture {
                            from: king_sq,
                            to,
                            capture: board.get_piece_unchecked(to),
                        };
                    }

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

            // Pawn captures
            // Rook Captures
            // Bishop Captures
            // Knight Captures
            // Queen Captures
            // King Captures
            // En passant
            // PawnPushes
            // Rook Quiets
            // Bishop Quiets
            // Knight Quiets
            // Queen Quiets
            // King Quiets
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Quick test to check the correctness of the
    // move generator. For a more in-depth, full test,
    // perft is used. It is implemented in the file
    // chess/tests/perft.rs
    #[test]
    fn openings() {
        let game = Game::default();

        let moves = game.legals().collect();

        println!("{:?}", moves.len());
        println!("{:?}", moves);
    }
}