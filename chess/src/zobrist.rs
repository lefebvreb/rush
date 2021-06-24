use std::ops::{self, BitXor, BitXorAssign, Not};

use crate::castle_rights::CastleRights;
use crate::color::Color;
use crate::en_passant::EnPassantSquare;
use crate::piece::Piece;
use crate::square::Square;

//#################################################################################################
//
//                                  Zobrist tables
//
//#################################################################################################

// The Keys struct, to hold all the zobrist keys
struct Keys {
    castle_rights_keys: [Zobrist; 16],
    color_keys: [Zobrist; 2],
    en_passant_file_keys: [Zobrist; 8],
    squares_colors_pieces_keys: [[[Zobrist; 2]; 6]; 64],
}

// The only variable with a type of Keys, initialized at
// the beginning of the program with random values
static mut KEYS: Keys = Keys {
    castle_rights_keys: [Zobrist::ZERO; 16],
    color_keys: [Zobrist::ZERO; 2],
    en_passant_file_keys: [Zobrist::ZERO; 8],
    squares_colors_pieces_keys: [[[Zobrist::ZERO; 2]; 6]; 64],
};

// The xorshift* algorithm for 64 bits numbers, producing
// good enough pseudo-random numbers
#[cold]
fn xorshift(seed: &mut u64) -> Zobrist {
    let mut x = *seed;
    x ^= x.wrapping_shl(13);
    x ^= x.wrapping_shr(7);
    x ^= x.wrapping_shl(17);
    *seed = x;
    Zobrist(x.wrapping_mul(0x2545F4914F6CDD1D))
}

// Initialize the zobrist keys at the beginning of the program
#[cold]
pub(crate) unsafe fn init() {
    // Changing the seed may make the cuckoo init() non terminating
    let mut seed = 0x0C3B301A1Af7EE42;

    for i in 0..16 {
        KEYS.castle_rights_keys[i] = xorshift(&mut seed);
    }

    KEYS.color_keys[0] = Zobrist(0);
    KEYS.color_keys[1] = Zobrist(0xFFFFFFFFFFFFFFFF);

    for i in 0..8 {
        KEYS.en_passant_file_keys[i] = xorshift(&mut seed);
    }

    for i in 0..64 {
        for j in 0..6 {
            KEYS.squares_colors_pieces_keys[i][j][0] = xorshift(&mut seed);
            KEYS.squares_colors_pieces_keys[i][j][1] = xorshift(&mut seed);
        }
    }
}

//#################################################################################################
//
//                                        struct Zobrist
//
//#################################################################################################

/// A zobrist key, that may be used for hashing
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Zobrist(u64);

// ================================ pub impl

impl Zobrist {
    /// The zero of that type
    pub const ZERO: Zobrist = Zobrist(0);

    /// First hash function for indexing the cuckoo table
    #[inline(always)]
    pub fn h1(self) -> usize {
        (self.0.wrapping_shr(32) & 0x1FFF) as usize
    }

    /// Second hash function for indexing the cuckoo table
    #[inline(always)]
    pub fn h2(self) -> usize {
        (self.0.wrapping_shr(48) & 0x1FFF) as usize
    }
}

// ================================ traits impl

impl From<CastleRights> for Zobrist {
    #[inline(always)]
    fn from(rights: CastleRights) -> Zobrist {
        unsafe {
            KEYS.castle_rights_keys[rights.get_raw() as usize]
        }
    }
}

impl From<Color> for Zobrist {
    #[inline(always)]
    fn from(color: Color) -> Zobrist {
        unsafe {
            KEYS.color_keys[color.idx()]
        }
    }
}

impl From<EnPassantSquare> for Zobrist {
    #[inline(always)]
    fn from(ep: EnPassantSquare) -> Zobrist {
        match ep {
            EnPassantSquare::Some(sq) => unsafe {
                KEYS.en_passant_file_keys[sq.x() as usize]
            },
            _ => Zobrist::ZERO,
        }
    }
}

impl From<(Color, Piece, Square)> for Zobrist {
    #[inline(always)]
    fn from((color, piece, sq): (Color, Piece, Square)) -> Zobrist {
        unsafe {
            KEYS.squares_colors_pieces_keys[sq.idx()][piece.idx()][color.idx()]
        }
    }
}

impl BitXor<Zobrist> for Zobrist {
    type Output = Zobrist;

    #[inline(always)]
    fn bitxor(self, rhs: Zobrist) -> Zobrist {
        Zobrist(self.0.bitxor(rhs.0))
    }
}

impl BitXorAssign<Zobrist> for Zobrist {
    #[inline(always)]
    fn bitxor_assign(&mut self, rhs: Zobrist) {
        self.0.bitxor_assign(rhs.0);
    }
}

impl Not for Zobrist {
    type Output = Zobrist;

    #[inline(always)]
    fn not(self) -> Zobrist {
        Zobrist(self.0.not())
    }
}