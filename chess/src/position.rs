use std::fmt;
use std::str::FromStr;

use crate::bitboard::BitBoard;
use crate::castle_rights::CastleRights;
use crate::color::Color;
use crate::en_passant::EnPassantSquare;
use crate::errors::ParseFenError;
use crate::moves::Move;
use crate::piece::Piece;
use crate::square::Square;
use crate::zobrist::Zobrist;

//#################################################################################################
//
//                                    struct StateInfo
//
//#################################################################################################

struct StateInfo {
    ep_square: EnPassantSquare,
    castle_rights: CastleRights,
    halfmove: u8,
    checkers: BitBoard,
    pinned: BitBoard,
    zobrist: Zobrist,
}

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
    ply: u16,
    side_to_move: Color,

    bitboards: [[BitBoard; 6]; 2],
    mailbox: [Option<(Color, Piece)>; 64],
    occ: Occupancy,

    state: StateInfo,
    state_history: Vec<StateInfo>,
}

// ================================ pub impl

impl Position {
    #[inline]
    pub fn is_legal(&self, mv: Move) -> bool {
        // Return true if the pseudo-legal move is legal:
        // - the king won't be in check after it is done:
        //   - |checkers| == 2 => king move
        //   - Use pinned bitboard: a move is legal if the piece is not pinned or
        //     if it is moving along the king-pinner axis
        // - castling rules are respected
        todo!()
    }

    #[inline]
    pub fn do_move(&mut self, mv: Move) {
        // Do the move, creating a new state
        todo!()
    }

    #[inline]
    pub fn undo_move(&mut self, mv: Move) {
        // Undo the move and reverts to the last state
        todo!()
    }

    #[inline]
    pub fn gives_check(&self, mv: Move) -> bool {
        // Return true if that pseudo-legal move
        // gives check to the other king
        todo!()
    }

    #[inline]
    pub fn attackers_to(&self, sq: Square) -> BitBoard {
        // Return the bitboard of the attackers to that square
        todo!()
    }
}

// ================================ traits impl

impl Default for Position {
    fn default() -> Position {
        Position::from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // To FEN
        todo!()
    }
}

impl FromStr for Position {
    type Err = ParseFenError;

    fn from_str(s: &str) -> Result<Position, ParseFenError> {
        // From FEN
        todo!()
    }
}