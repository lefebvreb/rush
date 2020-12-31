use std::fmt;

use crate::color::Color;
use crate::piece::Piece;
use crate::square::Square;

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
    pub fn is_none(&self) -> bool {
        match self {
            Move::None => true,
            _ => false,
        }
    }

     /// Return true if the move is not none
     #[inline(always)]
     pub fn is_some(&self) -> bool {
         match self {
             Move::None => false,
             _ => true,
         }
     }
}

impl fmt::Display for Move {
    // Display a move using pure algebraic coordinate notation
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let promote = match self {
            Move::Promote {promote, ..} |
            Move::PromoteCapture {promote, ..} => promote.to_string(),
            _ => "".to_owned(),
        };

        write!(fmt, "{}{}{}", self.from(), self.to(), promote)
    }
}