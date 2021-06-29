use std::fmt;

use crate::color::Color;
use crate::piece::Piece;
use crate::square::Square;

// Create the base for a move, with
#[inline]
const fn base(flags: u32, from: Square, to: Square) -> u32 {
    flags | (from as u32) << 5 | (to as u32) << 11
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
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Move(u32);

// ================================ pub impl

impl Move {
    /// Creates a quiet move.
    #[inline]
    pub const fn quiet(from: Square, to: Square) -> Move {
        Move(base(Move::QUIET, from, to))
    }

    /// Creates a standard capture move.
    #[inline]
    pub const fn capture(from: Square, to: Square, capture: Piece) -> Move {
        Move(base(Move::CAPTURE, from, to) | (capture as u32) << 17)
    }

    /// Creates a promotion move.
    #[inline]
    pub const fn promote(from: Square, to: Square, promote: Piece) -> Move {
        Move(base(Move::PROMOTE, from, to) | (promote as u32) << 20)
    }

    /// Creates a promotion and capture move.
    #[inline]
    pub const fn promote_capture(from: Square, to: Square, capture: Piece, promote: Piece) -> Move {
        Move(base(Move::CAPTURE | Move::PROMOTE, from, to) | (capture as u32) << 17 | (promote as u32) << 20)
    }

    /// Creates an en passant move.
    #[inline]
    pub const fn en_passant(from: Square, to: Square) -> Move {
        Move(base(Move::EN_PASSANT, from, to))
    }

    /// Creates a double push move.
    #[inline]
    pub const fn double_push(from: Square, to: Square) -> Move {
        Move(base(Move::DOUBLE_PUSH, from, to))
    }

    /// Crates a king castle (OO) move.
    #[inline]
    pub const fn king_castle(color: Color) -> Move {
        Move(match color {
            Color::White => base(Move::CASTLE, Square::E1, Square::G1),
            Color::Black => base(Move::CASTLE, Square::E8, Square::G8),
        })
    }

    /// Crates a queen castle (OOO) move.
    #[inline]
    pub const fn queen_castle(color: Color) -> Move {
        Move(match color {
            Color::White => base(Move::CASTLE, Square::E1, Square::C1),
            Color::Black => base(Move::CASTLE, Square::E8, Square::C8),
        })
    }

    #[inline]
    pub const fn is_quiet(self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub const fn is_capture(self) -> bool {
        self.0 & Move::CAPTURE != 0
    }

    #[inline]
    pub const fn is_promote(self) -> bool {
        self.0 & Move::PROMOTE != 0
    }

    #[inline]
    pub const fn is_castle(self) -> bool {
        self.0 & Move::CASTLE != 0
    }

    #[inline]
    pub const fn is_en_passant(self) -> bool {
        self.0 & Move::EN_PASSANT != 0
    }

    #[inline]
    pub const fn is_double_push(self) -> bool {
        self.0 & Move::DOUBLE_PUSH != 0
    }

    /// Returns the from square of the move.
    #[inline]
    pub fn from(self) -> Square {
        Square::from((self.0 >> 5 & 0x3F) as i8)
    }

    /// Returns the to square of the move.
    #[inline]
    pub fn to(self) -> Square {
        Square::from((self.0 >> 11 & 0x3F) as i8)
    }

    #[inline]
    pub fn squares(self) -> (Square, Square) {
        (self.from(), self.to())
    }

    /// Returns the capture piece of the move.
    #[inline]
    pub fn get_capture(self) -> Piece {
        Piece::from((self.0 >> 17 & 0x7) as u8)
    }

    /// Returns the promote piece of the move.
    #[inline]
    pub fn get_promote(self) -> Piece {
        Piece::from((self.0 >> 20 & 0x7) as u8)
    }
}

// ================================ impl

impl Move {
    const QUIET: u32 = 0b00000;
    const CAPTURE: u32 = 0b00001;
    const PROMOTE: u32 = 0b00010;
    const CASTLE: u32 = 0b00100;
    const EN_PASSANT: u32 = 0b01000;
    const DOUBLE_PUSH: u32 = 0b10000;    
}

// ================================ traits impl

impl fmt::Display for Move {
    // Displays a move using pure algebraic coordinate notation.
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        if self.is_promote() {
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
    #[inline]
    pub fn push(&mut self, mv: Move) {
        self.list[self.size] = mv;
        self.size += 1;
    }

    /// Gets an element from the list.
    #[inline]
    pub fn get(&self, i: usize) -> Move {
        self.list[i]
    }

    /// Clears the list.
    #[inline]
    pub fn clear(&mut self) {
        self.size = 0
    }
}