mod keys;
use keys::ZOBRIST_KEYS;

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
#[derive(Debug, Eq, PartialEq)]
pub struct Position {
    bitboards: [[BitBoard; 6]; 2],
    castle_rights: CastleRights,
    color: Color,
    ep_rights: EnPassantSquare,
    zobrist: Zobrist,
}

// ================================ traits impl

impl From<&Game> for Position {
    #[inline(always)]
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
//                                     struct Zobrist
//
//#################################################################################################

/// A structure describing a 64 bit zobrist hash key
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Zobrist(u64);

// ================================ pub impl

impl Zobrist {
    #[inline(always)]
    pub fn index<const N: usize>(&self) -> usize {
        self.0 as usize % N
    }

    #[inline(always)]
    pub fn get_lower(&self) -> u32 {
        (self.0 & 0xFFFFFFFF) as u32
    }
}

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
        self.0 ^= ZOBRIST_KEYS.castle_rights_keys[rhs.get_raw() as usize];
    }
}

impl BitXorAssign<EnPassantSquare> for Zobrist {
    #[inline(always)]
    fn bitxor_assign(&mut self, rhs: EnPassantSquare) {
        match rhs {
            EnPassantSquare::Some(sq) => {
                self.0 ^= ZOBRIST_KEYS.en_passant_file_key[sq.x() as usize];
            }
            _ => (),
        }
    }
}

impl Hash for Zobrist {
    #[inline(always)]
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.0);
    }
}