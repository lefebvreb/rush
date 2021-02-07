use crate::bitboard::BitBoard;
use crate::castle_rights::CastleRights;
use crate::color::Color;
use crate::en_passant::EnPassantSquare;
use crate::game::Game;
use crate::piece::Piece;
use crate::square::Square;

//#################################################################################################
//
//                                      Position struct
//
//#################################################################################################

/// Decribes completely a given position, useful for storing in a hash table
#[derive(Debug)]
pub struct Position {
    bitboards: [[BitBoard; 6]; 2],
    castle_rights: CastleRights,
    color: Color,
    ep_rights: EnPassantSquare,
}

impl Position {
    pub fn get_key(&self) -> u64 {
        let mut key = 0;

        // Xor the keys corresponding to bitboards
        for color in &Color::COLORS {
            for piece in &Piece::PIECES {
                for sq in self.bitboards[*color as usize][*piece as usize].iter_squares() {
                    key ^= ZOBRIST_KEYS.squares_colors_pieces_keys[sq as usize][*color as usize][*piece as usize]
                }
            }
        }

        // Xor the key corresponding to those castle rights
        let idx: usize = self.castle_rights.into();
        key ^= ZOBRIST_KEYS.castle_rights_keys[idx];

        // Xor the key corresponding to the color
        key ^= ZOBRIST_KEYS.color_keys[self.color as usize];

        // Xor the key corresponding to those en passant rights
        if let EnPassantSquare::Some(sq) = self.ep_rights {
            key ^= ZOBRIST_KEYS.en_passant_file_key[sq.x() as usize];
        }

        key
    }
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
            ep_rights: game.get_ep_rights(),
        }
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
    en_passant_file_key: [u64; 8],
}

impl Keys {
    pub fn get_square(&self, color: Color, piece: Piece, sq: Square) -> u64 {
        self.squares_colors_pieces_keys[sq as usize][color as usize][piece as usize]
    }

    pub fn get_castle(&self, raw_rights: u8) -> u64 {
        self.castle_rights_keys[raw_rights as usize]
    }

    pub fn get_color(&self, color: Color) -> u64 {
        self.color_keys[color as usize]
    }

    pub fn get_ep(&self, sq: Square) -> u64 {
        self.en_passant_file_key[sq.x() as usize]
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