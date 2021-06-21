use std::fmt;
use std::ops;

use crate::color::Color;
use crate::square::Square;

//#################################################################################################
//
//                                       static SHIFTS
//
//#################################################################################################

// An array whose ith element is 1 << i
static mut SHIFTS: [BitBoard; 64] = [BitBoard::EMPTY; 64];

// Initialize the SHIFTS static
pub(crate) unsafe fn init_shifts() {
    for i in 0..64 {
        SHIFTS[i] = BitBoard(1 << i);
    }
}

// Return  1 << i as a BitBoard
pub(crate) fn one_shl(i: u8) -> BitBoard {
    unsafe {
        SHIFTS[i as usize]
    }
}

//#################################################################################################
//
//                                       struct BitBoard
//
//#################################################################################################

/// Represent a 64 bits BitBoard
#[repr(transparent)]
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct BitBoard(pub u64);

// ================================ pub impl

impl BitBoard {
    /// An empty BitBoard
    pub const EMPTY: BitBoard = BitBoard(0);

    /// A full BitBoard
    pub const FULL: BitBoard = BitBoard(0xFFFFFFFFFFFFFFFF);

    // The ranks of the board
    pub const RANK_1: BitBoard = BitBoard(0xFF);
    pub const RANK_2: BitBoard = BitBoard(0xFF00);
    pub const RANK_3: BitBoard = BitBoard(0xFF0000);
    pub const RANK_4: BitBoard = BitBoard(0xFF000000);
    pub const RANK_5: BitBoard = BitBoard(0xFF00000000);
    pub const RANK_6: BitBoard = BitBoard(0xFF0000000000);
    pub const RANK_7: BitBoard = BitBoard(0xFF000000000000);
    pub const RANK_8: BitBoard = BitBoard(0xFF00000000000000);

    /// Return true if and only if the BitBoard `self` is empty
    #[inline]
    pub const fn empty(self) -> bool {
        self.0 == 0
    }

    /// Return true if and only if the BitBoard `self` is not empty
    #[inline]
    pub const fn not_empty(self) -> bool {
        self.0 != 0
    }

    /// Return an iterator over the bits of the BitBoard `self`
    #[inline]
    pub fn iter_squares(mut self) -> impl Iterator<Item = Square> {
        (0..self.0.count_ones()).map(move |_| {
            let lsb = self.0.trailing_zeros();
            self &= self - BitBoard(1);
            Square::from(lsb as u8)
        })
    }
    
    /// Count the bits of `self` that are 1
    #[inline]
    pub fn count(self) -> u8 {
        self.0.count_ones() as u8
    }

    /// Return true if that bitboard is on the last rank
    #[inline]
    pub fn last_rank(self, color: Color) -> bool {
        (self & match color {
            Color::White => BitBoard::RANK_8,
            Color::Black => BitBoard::RANK_1,
        }).0 == self.0
    }

    /// Return true if that bitboard contains sq
    #[inline]
    pub fn contains(self, sq: Square) -> bool {
        (self & sq.into()).0 != 0
    }
}

/// A convenient macro to construct a BitBoard from a collection of Squares
macro_rules! squares {
    ($($sq: expr),*) => {
        BitBoard::EMPTY $(| $sq.into())*
    };
}

// ================================ pub(crate) impl

impl BitBoard {
    // Return the first square of the bitboard, with no checks
    #[inline]
    pub(crate) fn as_square_unchecked(self) -> Square {
        Square::from(self.0.trailing_zeros() as u8)
    }

    // Perform a parallel bits extract (pext) using the intrinsic (fast)
    #[inline]
    #[cfg(target_feature = "bmi2")]
    pub(crate) fn pext(self, mask: BitBoard) -> BitBoard {
        BitBoard(unsafe {
            std::arch::x86_64::_pext_u64(self.0, mask.0)
        })
    }

    // Perform a parallel bits extract (pext) without the intrinsic (slow)
    #[inline]
    #[cfg(not(target_feature = "bmi2"))]
    pub(crate) fn pext(self, mut mask: BitBoard) -> BitBoard {
        compile_error!("pext intrinsic not found");
        let (mut i, mut res) = (0, 0);

        while mask.0 != 0 {
            let tmp = mask.0;
            mask.0 &= mask.0 - 1;
            if (mask.0 ^ tmp) & self.0 != 0 {
                res |= shift(i).0;
            }
            i += 1;
        }

        BitBoard(res)
    }

    // Perform a parallel bits deposit (pdep) using the intrinsic (fast)
    #[inline]
    #[cfg(target_feature = "bmi2")]
    pub(crate) fn pdep(self, mask: BitBoard) -> BitBoard {
        BitBoard(unsafe {
            std::arch::x86_64::_pdep_u64(self.0, mask.0)
        })
    }

