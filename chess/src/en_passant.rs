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
#[derive(Clone, Copy, Debug)]
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
        macro_rules! is_color_pawn {
            ($sq: expr) => {
                matches!(board.get_piece($sq), Some((c, Piece::Pawn)) if c == color)
            }
        }

        let x = pawn_sq.x();

        if x == 0 {
            // Left-most
            let rsq = pawn_sq.get_right_unchecked();

            if is_color_pawn!(rsq) {
                // There's one of our pawn right of our target
                EnPassantAvailability::Right(rsq)
            } else {
                EnPassantAvailability::None
            }
        } else if x == 7 {
            // Right-most
            let lsq = pawn_sq.get_left_unchecked();
            
            if is_color_pawn!(lsq) {
                // There's one of our pawn left of our target
                EnPassantAvailability::Left(lsq)
            } else {
                EnPassantAvailability::None
            }
        } else {
            // Not on the side
            let lsq = pawn_sq.get_left_unchecked();
            let rsq = pawn_sq.get_right_unchecked();

            // Our pawns
            let (l, r) = (is_color_pawn!(lsq), is_color_pawn!(rsq));

            // We have exactly one pawn and our king is on that rank
            if l ^ r && pawn_sq.y() == king_sq.y() {
                // Beware of horizontal sliders
                let sliders = board.get_bitboard(color_inv, Piece::Rook) | board.get_bitboard(color_inv, Piece::Queen);
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

            match (l, r) {
                (false, false) => EnPassantAvailability::None,
                (true, false)  => EnPassantAvailability::Left(lsq),
                (false, true)  => EnPassantAvailability::Right(rsq),
                (true, true)   => EnPassantAvailability::Both {left: lsq, right: rsq},
            }
        }
    }
}