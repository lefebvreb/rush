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
    KingCastle,
    QueenCastle,
}

impl Move {
    /// Return the square from which the move is performed
    #[cold]
    pub fn from(&self, color: Color) -> Square {
        match self {
            Move::Quiet {from, ..} | 
            Move::Capture {from, ..} | 
            Move::Promote {from, ..} | 
            Move::PromoteCapture {from, ..} | 
            Move::EnPassant {from, ..} | 
            Move::DoublePush {from, ..} => *from,
            Move::KingCastle | 
            Move::QueenCastle => if color == Color::White {
                Square::E1
            } else {
                Square::E8
            }
            _ => unreachable!(),
        }
    }

    /// Return the square to which the move is performed
    #[cold]
    pub fn to(&self, color: Color) -> Square {
        match self {
            Move::Quiet {to, ..} | 
            Move::Capture {to, ..} | 
            Move::Promote {to, ..} | 
            Move::PromoteCapture {to, ..} | 
            Move::EnPassant {to, ..} | 
            Move::DoublePush {to, ..} => *to,
            Move::KingCastle => if color == Color::White {
                Square::G1
            } else {
                Square::G8
            }
            Move::QueenCastle => if color == Color::White {
                Square::C1
            } else {
                Square::C8
            }
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
}