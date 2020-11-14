use std::mem::transmute;

use crate::bitboard::BitBoard;
use crate::bits::SHIFTS;

/// Represent a Square of the board
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Debug)]
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
    #[inline]
    fn from(i: u32) -> Square {
        unsafe {transmute(i as u8)}
    }
}

impl From<(u32, u32)> for Square {
    // Undefined behaviour if xy.0 + 8*xy.1 > 63
    #[inline]
    fn from(xy: (u32, u32)) -> Square {
        Square::from(xy.0 + 8*xy.1)
    }
}

impl Into<BitBoard> for Square {
    #[inline]
    fn into(self) -> BitBoard {
        BitBoard(SHIFTS[self as usize])
    }
}