    // Perform a parallel bits deposit (pdep) without the intrinsic (slow)
    #[inline]
    #[cfg(not(target_feature = "bmi2"))]
    pub(crate) fn pdep(self, mut mask: BitBoard) -> BitBoard {
        compile_error!("pdep intrinsic not found");
        let (mut i, mut res) = (0, 0);

        while mask.0 != 0 {
            let tmp = mask.0;
            mask.0 &= mask.0 - 1;
            if self.0 & shift(i).0 != 0 {
                res |= mask.0 ^ tmp;
            }
            i += 1;
        }

        BitBoard(res)
    }
}

// ================================ traits impl

impl fmt::Debug for BitBoard {
    // Print the bitboard in hex form for quick debugging
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BitBoard(0x{:X})", self.0)
    }
}

impl fmt::Display for BitBoard {
    // Pretty-print the bitboard for human eyes
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

impl ops::Add<BitBoard> for BitBoard {
    type Output = BitBoard;

    #[inline]
    fn add(self, rhs: BitBoard) -> BitBoard {
        BitBoard(self.0.wrapping_add(rhs.0))
    }
}

impl ops::AddAssign<BitBoard> for BitBoard {
    #[inline]
    fn add_assign(&mut self, rhs: BitBoard) {
        self.0.add_assign(rhs.0)
    }
}

impl ops::Sub<BitBoard> for BitBoard {
    type Output = BitBoard;

    #[inline]
    fn sub(self, rhs: BitBoard) -> BitBoard {
        BitBoard(self.0.wrapping_sub(rhs.0))
    }
}

impl ops::SubAssign<BitBoard> for BitBoard {
    #[inline]
    fn sub_assign(&mut self, rhs: BitBoard) {
        self.0.sub_assign(rhs.0)
    }
}

impl ops::Mul<BitBoard> for BitBoard {
    type Output = BitBoard;

    #[inline]
    fn mul(self, rhs: BitBoard) -> BitBoard {
        BitBoard(self.0.wrapping_mul(rhs.0))
    }
}

impl ops::MulAssign<BitBoard> for BitBoard {
    #[inline]
    fn mul_assign(&mut self, rhs: BitBoard) {
        self.0.mul_assign(rhs.0)
    }
}

impl ops::Not for BitBoard {
    type Output = BitBoard;

    #[inline]
    fn not(self) -> BitBoard {
        BitBoard(self.0.not())
    }
}

impl ops::Neg for BitBoard {
    type Output = BitBoard;

    #[inline]
    fn neg(self) -> BitBoard {
        BitBoard(self.0.wrapping_neg())
    }
}

impl ops::BitAnd<BitBoard> for BitBoard {
    type Output = BitBoard;

    #[inline]
    fn bitand(self, rhs: BitBoard) -> BitBoard {
        BitBoard(self.0.bitand(rhs.0))
    }
}

impl ops::BitAndAssign<BitBoard> for BitBoard {
    #[inline]
    fn bitand_assign(&mut self, rhs: BitBoard) {
        self.0.bitand_assign(rhs.0)
    }
}

impl ops::BitOr<BitBoard> for BitBoard {
    type Output = BitBoard;

    #[inline]
    fn bitor(self, rhs: BitBoard) -> BitBoard {
        BitBoard(self.0.bitor(rhs.0))
    }
}

impl ops::BitOrAssign<BitBoard> for BitBoard {
    #[inline]
    fn bitor_assign(&mut self, rhs: BitBoard) {
        self.0.bitor_assign(rhs.0)
    }
}

impl ops::BitXor<BitBoard> for BitBoard {
    type Output = BitBoard;

    #[inline]
    fn bitxor(self, rhs: BitBoard) -> BitBoard {
        BitBoard(self.0.bitxor(rhs.0))
    }
}

impl ops::BitXorAssign<BitBoard> for BitBoard {
    #[inline]
    fn bitxor_assign(&mut self, rhs: BitBoard) {
        self.0.bitxor_assign(rhs.0)
    }
}

impl ops::Shl<u32> for BitBoard {
    type Output = BitBoard;

    #[inline]
    fn shl(self, rhs: u32) -> BitBoard {
        BitBoard(self.0.wrapping_shl(rhs))
    }
}

impl ops::ShlAssign<BitBoard> for BitBoard {
    #[inline]
    fn shl_assign(&mut self, rhs: BitBoard) {
        self.0.shl_assign(rhs.0)
    }
}

impl ops::Shr<u32> for BitBoard {
    type Output = BitBoard;

    #[inline]
    fn shr(self, rhs: u32) -> BitBoard {
        BitBoard(self.0.wrapping_shr(rhs))
    }
}

impl ops::ShrAssign<BitBoard> for BitBoard {
    #[inline]
    fn shr_assign(&mut self, rhs: BitBoard) {
        self.0.shr_assign(rhs.0)
    }
}