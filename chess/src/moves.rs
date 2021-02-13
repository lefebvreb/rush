use std::fmt;

use crate::board::Board;
use crate::color::Color;
use crate::piece::Piece;
use crate::square::Square;

//#################################################################################################
//
//                                        enum Move
//
//#################################################################################################

/// A convenient enum to manipulate moves
#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Move {
    None,
    Quiet {
        from: Square,
        to: Square,
    },
    Capture {
        from: Square,
        to: Square,
        capture: Piece,
    },
    Promote {
        from: Square,
        to: Square,
        promote: Piece,
    },
    PromoteCapture {
        from: Square,
        to: Square,
        capture: Piece,
        promote: Piece,
    },
    EnPassant {
        from: Square,
        to: Square,
    },
    DoublePush {
        from: Square,
        to: Square,
    },
    KingCastle {
        color: Color,
    },
    QueenCastle {
        color: Color,
    },
}

// ================================ pub impl

impl Move {
    /// Return the square from which the move is performed
    #[inline(always)]
    pub fn from(self) -> Square {
        match self {
            Move::Quiet {from, ..} | 
            Move::Capture {from, ..} | 
            Move::Promote {from, ..} | 
            Move::PromoteCapture {from, ..} | 
            Move::EnPassant {from, ..} | 
            Move::DoublePush {from, ..} => from,
            Move::KingCastle {color} | 
            Move::QueenCastle {color} => match color {
                Color::White => Square::E1,
                Color::Black => Square::E8,
            },
            _ => unreachable!(),
        }
    }

    /// Return the square to which the move is performed
    #[inline(always)]
    pub fn to(self) -> Square {
        match self {
            Move::Quiet {to, ..} | 
            Move::Capture {to, ..} | 
            Move::Promote {to, ..} | 
            Move::PromoteCapture {to, ..} | 
            Move::EnPassant {to, ..} | 
            Move::DoublePush {to, ..} => to,
            Move::KingCastle {color} => match color {
                Color::White => Square::G1,
                Color::Black => Square::G8,
            },
            Move::QueenCastle {color} => match color {
                Color::White => Square::C1,
                Color::Black => Square::C8,
            },
            _ => unreachable!(),
        }
    }

    /// Return true if the move is none
    #[inline(always)]
    pub fn is_none(self) -> bool {
        matches!(self, Move::None)
    }

    /// Return true if the move is not none
    #[inline(always)]
    pub fn is_some(self) -> bool {
        !self.is_none()
    }
}

// ================================ pub(crate) impl

impl Move {
    // Return true if the move is reversible (according to FIDE rules)
    #[inline(always)]
    pub(crate) fn is_reversible(self, board: &Board) -> bool {
        match self {
            Move::KingCastle {..} | Move::QueenCastle {..} => true,
            Move::Quiet {from, ..} => board.get_piece_unchecked(from) != Piece::Pawn,
            _ => false,
        }
    }

    // Return true if the move is truly reversible
    #[inline(always)]
    pub(crate) fn is_truly_reversible(self, board: &Board) -> bool {
        matches!(self, Move::Quiet {from, ..} if board.get_piece_unchecked(from) != Piece::Pawn)
    }
}

// ================================ traits impl

impl fmt::Display for Move {
    // Display a move using pure algebraic coordinate notation
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let promote = match self {
            Move::Promote {promote, ..} |
            Move::PromoteCapture {promote, ..} => promote.to_string().to_lowercase(),
            _ => "".to_string(),
        };

        write!(fmt, "{}{}{}", self.from(), self.to(), promote)
    }
}

//#################################################################################################
//
//                                   struct EncodedMove
//
//#################################################################################################

/// A compact way to represent a move
#[derive(Clone, Copy, Debug)]
pub struct EncodedMove(u32);

// ================================ pub impl

impl EncodedMove {
    /// Return the EncodedMove as a raw integer
    #[inline(always)]
    pub fn get_raw(self) -> u32 {
        self.0
    }
}

// ================================ traits impl

impl From<Move> for EncodedMove {
    #[inline(always)]
    fn from(mv: Move) -> EncodedMove {
        macro_rules! shl {
            ($val: expr, $n: expr) => {
                (($val as u32) << $n)
            }
        }

        EncodedMove(match mv {
            Move::None => 
                0,
            Move::Quiet {from, to} => 
                1 | shl!(from, 4) | shl!(to, 10),
            Move::Capture {from, to, capture} => 
                2 | shl!(from, 4) | shl!(to, 10) | shl!(capture, 16),
            Move::Promote {from, to, promote} => 
                3 | shl!(from, 4) | shl!(to, 10) | shl!(promote, 20),
            Move::PromoteCapture {from, to, capture, promote} => 
                4 | shl!(from, 4) | shl!(to, 10) | shl!(capture, 16) | shl!(promote, 20),
            Move::EnPassant {from, to} => 
                5 | shl!(from, 4) | shl!(to, 10),
            Move::DoublePush {from, to} => 
                6 | shl!(from, 4) | shl!(to, 10),
            Move::KingCastle {color} => 
                7 | shl!(color, 4),
            Move::QueenCastle {color} =>
                8 | shl!(color, 4),
        })
    }
}

impl Into<Move> for EncodedMove {
    #[inline(always)]
    fn into(self) -> Move {
        macro_rules! shr {
            ($n: expr, $m: expr) => {
                ((self.0 >> $n) & ((1 << $m) - 1)) as u8
            }
        }

        macro_rules! from    {() => {Square::from(shr!(4, 6))}}
        macro_rules! to      {() => {Square::from(shr!(10, 6))}}
        macro_rules! capture {() => {Piece::from(shr!(16, 4))}}
        macro_rules! promote {() => {Piece::from(shr!(20, 4))}}
        macro_rules! color   {() => {Color::from(shr!(4, 1))}}

        match shr!(0, 4) {
            0 => Move::None,
            1 => Move::Quiet {
                from: from!(),
                to: to!(),
            },
            2 => Move::Capture {
                from: from!(),
                to: to!(),
                capture: capture!(),
            },
            3 => Move::Promote {
                from: from!(),
                to: to!(),
                promote: promote!(),
            },
            4 => Move::PromoteCapture {
                from: from!(),
                to: to!(),
                capture: capture!(),
                promote: promote!(),
            },
            5 => Move::EnPassant {
                from: from!(),
                to: to!(),
            },
            6 => Move::DoublePush {
                from: from!(),
                to: to!(),
            },
            7 => Move::KingCastle {
                color: color!(),
            },
            8 => Move::QueenCastle {
                color: color!(),
            },
            _ => unreachable!(),
        }
    }
}