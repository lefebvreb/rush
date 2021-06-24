use std::fmt;

use crate::color::Color;
use crate::piece::Piece;
use crate::square::Square;

//#################################################################################################
//
//                                           enum MoveType
//
//#################################################################################################

/// An enum to represent the type of a move.
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum MoveType {
    Quiet          = 0,
    Capture        = 1,
    Promote        = 2,
    PromoteCapture = 3,
    EnPassant      = 4,
    DoublePush     = 5,
    KingCastle     = 6,
    QueenCastle    = 7,
}

// ================================ impl

impl MoveType {
    // The list of all type moves.
    const MOVE_TYPES: [MoveType; 8] = [
        MoveType::Quiet, MoveType::Capture, MoveType::Promote, MoveType::PromoteCapture,
        MoveType::EnPassant, MoveType::DoublePush, MoveType::KingCastle, MoveType::QueenCastle,
    ];

    // Generates a base move with self as type and the two given squares.
    #[inline(always)]
    const fn base(self, from: Square, to: Square) -> u32 {
        self as u32 | (from as u32) << 3 | (to as u32) << 9
    }
}

//#################################################################################################
//
//                                            struct Move
//
//#################################################################################################

/// A move, encoded in a compact 32 bits representation:
/// `mmmffffffttttttcccppp` where `m` is the type of the move, 
/// `f` is the from square, `t` is the to square, `c` is the capture piece
/// and `p` is the promoted piece.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Move(u32);

// ================================ pub impl

impl Move {
    /// Creates a quiet move.
    #[inline(always)]
    pub const fn quiet(from: Square, to: Square) -> Move {
        Move(MoveType::Quiet.base(from, to))
    }

    /// Creates a standard capture move.
    #[inline(always)]
    pub const fn capture(from: Square, to: Square, capture: Piece) -> Move {
        Move(MoveType::Capture.base(from, to) | (capture as u32) << 15)
    }

    /// Creates a promotion move.
    #[inline(always)]
    pub const fn promote(from: Square, to: Square, promote: Piece) -> Move {
        Move(MoveType::Promote.base(from, to) | (promote as u32) << 18)
    }

    /// Creates a promotion and capture move.
    #[inline(always)]
    pub const fn promote_capture(from: Square, to: Square, capture: Piece, promote: Piece) -> Move {
        Move(MoveType::PromoteCapture.base(from, to) | (capture as u32) << 15 | (promote as u32) << 18)
    }

    /// Creates an en passant move.
    #[inline(always)]
    pub const fn en_passant(from: Square, to: Square) -> Move {
        Move(MoveType::EnPassant.base(from, to))
    }

    /// Creates a double push move.
    #[inline(always)]
    pub const fn double_push(from: Square, to: Square) -> Move {
        Move(MoveType::DoublePush.base(from, to))
    }

    /// Crates a king castle (o-o) move.
    #[inline(always)]
    pub const fn king_castle(color: Color) -> Move {
        Move(match color {
            Color::White => MoveType::KingCastle.base(Square::E1, Square::G1),
            Color::Black => MoveType::KingCastle.base(Square::E8, Square::G8),
        })
    }

    /// Crates a queen castle (o-o-o) move.
    #[inline(always)]
    pub const fn queen_castle(color: Color) -> Move {
        Move(match color {
            Color::White => MoveType::QueenCastle.base(Square::E1, Square::C1),
            Color::Black => MoveType::QueenCastle.base(Square::E8, Square::C8),
        })
    }

    /// Returns the type of the move.
    #[inline(always)]
    pub const fn get_type(self) -> MoveType {
        MoveType::MOVE_TYPES[(self.0 & 0x7) as usize]
    }

    /// Returns the from square of the move.
    #[inline(always)]
    pub const fn from(self) -> Square {
        Square::SQUARES[(self.0 >> 3 & 0x3F) as usize]
    }

    /// Returns the to square of the move.
    #[inline(always)]
    pub const fn to(self) -> Square {
        Square::SQUARES[(self.0 >> 9 & 0x3F) as usize]
    }

    /// Returns the capture piece of the move.
    #[inline(always)]
    pub const fn get_capture(self) -> Piece {
        Piece::PIECES[(self.0 >> 15 & 0x7) as usize]
    }

    /// Returns the promote piece of the move.
    #[inline(always)]
    pub const fn get_promote(self) -> Piece {
        Piece::PIECES[(self.0 >> 18 & 0x7) as usize]
    }
}

// ================================ traits impl

impl fmt::Display for Move {
    // Displays a move using pure algebraic coordinate notation.
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        if matches!(self.get_type(), MoveType::Promote | MoveType::PromoteCapture) {
            write!(fmt, "{}{}{}", self.from(), self.to(), self.get_promote())
        } else {
            write!(fmt, "{}{}", self.from(), self.to())
        }
    }
}

//#################################################################################################
//
//                                     struct MoveList
//
//#################################################################################################

/// A type to represent a list of moves, containing up to MoveList::SIZE elements.
/// Every operations are o(1).
pub struct MoveList {
    size: usize,
    list: [Move; MoveList::SIZE],
}

// ================================ pub impl

impl MoveList {
    /// The maximal size of a move list.
    pub const SIZE: usize = 256;

    /// Pushes an element to the top of the move.
    #[inline(always)]
    pub fn push(&mut self, mv: Move) {
        self.list[self.size] = mv;
        self.size += 1;
    }

    /// Gets an element from the list.
    #[inline(always)]
    pub fn get(&self, i: usize) -> Move {
        self.list[i]
    }

    /// Clears the list.
    #[inline(always)]
    pub fn clear(&mut self) {
        self.size = 0
    }
}