use std::fmt;
use std::str::FromStr;

use crate::bitboard::BitBoard;
use crate::castle_rights::CastleRights;
use crate::color::Color;
use crate::cuckoo;
use crate::en_passant::EnPassantSquare;
use crate::errors::ParseFenError;
use crate::moves::{Move, MoveType};
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
    colored: [BitBoard; 2],
    all: BitBoard,
    free: BitBoard,
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

    /// Do the move without checking anything about it's legality.
    /// Returns true if the move is irreversible.
    #[inline]
    pub fn do_move(&mut self, mv: Move) -> bool {
        self.prev_states.push(self.state.clone());

        let (from, to) = (mv.from(), mv.to());
        self.state.castle_rights.update(from);

        match mv.get_type() {
            MoveType::Quiet => {
                let (_, piece) = self.displace_piece(from, to);

                if piece != Piece::Pawn {
                    self.state.ep_square = EnPassantSquare::None;
                    self.state.halfmove += 1;
                    return false;
                }
            },
            MoveType::Capture => {
                let (color, piece) = self.remove_piece(from);
                self.replace_piece(color, piece, to);
                self.state.ep_square = EnPassantSquare::None;
            },
            MoveType::Promote => {
                let (color, _) = self.remove_piece(from);
                self.place_piece(color, mv.get_promote(), to);
                self.state.ep_square = EnPassantSquare::None;
            },
            MoveType::PromoteCapture => {
                let (color, _) = self.remove_piece(from);
                self.replace_piece(color, mv.get_promote(), to);
                self.state.ep_square = EnPassantSquare::None;
            },
            MoveType::EnPassant => {
                self.displace_piece(from, to);
                self.remove_piece(self.state.ep_square.unwrap());
                self.state.ep_square = EnPassantSquare::None;
            },
            MoveType::DoublePush => {
                let (color, _) = self.displace_piece(from, to);
                let y = match color {
                    Color::White => 2,
                    Color::Black => 5,
                };
                self.state.ep_square = EnPassantSquare::Some(Square::from((from.x(), y)));
            },
            MoveType::KingCastle => {
                self.displace_piece(from, to);
                match to {
                    Square::G1 => self.displace_piece(Square::H1, Square::F1),
                    Square::G8 => self.displace_piece(Square::H8, Square::F8),
                    _ => unreachable!(),
                };

                self.state.ep_square = EnPassantSquare::None;
            },
            MoveType::QueenCastle => {
                self.displace_piece(from, to);
                match to {
                    Square::C1 => self.displace_piece(Square::A1, Square::D1),
                    Square::C8 => self.displace_piece(Square::A8, Square::D8),
                    _ => unreachable!(),
                };
        
                self.state.ep_square = EnPassantSquare::None;
            },
        }

        self.state.halfmove = 0;
        true
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

// ================================ impl

impl Board {
    #[inline(always)]
    fn place_piece(&mut self, color: Color, piece: Piece, sq: Square) {
        self.bitboards[color.idx()][piece.idx()] ^= sq.into();
        self.mailbox[sq.idx()] = Some((color, piece));

        let mask = sq.into();
        self.occ.colored[color.idx()] |= mask;
        self.occ.all ^= mask;
        self.occ.free ^= mask;

        self.state.zobrist ^= Zobrist::from((color, piece, sq));
    }

    #[inline(always)]
    fn remove_piece(&mut self, sq: Square) -> (Color, Piece) {
        let (color, piece) = self.mailbox[sq.idx()].unwrap();
        self.bitboards[color.idx()][piece.idx()] ^= sq.into();
        self.mailbox[sq.idx()] = None;

        let mask = sq.into();
        self.occ.colored[color.idx()] |= mask;
        self.occ.all ^= mask;
        self.occ.free ^= mask;

        self.state.zobrist ^= Zobrist::from((color, piece, sq));

        (color, piece)
    }

    #[inline(always)]
    fn replace_piece(&mut self, color: Color, piece: Piece, sq: Square) {
        let (old_color, old_piece) = self.mailbox[sq.idx()].unwrap();

        self.mailbox[sq.idx()] = Some((color, piece));

        let mask = sq.into();
        self.occ.colored[color.idx()] |= mask;
        self.occ.colored[old_color.idx()] ^= mask;

        self.state.zobrist ^= Zobrist::from((color, piece, sq));
        self.state.zobrist ^= Zobrist::from((old_color, old_piece, sq));
    }

    #[inline(always)]
    fn displace_piece(&mut self, from: Square, to: Square) -> (Color, Piece) {
        let (color, piece) = self.remove_piece(from);
        self.place_piece(color, piece, to);
        (color, piece)
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