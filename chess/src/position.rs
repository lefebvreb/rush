use crate::bitboard::BitBoard;
use crate::castle_rights::CastleRights;
use crate::color::Color;
use crate::en_passant::EnPassantSquare;
use crate::zobrist::Zobrist;

//#################################################################################################
//
//                                      struct Occupancy
//
//#################################################################################################

// A struct holding all necessary occupancy informations
#[derive(Clone, Debug)]
struct Occupancy {
    white: BitBoard,
    black: BitBoard,
    all: BitBoard,
    free: BitBoard,
}

// ================================ impl

impl Occupancy {
    // Update the occupancy according to the given color and mask
    #[inline(always)]
    fn update(&mut self, color: Color, mask: BitBoard) {
        match color {
            Color::White => self.white ^= mask,
            Color::Black => self.black ^= mask,
        }
        self.all ^= mask;
        self.free ^= mask;
    }
}

//#################################################################################################
//
//                                    struct Position
//
//#################################################################################################

struct Position {
    side_to_move: Color,
    bitboards: [[BitBoard; 6]; 2],
    occ: Occupancy,

    ep_square: EnPassantSquare,
    castle_rights: CastleRights,

    zobrist: Zobrist,
}

// ================================ pub impl