use std::ops::{BitXor, BitXorAssign, Not};

use crate::color::Color;
use crate::piece::Piece;
use crate::square::Square;

//#################################################################################################
//
//                                       Zobrist table
//
//#################################################################################################

/// The zobrist keys.
static mut KEYS: [[[Zobrist; 2]; 6]; 64] = [[[Zobrist::ZERO; 2]; 6]; 64];

/// The xorshift* algorithm for 64 bits numbers, producing
/// good enough pseudo-random numbers.
#[cold]
fn xorshift(seed: &mut u64) -> Zobrist {
    let mut x = *seed;
    x ^= x.wrapping_shl(13);
    x ^= x.wrapping_shr(7);
    x ^= x.wrapping_shl(17);
    *seed = x;
    Zobrist(x.wrapping_mul(0x2545F4914F6CDD1D))
}

/// Initializes the zobrist keys at the beginning of the program.
#[cold]
pub(crate) unsafe fn init() {
    // Changing the seed may make the cuckoo init() non terminating.
    let mut seed = 0x0C3B301A1AF7EE42;

    for sq in Square::SQUARES {
        for piece in Piece::PIECES {
            KEYS[usize::from(sq)][usize::from(piece)][usize::from(Color::White)] = xorshift(&mut seed);
            KEYS[usize::from(sq)][usize::from(piece)][usize::from(Color::Black)] = xorshift(&mut seed);
        }
    }
}

//#################################################################################################
//
//                                        struct Zobrist
//
//#################################################################################################

/// A zobrist key, that may be used for hashing.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Zobrist(u64);

// ================================ pub impl

impl Zobrist {
    /// The zero of that type.
    pub const ZERO: Zobrist = Zobrist(0);

    /// First hash function for indexing the cuckoo table.
    #[inline]
    pub fn h1(self) -> usize {
        (self.0.wrapping_shr(32) & 0x1FFF) as usize
    }

    /// Second hash function for indexing the cuckoo table.
    #[inline]
    pub fn h2(self) -> usize {
        (self.0.wrapping_shr(48) & 0x1FFF) as usize
    }

    /// Returns the index corresponding to this zobrist hash, for a table
    /// of size MAX.
    #[inline]
    pub fn idx<const MAX: usize>(self) -> isize {
        (self.0 % MAX as u64) as isize
    }

    /// Returns the raw value of this zobrist.
    #[inline]
    pub fn get_raw(self) -> u64 {
        self.0
    }
}

// ================================ traits impl

impl From<(Color, Piece, Square)> for Zobrist {
    /// Hashes a color, piece, square triplet.
    #[inline]
    fn from((color, piece, sq): (Color, Piece, Square)) -> Zobrist {
        unsafe {
            *KEYS.get_unchecked(usize::from(sq)).get_unchecked(usize::from(piece)).get_unchecked(usize::from(color))
        }
    }
}

impl BitXor<Zobrist> for Zobrist {
    type Output = Zobrist;

    #[inline]
    fn bitxor(self, rhs: Zobrist) -> Zobrist {
        Zobrist(self.0.bitxor(rhs.0))
    }
}

impl BitXorAssign<Zobrist> for Zobrist {
    #[inline]
    fn bitxor_assign(&mut self, rhs: Zobrist) {
        self.0.bitxor_assign(rhs.0);
    }
}

impl Not for Zobrist {
    type Output = Zobrist;

    #[inline]
    fn not(self) -> Zobrist {
        Zobrist(self.0.not())
    }
}