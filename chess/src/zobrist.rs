use std::ops;

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

struct Keys {
    castle_rights_keys: [u64; 16],
    color_keys: [u64; 2],
    en_passant_file_keys: [u64; 8],
    squares_colors_pieces_keys: [[[u64; 2]; 6]; 64],
}

static mut KEYS: Keys = Keys {
    castle_rights_keys: [0; 16],
    color_keys: [0; 2],
    en_passant_file_keys: [0; 8],
    squares_colors_pieces_keys: [[[0; 2]; 6]; 64],
};

fn xorshift(seed: &mut u64) -> u64 {
    let mut x = *seed;
    x ^= x.wrapping_shl(13);
    x ^= x.wrapping_shr(7);
    x ^= x.wrapping_shl(17);
    *seed = x;
    x.wrapping_mul(0x2545F4914F6CDD1D)
}

#[cold]
pub(crate) unsafe fn init() {
    let mut seed = 0x0C3B301A1Af7EE42;

    for i in 0..16 {
        KEYS.castle_rights_keys[i] = xorshift(&mut seed);
    }

    KEYS.color_keys[0] = xorshift(&mut seed);
    KEYS.color_keys[1] = xorshift(&mut seed);

    for i in 0..8 {
        KEYS.en_passant_file_keys[i] = xorshift(&mut seed);
    }

    for i in 0..64 {
        for j in 0..64 {
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

#[derive(Clone, Copy, Debug, Default)]
pub struct Zobrist(u64);

// ================================ traits impl

impl ops::BitXorAssign<CastleRights> for Zobrist {
    fn bitxor_assign(&mut self, rhs: CastleRights) {
        unsafe {
            self.0 ^= KEYS.castle_rights_keys[rhs.get_raw() as usize];
        }
    }
}

impl ops::BitXorAssign<Color> for Zobrist {
    fn bitxor_assign(&mut self, rhs: Color) {
        unsafe {
            self.0 ^= KEYS.color_keys[rhs as usize];
        }
    }
}

impl ops::BitXorAssign<EnPassantSquare> for Zobrist {
    fn bitxor_assign(&mut self, rhs: EnPassantSquare) {
        match rhs {
            EnPassantSquare::Some(sq) => unsafe {
                self.0 ^= KEYS.en_passant_file_keys[sq.x() as usize];
            },
            _ => (),
        }
    }
}

impl ops::BitXorAssign<(Color, Piece, Square)> for Zobrist {
    fn bitxor_assign(&mut self, rhs: (Color, Piece, Square)) {
        unsafe {
            let (color, piece, sq) = rhs;
            self.0 ^= KEYS.squares_colors_pieces_keys[sq as usize][piece as usize][color as usize];
        }
    }
}