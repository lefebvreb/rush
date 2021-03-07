use std::collections::HashMap;
use std::ops::{Generator, GeneratorState};
use std::pin::Pin;

use crate::attacks::*;
use crate::bitboard::BitBoard;
use crate::castle_rights::CastleAvailability;
use crate::color::Color;
use crate::en_passant::{EnPassantAvailability, EnPassantSquare};
use crate::game::Game;
use crate::moves::Move;
use crate::piece::Piece;
use crate::square::Square;

// A table containing the pieces ordered by they value,
// useful for generating captures in the right order.
// King is excluded because it can never be captured
const CAPTURES: &[Piece] = &[
    Piece::Queen,
    Piece::Rook,
    Piece::Bishop,
    Piece::Knight,
    Piece::Pawn,
];

// A table containing the pieces ordered by lesser
// values, to allow move ordering
const CAPTURERS: &[Piece] = &[
    Piece::Knight,
    Piece::Bishop,
    Piece::Rook,
    Piece::Queen,
];

// A table containing the promotions of a pawn, in the ordered
// in which there are generated
const PROMOTIONS: &[Piece] = &[
    Piece::Queen,
    Piece::Rook,
    Piece::Bishop,
    Piece::Knight,
];

//#################################################################################################
//
//                                   trait MoveGenerator
//
//#################################################################################################

/// A trait used to provide an iterator-like interface for
/// dealing with move generators
pub trait MoveGenerator {
    /// Generate and return the next move of the generator. Panics
    /// if the last move returned was `Move::None`
    fn next(&mut self) -> Option<Move>;

    /// Collect the generator into a `Vec` of `Move`
    fn to_vec(&mut self) -> Vec<Move> {
        let mut res = Vec::new();

        while let Some(mv) = self.next() {
            res.push(mv);
        }

        res
    }

    /// Collect the generator into a `HashMap` with `String` as keys and `Move` as values
    fn to_map(&mut self) -> HashMap<String, Move> {
        let mut res = HashMap::new();

        while let Some(mv) = self.next() {
            res.insert(mv.to_string(), mv);
        }

        res
    }
}

// Implementation of the MoveGenerator trait for rust's native Generator traits
#[doc(hidden)]
impl<G: Generator<(), Yield=Move, Return=()> + Unpin> MoveGenerator for G {
    #[inline(always)]
    fn next(&mut self) -> Option<Move> {
        match Pin::new(self).resume(()) {
            GeneratorState::Yielded(mv) => Some(mv),
            GeneratorState::Complete(_) => None,
        }
    }
}

