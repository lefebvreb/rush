use std::fmt;
use std::str::FromStr;

use crate::attacks::squares_between;
use crate::board::Board;
use crate::color::Color;
use crate::errors::ParseFenError;
use crate::moves::Move;
use crate::piece::Piece;
use crate::square::Square;

//#################################################################################################
//
//                                  enum EnPassantSquare
//
//#################################################################################################

// Keep track off the en passant target square
#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub(crate) enum EnPassantSquare {
    Some(Square),
    None,
}

// ================================ pub(crate) impl

impl EnPassantSquare {
    // Update and return the new en passant rights, and modify the zobrist key accordingly
    pub(crate) fn update(&self, last_move: Move) -> EnPassantSquare {
        match last_move {
            Move::DoublePush {from, to} => {
                let mid = from.get_mid(to);
                EnPassantSquare::Some(mid)
            },
            _ => EnPassantSquare::None,
        }
    }
}

// ================================ traits impl

impl Default for EnPassantSquare {
    fn default() -> EnPassantSquare {
        EnPassantSquare::None
    }
}

impl fmt::Display for EnPassantSquare {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EnPassantSquare::Some(mid) => write!(f, "{}", mid),
            _ => write!(f, "-"),
        }
    }
}

impl FromStr for EnPassantSquare {
    type Err = ParseFenError;

    fn from_str(s: &str) -> Result<EnPassantSquare, ParseFenError> {
        Ok(match s {
            "-" => EnPassantSquare::None,
            s => EnPassantSquare::Some(Square::from_str(s)?),
        })
    }
}

//#################################################################################################
//
//                                enum EnPassantAvailability
//
//#################################################################################################

// A type to represent en passant availability
#[repr(u8)]
#[derive(Debug)]
pub(crate) enum EnPassantAvailability {
    None,
    Left(Square),
    Right(Square),
    Both {
        left: Square,
        right: Square,
    }
}

// ================================ pub(crate) impl

impl EnPassantAvailability {
    // Get the en passant availability of a position
    pub(crate) fn get(color: Color, color_inv: Color, pawn_sq: Square, king_sq: Square, board: &Board) -> EnPassantAvailability {
        let x = pawn_sq.x();

        if x == 0 {
            // Left-most
            let right = pawn_sq.get_right_unchecked();

            match board.get_piece(right) {
                Some((c, Piece::Pawn)) if c == color => EnPassantAvailability::Right(right),
                _ => EnPassantAvailability::None,
            }
        } else if x == 7 {
            // Right-most
            let left = pawn_sq.get_left_unchecked();

            match board.get_piece(left) {
                Some((c, Piece::Pawn)) if c == color => EnPassantAvailability::Left(left),
                _ => EnPassantAvailability::None,
            }
        } else {
            // Not on the side
            let left = pawn_sq.get_left_unchecked();
            let right = pawn_sq.get_right_unchecked();

            let mut flags = 0u8;

            match board.get_piece(left) {
                Some((c, Piece::Pawn)) if c == color => flags |= 0b01,
                _ => (),
            }
            match board.get_piece(right) {
                Some((c, Piece::Pawn)) if c == color => flags |= 0b10,
                _ => (),
            }

            let queens = board.get_bitboard(color_inv, Piece::Queen);

            // Check if capturing may reveal a slider from the side
            match flags {
                0b00 => return EnPassantAvailability::None,
                0b01 | 0b10 => if pawn_sq.y() == king_sq.y() {
                    let sliders = board.get_bitboard(color_inv, Piece::Rook) | queens;
                    let occ = board.get_occupancy();

                    for sq in sliders.iter_squares() {
                        if sq.y() == king_sq.y() {
                            let between = squares_between(king_sq, sq);

                            if between.contains(pawn_sq) && (between & occ).count_bits() == 2 {
                                return EnPassantAvailability::None;
                            }
                        }
                    }
                }
                _ => (),
            }

            match flags {
                0b01 => EnPassantAvailability::Left(left),
                0b10 => EnPassantAvailability::Right(right),
                0b11 => EnPassantAvailability::Both {left, right},
                _ => unreachable!()
            }
        }
    }
}