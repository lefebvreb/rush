use std::ops::{BitXor, BitXorAssign, Not};

use crate::castle_rights::CastleRights;
use crate::color::Color;
use crate::en_passant::EnPassantSquare;
use crate::piece::Piece;
use crate::square::Square;

//#################################################################################################
//
//                                       Zobrist table
//
//#################################################################################################

/// The zobrist keys for pieces.
static mut PIECES: [[[Zobrist; 2]; 6]; 64] = [[[Zobrist::ZERO; 2]; 6]; 64];
static mut EP_FILES: [Zobrist; 8] = [Zobrist::ZERO; 8];
static mut CASTLING_RIGHTS: [Zobrist; 16] = [Zobrist::ZERO; 16];

/// The xorshift* algorithm for 64 bits numbers, producing
/// good enough pseudo-random numbers for initializing zobrist keys.
#[cold]
fn xorshift(seed: &mut u64) -> Zobrist {
    *seed ^= seed.wrapping_shl(13);
    *seed ^= seed.wrapping_shr(7);
    *seed ^= seed.wrapping_shl(17);
    Zobrist(seed.wrapping_mul(0x2545F4914F6CDD1D))
}

/// Initializes the zobrist keys at the beginning of the program.
#[cold]
pub(crate) unsafe fn init() {
    // Warning: changing the seed may make the cuckoo init() function non terminating.
    let mut seed = 0x0C3B301A1AF7EE42;

    for i in 0..64 {
        for j in 0..6 {
            PIECES[i][j][0] = xorshift(&mut seed);
            PIECES[i][j][1] = xorshift(&mut seed);
        }
    }

    for i in 0..8 {
        EP_FILES[i] = xorshift(&mut seed);
    }

    for i in 0..16 {
        CASTLING_RIGHTS[i] = xorshift(&mut seed);
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
        // SAFE: array initialization is done at startup
        unsafe {PIECES[usize::from(sq)][usize::from(piece)][usize::from(color)]}
    }
}

impl From<EnPassantSquare> for Zobrist {
    /// Hashes an en passant square.
    #[inline]
    fn from(ep_square: EnPassantSquare) -> Zobrist {
        match ep_square {
            // SAFE: array initialization is done at startup
            EnPassantSquare::Some(sq) => unsafe {EP_FILES[sq.x() as usize]}
            _ => Zobrist::ZERO,
        }
    }
}

impl From<CastleRights> for Zobrist {
    /// Hashes a castle rights object.
    #[inline]
    fn from(castle_rights: CastleRights) -> Zobrist {
        // SAFE: array initialization is done at startup
        unsafe {CASTLING_RIGHTS[usize::from(castle_rights.raw())]}
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