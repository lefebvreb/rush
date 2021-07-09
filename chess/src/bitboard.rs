use std::fmt;
use std::ops;

use crate::color::Color;
use crate::square::Square;

//#################################################################################################
//
//                                            tables
//
//#################################################################################################

// An array whose ith element is 1 << i, precalculated as lookup
// is slightly faster than calculating them.
static mut SHIFTS: [BitBoard; 64] = [BitBoard::EMPTY; 64];

// These arrays contain bitboards indexed by two squares, from and to. They contain respectively:
// - the squares between from and to if they are aligned horizontally or vertically.
// - the squares between from and to if they are aligned diagonally.
// - the xor of the two former arrays.
// - same as the previous array but goes past to until the end of the board.
static mut SQUARES_BETWEEN_STRAIGHT: [[BitBoard; 64]; 64] = [[BitBoard::EMPTY; 64]; 64];
static mut SQUARES_BETWEEN_DIAGNOAL: [[BitBoard; 64]; 64] = [[BitBoard::EMPTY; 64]; 64];
static mut SQUARES_BETWEEN: [[BitBoard; 64]; 64] = [[BitBoard::EMPTY; 64]; 64];
static mut SQUARES_RAY_MASK: [[BitBoard; 64]; 64] = [[BitBoard::EMPTY; 64]; 64];

// Initializes the arrays above and the shifts table.
#[cold]
pub(crate) unsafe fn init() {
    for (i, shift) in SHIFTS.iter_mut().enumerate() {
        *shift = BitBoard(1 << i);
    }

    const SIGN: fn(i8) -> i8 = |i| match i {
        0 => 0,
        j if j > 0 => 1,
        _ => -1,
    };

    for sq1 in Square::SQUARES {
        for sq2 in Square::SQUARES {
            if sq1 == sq2 {
                continue;
            }

            let dx = sq2.x() - sq1.x();
            let dy = sq2.y() - sq1.y();

            let dir = (SIGN(dx), SIGN(dy));

            let between;
            if dx == 0 || dy == 0 {
                between = &mut SQUARES_BETWEEN_STRAIGHT
            } else if dx.abs() == dy.abs() {
                between = &mut SQUARES_BETWEEN_DIAGNOAL
            } else {
                continue;
            }

            let i = sq1.idx();
            let j = sq2.idx();

            let mut sq = sq1;
            loop {
                sq = sq.displace(dir).unwrap();
                if sq == sq2 {
                    break;
                }
                between[i][j] |= sq.into();
            }

            SQUARES_BETWEEN[i][j] = between[i][j];
            SQUARES_RAY_MASK[i][j] = SQUARES_BETWEEN[i][j];
            
            loop {
                SQUARES_RAY_MASK[i][j] |= sq.into();
                if let Some(s) = sq.displace(dir) {
                    sq = s;
                } else {
                    break;
                }
            }
        }
    }
}

//#################################################################################################
//
//                                       struct BitBoard
//
//#################################################################################################

/// Represents a 64 bits BitBoard.
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, Default)]
pub struct BitBoard(pub u64);

// ================================ pub impl

impl BitBoard {
    /// An empty BitBoard.
    pub const EMPTY: BitBoard = BitBoard(0);

    /// A full BitBoard.
    pub const FULL: BitBoard = BitBoard(0xFFFFFFFFFFFFFFFF);

    // The ranks of the board.
    pub const RANK_1: BitBoard = BitBoard(0xFF);
    pub const RANK_2: BitBoard = BitBoard(0xFF00);
    pub const RANK_3: BitBoard = BitBoard(0xFF0000);
    pub const RANK_4: BitBoard = BitBoard(0xFF000000);
    pub const RANK_5: BitBoard = BitBoard(0xFF00000000);
    pub const RANK_6: BitBoard = BitBoard(0xFF0000000000);
    pub const RANK_7: BitBoard = BitBoard(0xFF000000000000);
    pub const RANK_8: BitBoard = BitBoard(0xFF00000000000000);

    /// Return true if and only if the BitBoard self is empty.
    #[inline]
    pub fn empty(self) -> bool {
        self.0 == 0
    }

    /// Returns true if and only if the BitBoard self is not empty.
    #[inline]
    pub fn not_empty(self) -> bool {
        self.0 != 0
    }

    /// Returns an iterator over the bits of the BitBoard self.
    #[inline]
    pub fn iter_squares(mut self) -> impl Iterator<Item = Square> {
        (0..self.0.count_ones()).map(move |_| {
            let lsb = self.0.trailing_zeros();
            self &= self - BitBoard(1);
            Square::from(lsb as i8)
        })
    }

    // Returns the first square of the bitboard, with no checks.
    #[inline]
    pub fn as_square_unchecked(self) -> Square {
        Square::from(self.0.trailing_zeros() as i8)
    }
    
