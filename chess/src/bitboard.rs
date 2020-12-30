use std::fmt;

use crate::color::Color;
use crate::square::Square;

/// Represent a 64 bits BitBoard
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq)]
pub struct BitBoard(pub u64);

//#################################################################################################
//
//                                     Implementation
//
//#################################################################################################

impl BitBoard {
    /// An empty BitBoard
    pub const EMPTY: BitBoard = BitBoard(0);

    /// A full BitBoard
    pub const FULL: BitBoard = BitBoard(0xFFFFFFFFFFFFFFFF);

    /// Return true if and only if the BitBoard `self` is empty
    #[inline(always)]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Return true if and only if the BitBoard `self` is not empty
    #[inline(always)]
    pub const fn is_not_empty(self) -> bool {
        self.0 != 0
    }

    /// Return an iterator over the bits of the BitBoard `self`
    #[inline(always)]
    pub fn iter_squares(mut self) -> impl Iterator<Item = Square> {
        (0..self.0.count_ones()).map(move |_| {
            let lsb = self.0.trailing_zeros();
            self &= self - BitBoard(1);
            Square::from(lsb as u8)
        })
    }

    /// Return an iterator over the BitBoards of each bits of the BitBoard `self`
    #[inline(always)]
    pub fn iter_bitboards(mut self) -> impl Iterator<Item = BitBoard> {
        (0..self.0.count_ones()).map(move |_| {
            let old = self;
            self &= self - BitBoard(1);
            old ^ self
        })
    }
    
    /// Count the bits of `self` that are 1
    #[inline(always)]
    pub fn count_bits(self) -> u8 {
        self.0.count_ones() as u8
    }

    // Return the first square of the bitboard
    #[inline(always)]
    pub(crate) fn as_square_unchecked(self) -> Square {
        Square::from(self.0.trailing_zeros() as u8)
    }

    /// Return true if that bitboard is on the last rank
    #[inline(always)]
    pub fn is_last_rank(self, color: Color) -> bool {
        self.0 & match color {
            Color::White => 0xFF00000000000000,
            Color::Black => 0xFF,
        } == self.0
    }

    /// Return true if that bitboard contains sq
    #[inline(always)]
    pub fn contains(self, sq: Square) -> bool {
        (self & sq.into()).0 != 0
    }
}

impl Into<usize> for BitBoard {
    #[inline(always)]
    fn into(self) -> usize {
        self.0 as usize
    }
}

/// A convenient macro to construct a BitBoard from a collection of Squares
macro_rules! squares {
    ($($sq: expr),*) => {
        BitBoard::EMPTY $(| $sq.into())*
    };
}

//#################################################################################################
//
//                                          Format
//
//#################################################################################################

impl fmt::Debug for BitBoard {
    #[cold]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BitBoard(0x{:X})", self.0)
    }
}

impl fmt::Display for BitBoard {
    #[cold]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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

impl std::ops::Add<BitBoard> for BitBoard {
    type Output = BitBoard;

    #[inline(always)]
    fn add(self, rhs: BitBoard) -> BitBoard {
        BitBoard(self.0.wrapping_add(rhs.0))
    }
}

impl std::ops::AddAssign<BitBoard> for BitBoard {
    #[inline(always)]
    fn add_assign(&mut self, rhs: BitBoard) {
        self.0.add_assign(rhs.0)
    }
}

impl std::ops::Sub<BitBoard> for BitBoard {
    type Output = BitBoard;

    #[inline(always)]
    fn sub(self, rhs: BitBoard) -> BitBoard {
        BitBoard(self.0.wrapping_sub(rhs.0))
    }
}

impl std::ops::SubAssign<BitBoard> for BitBoard {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: BitBoard) {
        self.0.sub_assign(rhs.0)
    }
}

impl std::ops::Mul<BitBoard> for BitBoard {
    type Output = BitBoard;

    #[inline(always)]
    fn mul(self, rhs: BitBoard) -> BitBoard {
        BitBoard(self.0.wrapping_mul(rhs.0))
    }
}

impl std::ops::MulAssign<BitBoard> for BitBoard {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: BitBoard) {
        self.0.mul_assign(rhs.0)
    }
}

impl std::ops::Not for BitBoard {
    type Output = BitBoard;

    #[inline(always)]
    fn not(self) -> BitBoard {
        BitBoard(self.0.not())
    }
}

impl std::ops::Neg for BitBoard {
    type Output = BitBoard;

    #[inline(always)]
    fn neg(self) -> BitBoard {
        BitBoard(self.0.wrapping_neg())
    }
}

impl std::ops::BitAnd<BitBoard> for BitBoard {
    type Output = BitBoard;

    #[inline(always)]
    fn bitand(self, rhs: BitBoard) -> BitBoard {
        BitBoard(self.0.bitand(rhs.0))
    }
}

impl std::ops::BitAndAssign<BitBoard> for BitBoard {
    #[inline(always)]
    fn bitand_assign(&mut self, rhs: BitBoard) {
        self.0.bitand_assign(rhs.0)
    }
}

impl std::ops::BitOr<BitBoard> for BitBoard {
    type Output = BitBoard;

    #[inline(always)]
    fn bitor(self, rhs: BitBoard) -> BitBoard {
        BitBoard(self.0.bitor(rhs.0))
    }
}

impl std::ops::BitOrAssign<BitBoard> for BitBoard {
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: BitBoard) {
        self.0.bitor_assign(rhs.0)
    }
}

impl std::ops::BitXor<BitBoard> for BitBoard {
    type Output = BitBoard;

    #[inline(always)]
    fn bitxor(self, rhs: BitBoard) -> BitBoard {
        BitBoard(self.0.bitxor(rhs.0))
    }
}

impl std::ops::BitXorAssign<BitBoard> for BitBoard {
    #[inline(always)]
    fn bitxor_assign(&mut self, rhs: BitBoard) {
        self.0.bitxor_assign(rhs.0)
    }
}

impl std::ops::Shl<u32> for BitBoard {
    type Output = BitBoard;

    #[inline(always)]
    fn shl(self, rhs: u32) -> BitBoard {
        BitBoard(self.0.wrapping_shl(rhs))
    }
}

impl std::ops::ShlAssign<BitBoard> for BitBoard {
    #[inline(always)]
    fn shl_assign(&mut self, rhs: BitBoard) {
        self.0.shl_assign(rhs.0)
    }
}

impl std::ops::Shr<u32> for BitBoard {
    type Output = BitBoard;

    #[inline(always)]
    fn shr(self, rhs: u32) -> BitBoard {
        BitBoard(self.0.wrapping_shr(rhs))
    }
}

impl std::ops::ShrAssign<BitBoard> for BitBoard {
    #[inline(always)]
    fn shr_assign(&mut self, rhs: BitBoard) {
        self.0.shr_assign(rhs.0)
    }
}