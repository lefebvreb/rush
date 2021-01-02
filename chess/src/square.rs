use std::fmt;
use std::mem::transmute;
use std::str::FromStr;

use crate::bitboard::BitBoard;
use crate::bits::SHIFTS;
use crate::color::Color;
use crate::errors::ParseFenError;

/// Represent a Square of the board.
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Square {
    A1 =  0, B1 =  1, C1 =  2, D1 =  3, E1 =  4, F1 =  5, G1 =  6, H1 =  7,
    A2 =  8, B2 =  9, C2 = 10, D2 = 11, E2 = 12, F2 = 13, G2 = 14, H2 = 15, 
    A3 = 16, B3 = 17, C3 = 18, D3 = 19, E3 = 20, F3 = 21, G3 = 22, H3 = 23,
    A4 = 24, B4 = 25, C4 = 26, D4 = 27, E4 = 28, F4 = 29, G4 = 30, H4 = 31, 
    A5 = 32, B5 = 33, C5 = 34, D5 = 35, E5 = 36, F5 = 37, G5 = 38, H5 = 39, 
    A6 = 40, B6 = 41, C6 = 42, D6 = 43, E6 = 44, F6 = 45, G6 = 46, H6 = 47, 
    A7 = 48, B7 = 49, C7 = 50, D7 = 51, E7 = 52, F7 = 53, G7 = 54, H7 = 55, 
    A8 = 56, B8 = 57, C8 = 58, D8 = 59, E8 = 60, F8 = 61, G8 = 62, H8 = 63,
}

impl Square {
    /// An array containing all squares in order: ranks first then files, starting from A1.
    pub const SQUARES: [Square; 64] = [
        Square::A1, Square::B1, Square::C1, Square::D1, Square::E1, Square::F1, Square::G1, Square::H1,
        Square::A2, Square::B2, Square::C2, Square::D2, Square::E2, Square::F2, Square::G2, Square::H2,
        Square::A3, Square::B3, Square::C3, Square::D3, Square::E3, Square::F3, Square::G3, Square::H3,
        Square::A4, Square::B4, Square::C4, Square::D4, Square::E4, Square::F4, Square::G4, Square::H4,
        Square::A5, Square::B5, Square::C5, Square::D5, Square::E5, Square::F5, Square::G5, Square::H5,
        Square::A6, Square::B6, Square::C6, Square::D6, Square::E6, Square::F6, Square::G6, Square::H6,
        Square::A7, Square::B7, Square::C7, Square::D7, Square::E7, Square::F7, Square::G7, Square::H7,
        Square::A8, Square::B8, Square::C8, Square::D8, Square::E8, Square::F8, Square::G8, Square::H8,
    ];

    /// Return the x coodinate of that square
    #[inline(always)]
    pub fn x(self) -> u8 {
        (self as u8) & 0x7
    }

    /// Return the y coodinate of that square
    #[inline(always)]
    pub fn y(self) -> u8 {
        (self as u8).wrapping_shr(3)
    }

    /// Return true if that square is last rank
    #[inline(always)]
    pub fn is_last_rank(self, color: Color) -> bool {
        self.y() == match color {
            Color::White => 7,
            Color::Black => 0,
        }
    }

    // Return the square immediately left of that square 
    // (from white's point of view)
    #[inline(always)]
    pub(crate) fn get_left_unchecked(self) -> Square {
        Square::from((self.x() - 1, self.y()))
    }

    // Return the square immediately right of that square 
    // (from white's point of view)
    #[inline(always)]
    pub(crate) fn get_right_unchecked(self) -> Square {
        Square::from((self.x() + 1, self.y()))
    }

    #[inline(always)]
    pub(crate) fn get_mid(self, other: Square) -> Square {
        Square::from((self.x(), (self.y() + other.y()) / 2))
    }
}

impl fmt::Display for Square {
    // Give the square's pure algebraic coordinates notation
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        const FILES: [char; 8] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
        const RANKS: [char; 8] = ['1', '2', '3', '4', '5', '6', '7', '8'];

        write!(f, "{}{}", FILES[self.x() as usize], RANKS[self.y() as usize])
    }
}

impl Into<BitBoard> for Square {
    #[inline(always)]
    fn into(self) -> BitBoard {
        BitBoard(SHIFTS[self as usize])
    }
}

impl From<u8> for Square {
    // Undefined behaviour if i > 63
    #[inline(always)]
    fn from(i: u8) -> Square {
        unsafe {transmute(i as u8)}
    }
}

impl From<(u8, u8)> for Square {
    // Undefined behaviour if xy.0 + 8*xy.1 > 63
    #[inline(always)]
    fn from(xy: (u8, u8)) -> Square {
        Square::from(xy.0 + 8*xy.1)
    }
}

impl FromStr for Square {
    type Err = ParseFenError;

    // Try to construct a square from a pure algebraic coordinates notation
    fn from_str(s: &str) -> Result<Square, ParseFenError> {
        if s.len() == 2 {
            let mut chars = s.chars();

            let file = chars.next().unwrap();
            let rank = chars.next().unwrap();

            Ok(Square::from((
                match file {
                    'a'..='h' => file as u8 - 'a' as u8,
                    _ => return Err(ParseFenError::new("first character of a square should be a letter between a and h")),
                },
                match rank {
                    '1'..='8' => rank as u8 - '1' as u8,
                    _ => return Err(ParseFenError::new("second character of a square should be a digit between 1 and 8")),
                },
            )))
        } else {
            Err(ParseFenError::new("a square should be exactly 2 characters long"))
        }
    }
}