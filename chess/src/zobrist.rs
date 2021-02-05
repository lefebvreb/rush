use std::hash::{Hash, Hasher};

use crate::bitboard::BitBoard;
use crate::castle_rights::CastleRights;
use crate::color::Color;
use crate::game::Game;
use crate::piece::Piece;
use crate::square::Square;

//#################################################################################################
//
//                                      Position struct
//
//#################################################################################################

/// Decribes completely a given position
#[derive(Debug)]
pub struct Position {
    bitboards: [[BitBoard; 6]; 2],
    castle_rights: CastleRights,
    color: Color,
}

impl PartialEq for Position {
    fn eq(&self, other: &Position) -> bool {
        self.bitboards == other.bitboards && self.castle_rights == other.castle_rights && self.color == other.color
    }
}

impl From<&Game> for Position {
    fn from(game: &Game) -> Position {
        Position {
            bitboards: game.get_board().get_bitboards(),
            castle_rights: game.get_castle_rights(),
            color: game.get_color(),
        }
    }
}

impl Hash for Position {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for color in &Color::COLORS {
            for piece in &Piece::PIECES {
                for sq in self.bitboards[*color as usize][*piece as usize].iter_squares() {
                    state.write_u64(ZOBRIST_KEYS.get_piece(sq, *color, *piece));
                }
            }
        }
        state.write_u64(ZOBRIST_KEYS.get_castle(self.castle_rights));
        state.write_u64(ZOBRIST_KEYS.get_color(self.color));
    }
}

//#################################################################################################
//
//                                       Keys struct
//
//#################################################################################################

// A struct holding the zobrist keys necessary to zobrist hashing
pub(crate) struct Keys {
    squares_colors_pieces_keys: [[[u64; 6]; 2]; 64],
    castle_rights_keys: [u64; 16],
    color_keys: [u64; 2],
}

impl Keys {
    // Get the key associated to that square, color and piece
    #[inline(always)]
    pub fn get_piece(&self, sq: Square, color: Color, piece: Piece) -> u64 {
        self.squares_colors_pieces_keys[sq as usize][color as usize][piece as usize]
    }

    // Get the key associated to those castle rights
    #[inline(always)]
    pub fn get_castle(&self, castle_rights: CastleRights) -> u64 {
        self.castle_rights_keys[castle_rights.0 as usize]
    }

    // Get the key associated to that color
    #[inline(always)]
    pub fn get_color(&self, color_keys: Color) -> u64 {
        self.color_keys[color_keys as usize]
    }
}

//#################################################################################################
//
//                                  Zobrist keys initialization
//
//#################################################################################################

// A type alias to hold 256 bits
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
pub(crate) const ZOBRIST_KEYS: Keys = {
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
    };

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

    i = 0;
    while i < 16 {
        keys.castle_rights_keys[i] = random(&mut s);
        i += 1;
    }

    i = 0;
    while i < 2 {
        keys.color_keys[i] = random(&mut s);
        i += 1;
    }

    keys
};