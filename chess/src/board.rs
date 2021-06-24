use std::fmt;
use std::str::FromStr;

use crate::bitboard::BitBoard;
use crate::castle_rights::CastleRights;
use crate::color::Color;
use crate::cuckoo;
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

#[derive(Clone, Debug)]
struct StateInfo {
    ep_square: EnPassantSquare,
    castle_rights: CastleRights,
    halfmove: u8,
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
    // Updates the occupancy according to the given color and mask.
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
//                                         struct Board
//
//#################################################################################################

#[derive(Clone, Debug)]
pub struct Board {
    ply: u16,
    side_to_move: Color,

    bitboards: [[BitBoard; 6]; 2],
    mailbox: [Option<(Color, Piece)>; 64],
    occ: Occupancy,

    state: StateInfo,
    prev_states: Vec<StateInfo>,
}

// ================================ pub impl

impl Board {
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

    /// Efficiently tests for an upcoming repetition on the line,
    /// using cuckoo hashing.
    #[inline]
    pub fn test_upcoming_repetition(&self) -> bool {
        if self.state.halfmove < 3 {
            return false;
        }

        let cur_zobrist = self.state.zobrist;
        let nth_zobrist = |n: u8| {
            self.prev_states[self.prev_states.len() - n as usize].zobrist
        };

        let mut other = !(cur_zobrist ^ nth_zobrist(1));

        for d in (3..self.state.halfmove).step_by(2) {
            other ^= !(nth_zobrist(d-1) ^ nth_zobrist(d));

            if other != Zobrist::ZERO {
                continue;
            }

            let diff = cur_zobrist ^ nth_zobrist(d);

            if cuckoo::is_hash_of_legal_move(self, diff) {
                return true;
            }
        }

        return false
    }
}

// ================================ pub(crate) impl

impl Board {
    /// Returns true from and to are not aligned, or if the squares
    /// between them are empty.
    #[inline(always)]
    pub(crate) fn is_path_clear(&self, from: Square, to: Square) -> bool {
        (BitBoard::between(from, to) & self.occ.all).empty()
    }
}

// ================================ traits impl

impl Default for Board {
    /// Returns the default position of chess.
    fn default() -> Board {
        Board::from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
    }
}

impl fmt::Display for Board {
    /// Formats the board to it's fen representation.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        todo!()
    }
}

impl FromStr for Board {
    type Err = ParseFenError;

    /// Tries to parse a board from a string in fen representation.
    fn from_str(s: &str) -> Result<Board, ParseFenError> {
        todo!()
    }
}