impl Game {
    /// Return a generator able to produce the legal moves associated
    /// to a specific position
    #[inline(always)]
    pub fn legals<'a>(&'a self) -> impl MoveGenerator + 'a {        
        move || {
            // Board and colors
            let board = self.get_board();
            let color = self.get_color();
            let color_inv = color.invert();

            // Occupancy bitboards
            let occ = board.get_occupancy();
            let them = board.get_color_occupancy(color_inv);
            let free = board.get_free();

            // King square and squares attacking the king
            let king_sq = board.get_bitboard(color, Piece::King).as_square_unchecked();
            let king_attacks = board.get_attacks(king_sq) & them;

            // All squares attacked by them
            let danger = Piece::PIECES.iter()
                .fold(BitBoard::EMPTY, |danger, &piece|
                    danger | board.get_bitboard(color_inv, piece)
                        .iter_squares()
                        .fold(BitBoard::EMPTY, |danger, sq|
                            danger | board.get_defend_unchecked(sq)
                        )
                );

            // The king's unreachable squares
            macro_rules! king_danger {
                () => {
                    danger | king_attacks.iter_squares()
                        .fold(BitBoard::EMPTY, |danger, checker_sq|
                            danger | if board.get_piece_unchecked(checker_sq).is_slider() {
                                get_projected_mask(checker_sq, king_sq)
                            } else {
                                BitBoard::EMPTY
                            }
                        );
                }
            }

            // Pinned pieces bitboard
            let pinned = get_pinned(color, board);

            // Macro to give, if needed, the pin mask
            macro_rules! pin {
                ($from: expr) => {
                    if pinned.contains($from) {
                        get_projected_mask(king_sq, $from)
                    } else {
                        BitBoard::FULL
                    }
                };
            }

            // Count how many checkers there are
            let check_mask = match king_attacks.count_bits() {
                0 => { // No checkers: may castle
                    match self.get_castle_rights().get_availability(color, occ, danger) {
                        CastleAvailability::KingSide => yield Move::KingCastle {color},
                        CastleAvailability::QueenSide => yield Move::QueenCastle {color},
                        CastleAvailability::Both => {
                            yield Move::KingCastle {color};
                            yield Move::QueenCastle {color};
                        }
                        _ => (),
                    };

                    BitBoard::FULL
                }
                1 => { // One checker: extra mask to apply during move generation
                    let checker_sq = king_attacks.as_square_unchecked();
                    squares_between(king_sq, checker_sq) | checker_sq.into()
                }
                2 => { // Two checkers: can only capture with king or move with king
                    let king_defend = board.get_defend_unchecked(king_sq) & !king_danger!();

                    // King captures
                    for &piece in CAPTURES {
                        for to in (king_defend & board.get_bitboard(color_inv, piece)).iter_squares() {
                            yield Move::Capture {
                                from: king_sq,
                                to,
                                capture: board.get_piece_unchecked(to),
                            };
                        }
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
                        for &promote in PROMOTIONS {
                            yield Move::PromoteCapture {
                                from, 
                                to, 
                                capture: board.get_piece_unchecked(to),
                                promote,
                            };
                        }
                    }
                } else {
                    for &piece in CAPTURES {
                        for to in (defend & board.get_bitboard(color_inv, piece)).iter_squares() {
                            yield Move::Capture {
                                from, 
                                to, 
                                capture: board.get_piece_unchecked(to),
                            };
                        }
                    }               
                }
            }

            // En passant
            if let EnPassantSquare::Some(mid) = self.get_ep_rights() {
                let to = match color_inv {
                    Color::White => Square::from((mid.x(), 3)),
                    Color::Black => Square::from((mid.x(), 4)),
                };

                if !(pinned.contains(to)) {
                    let en_passant = EnPassantAvailability::get(color, color_inv, to, king_sq, board);

                    let mask = if check_mask.contains(to) {
                        BitBoard::FULL
                    } else {
                        check_mask
                    };

                    macro_rules! yield_en_passant {
                        ($from: ident) => {
                            if (mask & pin!($from)).contains(mid) {
                                yield Move::EnPassant {
                                    from: $from,
                                    to: mid,
                                };
                            }
                        }
                    }

                    match en_passant {
                        EnPassantAvailability::Left(left) => yield_en_passant!(left),
                        EnPassantAvailability::Right(right) => yield_en_passant!(right),
                        EnPassantAvailability::Both {left, right} => {
                            yield_en_passant!(left);
                            yield_en_passant!(right);
                        }
                        _ => (),
                    }                    
                }
            }

            // Rook, Bishop, Knight and Queen captures
            for &piece in CAPTURERS {
                for from in board.get_bitboard(color, piece).iter_squares() {
                    let defend = board.get_defend_unchecked(from) & mask & pin!(from);

                    for &piece in CAPTURES {
                        for to in (defend & board.get_bitboard(color_inv, piece)).iter_squares() {
                            yield Move::Capture {
                                from, 
                                to, 
                                capture: board.get_piece_unchecked(to),
                            };
                        }
                    } 
                }
            }

            // The defend bitboard of the king
            let king_defend = board.get_defend_unchecked(king_sq) & !king_danger!();

            // King captures
            for &piece in CAPTURES {
                for to in (king_defend & board.get_bitboard(color_inv, piece)).iter_squares() {
                    yield Move::Capture {
                        from: king_sq,
                        to,
                        capture: board.get_piece_unchecked(to),
                    };
                }
            }

            // Quiets mask
            let mask = check_mask & free;

            // Pawn quiets and double pushes
            for from in board.get_bitboard(color, Piece::Pawn).iter_squares() {
                let mask = mask & pin!(from);

                if let Some(to_single) = get_pawn_push(color, from) {
                    if mask.contains(to_single) {
                        if to_single.is_last_rank(color) {
                            for &promote in PROMOTIONS {
                                yield Move::Promote {
                                    from,
                                    to: to_single,
                                    promote,
                                };
                            }
                        } else {
                            yield Move::Quiet {
                                from,
                                to: to_single,
                            };
                        }                        
                    }

                    if let Some(to_double) = get_pawn_double_push(color, from) {
                        if mask.contains(to_double) & board.is_empty(to_single) {
                            yield Move::DoublePush {
                                from,
                                to: to_double,
                            };
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