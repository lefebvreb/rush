use std::mem::transmute;

use crate::bitboard::BitBoard;
use crate::bits::SHIFTS;

/// Represent a Square of the board
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
    pub fn x(self) -> u32 {
        (self as u32) & 0x7
    }

    /// Return the y coodinate of that square
    #[inline(always)]
    pub fn y(self) -> u32 {
        (self as u32).wrapping_shr(3)
    }
}

impl From<u32> for Square {
    /// Undefined behaviour if i > 63
    #[inline(always)]
    fn from(i: u32) -> Square {
        unsafe {transmute(i as u8)}
    }
}

impl From<(u32, u32)> for Square {
    // Undefined behaviour if xy.0 + 8*xy.1 > 63
    #[inline(always)]
    fn from(xy: (u32, u32)) -> Square {
        Square::from(xy.0 + 8*xy.1)
    }
}

impl Into<BitBoard> for Square {
    #[inline(always)]
    fn into(self) -> BitBoard {
        BitBoard(SHIFTS[self as usize])
    }
}