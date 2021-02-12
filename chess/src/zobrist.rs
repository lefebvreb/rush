use std::hash::{Hash, Hasher};
use std::ops::BitXorAssign;

use crate::bitboard::BitBoard;
use crate::castle_rights::CastleRights;
use crate::color::Color;
use crate::en_passant::EnPassantSquare;
use crate::game::Game;
use crate::piece::Piece;
use crate::square::Square;

//#################################################################################################
//
//                                     struct Position
//
//#################################################################################################

/// Decribes completely a given position, useful for storing in a hash table
#[derive(Debug)]
pub struct Position {
    bitboards: [[BitBoard; 6]; 2],
    castle_rights: CastleRights,
    color: Color,
    ep_rights: EnPassantSquare,
    zobrist: Zobrist,
}

// ================================ traits impl

impl PartialEq for Position {
    fn eq(&self, other: &Position) -> bool {
        self.bitboards == other.bitboards && self.castle_rights == other.castle_rights && self.color == other.color
    }
}

impl Eq for Position {}

impl From<&Game> for Position {
    fn from(game: &Game) -> Position {
        Position {
            bitboards: game.get_board().get_bitboards(),
            castle_rights: game.get_castle_rights(),
            color: game.get_color(),
            ep_rights: game.get_ep_rights(),
            zobrist: game.get_zobrist(),
        }
    }
}

impl Hash for Position {
    #[inline(always)]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.zobrist.hash(state);
    }
}

//#################################################################################################
//
//                                      struct Keys 
//
//#################################################################################################

// A struct holding the zobrist keys necessary to zobrist hashing
pub struct Keys {
    castle_rights_keys: [u64; 16],
    color_keys: [u64; 2],
    en_passant_file_key: [u64; 8],
    squares_colors_pieces_keys: [[[u64; 6]; 2]; 64],
}

//#################################################################################################
//
//                                     struct Zobrist
//
//#################################################################################################

/// A structure describing a 64 bit zobrist hash key
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Zobrist(u64);

// ================================ traits impl

impl BitXorAssign<(Color, Piece, Square)> for Zobrist {
    #[inline(always)]
    fn bitxor_assign(&mut self, rhs: (Color, Piece, Square)) {
        self.0 ^= ZOBRIST_KEYS.squares_colors_pieces_keys[rhs.2 as usize][rhs.0 as usize][rhs.1 as usize];
    }
}

impl BitXorAssign<Color> for Zobrist {
    #[inline(always)]
    fn bitxor_assign(&mut self, rhs: Color) {
        self.0 ^= ZOBRIST_KEYS.color_keys[rhs as usize];
    }
}

impl BitXorAssign<CastleRights> for Zobrist {
    #[inline(always)]
    fn bitxor_assign(&mut self, rhs: CastleRights) {
        self.0 ^= ZOBRIST_KEYS.castle_rights_keys[rhs.0 as usize];
    }
}

impl BitXorAssign<EnPassantSquare> for Zobrist {
    #[inline(always)]
    fn bitxor_assign(&mut self, rhs: EnPassantSquare) {
        match rhs {
            EnPassantSquare::Some(sq) => {
                self.0 ^= ZOBRIST_KEYS.en_passant_file_key[sq.x() as usize];
            }
            EnPassantSquare::None => (),
        }
    }
}

impl Hash for Zobrist {
    #[inline(always)]
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.0);
    }
}

//#################################################################################################
//
//                                  Zobrist keys initialization
//
//#################################################################################################

// A type holding 256 bits
#[allow(non_camel_case_types)]
type u256 = (u64, u64, u64, u64);

// The seed used to initialize zobrist keys
const SEED: u256 = (
    0xD6834E97A6CB2138,
    0xF5D7EFFD28F8F2F7,
    0x96287DBDEE1EA941,
    0x3FCE14B736DF4FF6,
);

// The RKISS random number generator
const fn random(s: &mut u256) -> u64 {
    let x = s.0.wrapping_sub(s.1.rotate_left(7));
    s.0 = s.1 ^ s.2.rotate_left(13);
    s.1 = s.2.wrapping_add(s.3.rotate_left(37));
    s.2 = s.3.wrapping_add(x);
    s.3 = x.wrapping_add(s.0);
    s.3
}

// Initialize the zobrist keys
pub const ZOBRIST_KEYS: Keys = {
    let (mut i, mut j, mut k);
    let mut s = SEED;

    // Enhance seed quality
    i = 0;
    while i < 20 {
        random(&mut s);
        i += 1;
    }

    let mut keys = Keys {
        squares_colors_pieces_keys: [[[0; 6]; 2]; 64],
        castle_rights_keys: [0; 16],
        color_keys: [0; 2],
        en_passant_file_key: [0; 8],
    };

    // Fill pieces keys
    i = 0;
    while i < 64 {
        j = 0;
        while j < 2 {
            k = 0;
            while k < 6 {
                keys.squares_colors_pieces_keys[i][j][k] = random(&mut s);
                k += 1;
            }
            j += 1;
        }
        i += 1;
    }

    // Fill castle rights keys
    i = 0;
    while i < 16 {
        let n = keys.castle_rights_keys.len();
        keys.castle_rights_keys[i] = random(&mut s);
        i += 1;
    }

    // Fill color keys
    i = 0;
    while i < 2 {
        keys.color_keys[i] = random(&mut s);
        i += 1;
    }

    // Fill en passant file keys
    i = 0;
    while i < 8 {
        keys.en_passant_file_key[i] = random(&mut s);
        i += 1;
    }

    keys
};