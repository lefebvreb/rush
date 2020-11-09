use std::fmt;
use std::ops::{Add, BitAnd, BitOr, BitXor, Mul, Not, Shl, Shr};

use crate::square::Square;

/// Represent a 64 bits BitBoard
#[derive(Copy, Clone, PartialEq)]
pub struct BitBoard(pub u64);

//#################################################################################################
//
//                                     Implementation
//
//#################################################################################################

impl BitBoard {
    pub const FILE_A: BitBoard = BitBoard(0x0101010101010101);
    pub const FILE_B: BitBoard = BitBoard(0x0202020202020202);
    pub const FILE_C: BitBoard = BitBoard(0x0404040404040404);
    pub const FILE_D: BitBoard = BitBoard(0x0808080808080808);
    pub const FILE_E: BitBoard = BitBoard(0x1010101010101010);
    pub const FILE_F: BitBoard = BitBoard(0x2020202020202020);
    pub const FILE_G: BitBoard = BitBoard(0x4040404040404040);
    pub const FILE_H: BitBoard = BitBoard(0x8080808080808080);

    pub const RANK_1: BitBoard = BitBoard(0x00000000000000FF);
    pub const RANK_2: BitBoard = BitBoard(0x000000000000FF00);
    pub const RANK_3: BitBoard = BitBoard(0x0000000000FF0000);
    pub const RANK_4: BitBoard = BitBoard(0x00000000FF000000);
    pub const RANK_5: BitBoard = BitBoard(0x000000FF00000000);
    pub const RANK_6: BitBoard = BitBoard(0x0000FF0000000000);
    pub const RANK_7: BitBoard = BitBoard(0x00FF000000000000);
    pub const RANK_8: BitBoard = BitBoard(0xFF00000000000000);

    /// Return true if the BitBoard `self` is empty
    #[inline]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Return an iterator over the bits of the BitBoard `self`
    #[inline]
    pub fn iter_squares(mut self) -> impl Iterator<Item = Square> {
        (0..self.0.count_ones()).map(move |_| {
            let lsb = self.0.trailing_zeros();
            self.0 ^= 1u64 << lsb;
            Square::from(lsb)
        })
    }

    /// Return an iterator over the BitBoards of each bits of the BitBoard `self`
    #[inline]
    pub fn iter_bitboards(mut self) -> impl Iterator<Item = BitBoard> {
        (0..self.0.count_ones()).map(move |_| {
            let lsb = 1u64 << self.0.trailing_zeros();
            self.0 ^= lsb;
            BitBoard(lsb)
        })
    }
}

impl Into<usize> for BitBoard {
    #[inline]
    fn into(self) -> usize {
        self.0 as usize
    }
}

/// A convenient macro to construct a BitBoard from a collection of Squares
#[macro_export]
macro_rules! squares {
    ($($sq: expr),*) => {
        BitBoard(0) $(| $sq.into())*
    };
}

//#################################################################################################
//
//                                          Format
//
//#################################################################################################

impl fmt::Debug for BitBoard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BitBoard(0x{:X})", self.0)
    }
}

impl fmt::Display for BitBoard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut bits = self.0.reverse_bits();

        writeln!(f, "┌BitBoard┐").unwrap();
        for _ in 0..8 {
            let rank = bits & 0xFF;
            bits = bits.wrapping_shr(8);
            writeln!(f, "│{:08b}│", rank).unwrap();
        }
        writeln!(f, "└────────┘")
    }
}

//#################################################################################################
//
//                                         Operators
//
//#################################################################################################

impl Add<BitBoard> for BitBoard {
    type Output = BitBoard;

    #[inline]
    fn add(self, rhs: BitBoard) -> BitBoard {
        BitBoard(self.0.wrapping_add(rhs.0))
    }
}

impl Mul<BitBoard> for BitBoard {
    type Output = BitBoard;

    #[inline]
    fn mul(self, rhs: BitBoard) -> BitBoard {
        BitBoard(self.0.wrapping_mul(rhs.0))
    }
}

impl Not for BitBoard {
    type Output = BitBoard;

    #[inline]
    fn not(self) -> BitBoard {
        BitBoard(self.0.not())
    }
}

impl BitAnd<BitBoard> for BitBoard {
    type Output = BitBoard;

    #[inline]
    fn bitand(self, rhs: BitBoard) -> BitBoard {
        BitBoard(self.0.bitand(rhs.0))
    }
}

impl BitOr<BitBoard> for BitBoard {
    type Output = BitBoard;

    #[inline]
    fn bitor(self, rhs: BitBoard) -> BitBoard {
        BitBoard(self.0.bitor(rhs.0))
    }
}

impl BitXor<BitBoard> for BitBoard {
    type Output = BitBoard;

    #[inline]
    fn bitxor(self, rhs: BitBoard) -> BitBoard {
        BitBoard(self.0.bitxor(rhs.0))
    }
}

impl Shl<u32> for BitBoard {
    type Output = BitBoard;

    #[inline]
    fn shl(self, rhs: u32) -> BitBoard {
        BitBoard(self.0.wrapping_shl(rhs))
    }
}

impl Shr<u32> for BitBoard {
    type Output = BitBoard;

    #[inline]
    fn shr(self, rhs: u32) -> BitBoard {
        BitBoard(self.0.wrapping_shr(rhs))
    }
}
