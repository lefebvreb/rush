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
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Move {
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
        }
    }

    /// Return true if the move is a capture
    #[inline(always)]
    pub fn is_capture(self) -> bool {
        matches!(self, Move::Capture {..} | Move::PromoteCapture {..} | Move::EnPassant {..})
    }

    // Return true if the move is truly reversible
    #[inline(always)]
    pub fn is_truly_reversible(self, board: &Board) -> bool {
        matches!(self, Move::Quiet {from, ..} if board.get_piece_unchecked(from) != Piece::Pawn)
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
            Move::Quiet {from, to} => 
                0 | shl!(from, 3) | shl!(to, 9),
            Move::Capture {from, to, capture} => 
                1 | shl!(from, 3) | shl!(to, 9) | shl!(capture, 15),
            Move::Promote {from, to, promote} => 
                2 | shl!(from, 3) | shl!(to, 9) | shl!(promote, 19),
            Move::PromoteCapture {from, to, capture, promote} => 
                3 | shl!(from, 3) | shl!(to, 9) | shl!(capture, 15) | shl!(promote, 19),
            Move::EnPassant {from, to} => 
                4 | shl!(from, 3) | shl!(to, 9),
            Move::DoublePush {from, to} => 
                5 | shl!(from, 3) | shl!(to, 9),
            Move::KingCastle {color} => 
                6 | shl!(color, 3),
            Move::QueenCastle {color} =>
                7 | shl!(color, 3),
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

        macro_rules! from    {() => {Square::from(shr!(3, 6))}}
        macro_rules! to      {() => {Square::from(shr!(9, 6))}}
        macro_rules! capture {() => {Piece::from(shr!(15, 4))}}
        macro_rules! promote {() => {Piece::from(shr!(19, 4))}}
        macro_rules! color   {() => {Color::from(shr!(3, 1))}}

        match shr!(0, 3) {
            0 => Move::Quiet {
                from: from!(),
                to: to!(),
            },
            1 => Move::Capture {
                from: from!(),
                to: to!(),
                capture: capture!(),
            },
            2 => Move::Promote {
                from: from!(),
                to: to!(),
                promote: promote!(),
            },
            3 => Move::PromoteCapture {
                from: from!(),
                to: to!(),
                capture: capture!(),
                promote: promote!(),
            },
            4 => Move::EnPassant {
                from: from!(),
                to: to!(),
            },
            5 => Move::DoublePush {
                from: from!(),
                to: to!(),
            },
            6 => Move::KingCastle {
                color: color!(),
            },
            7 => Move::QueenCastle {
                color: color!(),
            },
            _ => unreachable!(),
        }
    }
}