use std::fmt;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;

use crate::piece::Piece;
use crate::prelude::Color;
use crate::square::Square;

/// Create the base for a move, with the given flags, from and to squares.
#[inline]
const fn base(flags: u32, from: Square, to: Square) -> u32 {
    flags | (from as u32) << 5 | (to as u32) << 11
}

//#################################################################################################
//
//                                            struct Move
//
//#################################################################################################

/// A move, encoded in a compact 32 bits representation. 
/// In big endian, the encoding is done like that:
/// pppcccttttttffffffmmmmm, where m is the type of the move, 
/// f is the from square, t is the to square, c is the captured piece
/// and p is the promote piece.
#[repr(transparent)]
#[derive(Clone, Copy, Eq, PartialEq)]
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
    pub const fn castle(from: Square, to: Square) -> Move {
        Move(base(Move::CASTLE, from, to))
    }

    /// Returns true if the move is quiet.
    #[inline]
    pub const fn is_quiet(self) -> bool {
        self.0 & 0b11111 == 0
    }

    /// Returns true if the move is a capture.
    #[inline]
    pub const fn is_capture(self) -> bool {
        self.0 & Move::CAPTURE != 0
    }

    /// Returns true if the move is a promotion.
    #[inline]
    pub const fn is_promote(self) -> bool {
        self.0 & Move::PROMOTE != 0
    }

    /// Returns true if the move is castling.
    #[inline]
    pub const fn is_castle(self) -> bool {
        self.0 & Move::CASTLE != 0
    }

    /// Returns true if the move is en passant.
    #[inline]
    pub const fn is_en_passant(self) -> bool {
        self.0 & Move::EN_PASSANT != 0
    }

    /// Returns true if the move is a double pawn push.
    #[inline]
    pub const fn is_double_push(self) -> bool {
        self.0 & Move::DOUBLE_PUSH != 0
    }

    /// Returns the from square of the move.
    #[inline]
    pub fn from(self) -> Square {
        // SAFE: 0 <= argument < 64
        unsafe {
            Square::from_unchecked((self.0 >> 5 & 0x3F) as i8)
        }
    }

    /// Returns the to square of the move.
    #[inline]
    pub fn to(self) -> Square {
        // SAFE: 0 <= argument < 64
        unsafe {
            Square::from_unchecked((self.0 >> 11 & 0x3F) as i8)
        }
    }

    #[inline]
    pub fn squares(self) -> (Square, Square) {
        (self.from(), self.to())
    }

    /// Returns the capture piece of the move.
    #[inline]
    pub fn get_capture(self) -> Piece {
        // SAFE: 0 <= argument < 6
        unsafe {
            Piece::from_unchecked((self.0 >> 17 & 0x7) as u8)
        }
    }

    /// Returns the promote piece of the move.
    #[inline]
    pub fn get_promote(self) -> Piece {
        // SAFE: 0 <= argument < 6
        unsafe {
            Piece::from_unchecked((self.0 >> 20 & 0x7) as u8)
        }
    }

    /// Returns the raw value of the move.
    #[inline]
    pub fn get_raw(self) -> u32 {
        self.0
    }
}

// ================================ impl

impl Move {
    // Move type masks. 
    const QUIET       : u32 = 0b00000;
    const CAPTURE     : u32 = 0b00001;
    const PROMOTE     : u32 = 0b00010;
    const CASTLE      : u32 = 0b00100;
    const EN_PASSANT  : u32 = 0b01000;
    const DOUBLE_PUSH : u32 = 0b10000;    
}

// ================================ traits impl

impl fmt::Display for Move {
    /// Displays a move using pure algebraic coordinate notation.
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        if self.is_promote() {
            write!(fmt, "{}{}{}", self.from(), self.to(), self.get_promote().as_char(Color::Black))
        } else {
            write!(fmt, "{}{}", self.from(), self.to())
        }
    }
}

impl fmt::Debug for Move {
    /// Displays useful debugging informations about a move.
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("Move")
            .field("from", &self.from())
            .field("to", &self.to())
            .field("is_quiet", &self.is_quiet())
            .field("is_capture", &self.is_capture())
            .field("is_promote", &self.is_promote())
            .field("is_castle", &self.is_castle())
            .field("is_en_passant", &self.is_en_passant())
            .field("is_double_push", &self.is_double_push())
            .finish()
    }
}

//#################################################################################################
//
//                                    struct AtomicMove
//
//#################################################################################################

/// An atomic type containing a move, providing load and store methods.
#[repr(transparent)]
#[derive(Default, Debug)]
pub struct AtomicMove(AtomicU32);

// ================================ pub impl

impl AtomicMove {
    /// Atomically resets the move contained in the atomic.
    #[inline]
    pub fn reset(&self) {
        self.0.store(0, Ordering::Release);
    }

    /// Loads the move stored in the atomic.
    #[inline]
    pub fn load(&self) -> Option<Move> {
        // Since a valid move's raw value is never 0, it is acceptable 
        // to use the 0 value as representing None.
        match self.0.load(Ordering::Acquire) {
            0 => None,
            raw => Some(Move(raw)),
        }
    }

    /// Stores the move into the atomic.
    #[inline]
    pub fn store(&self, mv: Move) {
        self.0.store(mv.0, Ordering::Release);
    }
}