    /// Counts the bits of self that are 1.
    #[inline]
    pub fn count(self) -> u8 {
        self.0.count_ones() as u8
    }

    /// Returns true if that bitboard contains sq.
    #[inline]
    pub fn contains(self, sq: Square) -> bool {
        (self & sq.into()).0 != 0
    }

    /// Returns the rank where the pawns of the given color are promoting from.
    #[inline]
    pub fn promote_rank(color: Color) -> BitBoard {
        match color {
            Color::White => BitBoard::RANK_7,
            Color::Black => BitBoard::RANK_2,
        }
    }

    /// Returns a bitboard of the squares between from and to (exclusive) if 
    /// from and to are aligned horizontally or vertically. Returns an empty bitboard if they are not.
    #[inline]
    pub fn between_straight(from: Square, to: Square) -> BitBoard {
        unsafe {
            SQUARES_BETWEEN_STRAIGHT[from.idx()][to.idx()]
        }
    }

    /// Returns a bitboard of the squares between from and to (exclusive) if 
    /// from and to are aligned diagonally. Returns an empty bitboard if they are not.
    #[inline]
    pub fn between_diagonal(from: Square, to: Square) -> BitBoard {
        unsafe {
            SQUARES_BETWEEN_DIAGNOAL[from.idx()][to.idx()]
        }
    }

    /// Returns a bitboard of the squares between from and to (exclusive).
    /// if they are aligned. Returns an empty bitboard if they are not.
    #[inline]
    pub fn between(from: Square, to: Square) -> BitBoard {
        unsafe {
            SQUARES_BETWEEN[from.idx()][to.idx()]
        }
    }

    /// Returns a bitboard of the squares on the ray from-to, with
    /// from inclusive, if from and to are aligned.
    /// Returns an empty bitboard if they are not.
    #[inline]
    pub fn ray_mask(from: Square, to: Square) -> BitBoard {
        unsafe {
            SQUARES_RAY_MASK[from.idx()][to.idx()]
        }
    }

    /// Returns true if that bitboard contains more than
    /// one bit set to 1.
    #[inline]
    pub fn more_than_one(self) -> bool {
        (self & self - BitBoard(1)).0 != 0
    }
}

// ================================ pub(crate) impl

impl BitBoard {
    // Performs a parallel bits extract (pext) using the intrinsic (fast).
    #[cfg(target_feature = "bmi2")]
    #[inline]
    pub(crate) fn pext(self, mask: BitBoard) -> BitBoard {
        BitBoard(unsafe {
            std::arch::x86_64::_pext_u64(self.0, mask.0)
        })
    }

    // Performs a parallel bits extract (pext) without the intrinsic (slow).
    #[cfg(not(target_feature = "bmi2"))]
    #[inline]
    pub(crate) fn pext(self, mut mask: BitBoard) -> BitBoard {
        let (mut i, mut res) = (0, 0);

        while mask.0 != 0 {
            let tmp = mask.0;
            mask.0 &= mask.0 - 1;
            if (mask.0 ^ tmp) & self.0 != 0 {
                res |= 1 << i;
            }
            i += 1;
        }

        BitBoard(res)
    }

    // Performs a parallel bits deposit (pdep) using the intrinsic (fast).
    #[cfg(target_feature = "bmi2")]
    #[inline]
    pub(crate) fn pdep(self, mask: BitBoard) -> BitBoard {
        BitBoard(unsafe {
            std::arch::x86_64::_pdep_u64(self.0, mask.0)
        })
    }

    // Performs a parallel bits deposit (pdep) without the intrinsic (slow).
    #[cfg(not(target_feature = "bmi2"))]
    #[inline]
    pub(crate) fn pdep(self, mut mask: BitBoard) -> BitBoard {
        let (mut i, mut res) = (0, 0);

        while mask.0 != 0 {
            let tmp = mask.0;
            mask.0 &= mask.0 - 1;
            if self.0 & (1 << i) != 0 {
                res |= mask.0 ^ tmp;
            }
            i += 1;
        }

        BitBoard(res)
    }
}

// ================================ traits impl

impl fmt::Debug for BitBoard {
    // Prints the bitboard in hex form for quick debugging.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BitBoard(0x{:X})", self.0)
    }
}

impl fmt::Display for BitBoard {
    // Pretty-prints the bitboard for human eyes.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut bits = self.0.reverse_bits();

        writeln!(f, "┌BitBoard┐").unwrap();
        for _ in 0..8 {
            let rank = bits & 0xFF;
            bits = bits.wrapping_shr(8);
            writeln!(f, "│{:08b}│", rank).unwrap();
        }
        writeln!(f, "└────────┘").unwrap();

        Ok(())
    }
}

impl From<Square> for BitBoard {
    /// Returns the bitboard containing only that square.
    #[inline]
    fn from(sq: Square) -> BitBoard {
        unsafe {
            SHIFTS[sq.idx()]
        }
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