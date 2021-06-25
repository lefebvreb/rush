use std::fmt;
use std::str::FromStr;

use crate::attacks;
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
    castle_rights: CastleRights,
    ep_square: EnPassantSquare,
    halfmove: u8,
    side_to_move: Color,
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
    all: BitBoard,
    colored: [BitBoard; 2],
    free: BitBoard,
}

// ================================ traits impl

impl Default for Occupancy {
    fn default() -> Occupancy {
        Occupancy {
            all: BitBoard::EMPTY,
            colored: [BitBoard::EMPTY; 2],
            free: BitBoard::FULL,
        }
    }
}

//#################################################################################################
//
//                                         struct Board
//
//#################################################################################################

#[derive(Clone, Debug)]
pub struct Board {
    fullmove: u16,

    bitboards: [[BitBoard; 6]; 2],
    mailbox: [Option<(Color, Piece)>; 64],
    occ: Occupancy,

    state: StateInfo,
    prev_states: Vec<StateInfo>,
}

// ================================ pub impl

impl Board {
    #[inline(always)]
    pub fn get_piece(&self, sq: Square) -> Option<(Color, Piece)> {
        self.mailbox[sq.idx()]
    }

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
    pub fn do_move(&mut self, mv: Move) -> bool {
        self.fullmove += 1;
        self.prev_states.push(self.state.clone());

        let (from, to) = (mv.from(), mv.to());
        self.state.castle_rights.update(from, to);
        self.state.side_to_move = self.state.side_to_move.invert();

        self.state.zobrist = !self.state.zobrist;
        let (color, piece) = self.remove_piece(from, true);

        match mv.get_type() {
            MoveType::Quiet => {
                self.place_piece(color, piece, to, true);
                if piece != Piece::Pawn {
                    self.state.ep_square = EnPassantSquare::None;
                    self.state.halfmove += 1;
                    return false;
                }
            },
            MoveType::Capture => {
                self.replace_piece(color, piece, to, true);
                self.state.ep_square = EnPassantSquare::None;
            },
            MoveType::Promote => {
                self.place_piece(color, mv.get_promote(), to, true);
                self.state.ep_square = EnPassantSquare::None;
            },
            MoveType::PromoteCapture => {
                self.replace_piece(color, mv.get_promote(), to, true);
                self.state.ep_square = EnPassantSquare::None;
            },
            MoveType::EnPassant => {
                self.place_piece(color, piece, to, true);
                self.remove_piece(self.state.ep_square.unwrap(), true);
                self.state.ep_square = EnPassantSquare::None;
            },
            MoveType::DoublePush => {
                self.place_piece(color, piece, to, true);
                self.state.ep_square = EnPassantSquare::Some(to);
            },
            MoveType::KingCastle => {
                self.place_piece(color, piece, to, true);
                match to {
                    Square::G1 => self.displace_piece(Square::H1, Square::F1, true),
                    Square::G8 => self.displace_piece(Square::H8, Square::F8, true),
                    _ => unreachable!(),
                };
                self.state.ep_square = EnPassantSquare::None;
            },
            MoveType::QueenCastle => {
                self.place_piece(color, piece, to, true);
                match to {
                    Square::C1 => self.displace_piece(Square::A1, Square::D1, true),
                    Square::C8 => self.displace_piece(Square::A8, Square::D8, true),
                    _ => unreachable!(),
                };
                self.state.ep_square = EnPassantSquare::None;
            },
        }

        self.state.halfmove = 0;
        true
    }

    pub fn undo_move(&mut self, mv: Move) {
        self.fullmove -= 1;
        let (from, to) = (mv.from(), mv.to());
        let color = self.state.side_to_move;

        self.state = self.prev_states.pop().unwrap();

        match mv.get_type() {
            MoveType::Quiet => {
                self.displace_piece(to, from, false);
            },
            MoveType::Capture => {
                let (color, piece) = self.replace_piece(color, mv.get_capture(), to, false);
                self.place_piece(color, piece, from, false);
            },
            MoveType::Promote => {
                let (color, _) = self.remove_piece(to, false);
                self.place_piece(color, Piece::Pawn, from, false);
            },
            MoveType::PromoteCapture => {
                let (color, _) = self.replace_piece(color, mv.get_capture(), to, false);
                self.place_piece(color, Piece::Pawn, from, false);
            },
            MoveType::EnPassant => {
                self.place_piece(color, Piece::Pawn, self.state.ep_square.unwrap(), false);
                let (color, _) = self.remove_piece(to, false);
                self.place_piece(color, Piece::Pawn, from, false);
            },
            MoveType::DoublePush => {
                self.displace_piece(to, from, false);
            },
            MoveType::KingCastle => {
                self.displace_piece(to, from, false);
                match to {
                    Square::G1 => self.displace_piece(Square::F1, Square::H1, false),
                    Square::G8 => self.displace_piece(Square::F8, Square::H8, false),
                    _ => unreachable!(),
                };
            },
            MoveType::QueenCastle => {
                self.displace_piece(to, from, false);
                match to {
                    Square::C1 => self.displace_piece(Square::D1, Square::A1, true),
                    Square::C8 => self.displace_piece(Square::D8, Square::A8, true),
                    _ => unreachable!(),
                };
            },
        }
    }

    #[inline]
    pub fn gives_check(&self, mv: Move) -> bool {
        // Return true if that pseudo-legal move
        // gives check to the other king
        todo!()
    }

    #[inline]
    pub fn attackers_to(&self, sq: Square) -> BitBoard {
        todo!()
    }

    /// Efficiently tests for an upcoming repetition on the line,
    /// using cuckoo hashing.
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

    /// Pretty-prints the board to stdout, using utf-8 characters
    /// to represent the pieces
    pub fn pretty_print(&self) -> String {
        const CHARS: [[char; 6]; 2] = [
            ['♙', '♖', '♘', '♗', '♕', '♔'],
            ['♟', '♜', '♞', '♝', '♛', '♚'],
        ];

        let mut res = String::new();

        res += "  a b c d e f g h\n";
        for y in (0..8).rev() {
            res += &(y + 1).to_string();
            for x in 0..8 {
                res.push(' ');
                if let Some((color, piece)) = self.mailbox[Square::from((x, y)).idx()] {
                    res.push(CHARS[color.idx()][piece.idx()]);
                } else {
                    res.push(' ');
                }
            }
            if y != 0 {
                res.push('\n');
            }
        }

        res
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
    fn place_piece(&mut self, color: Color, piece: Piece, sq: Square, zobrist: bool) {
        self.bitboards[color.idx()][piece.idx()] ^= sq.into();
        self.mailbox[sq.idx()] = Some((color, piece));

        let mask = sq.into();
        self.occ.colored[color.idx()] |= mask;
        self.occ.all ^= mask;
        self.occ.free ^= mask;

        if zobrist {
            self.state.zobrist ^= Zobrist::from((color, piece, sq));
        }
    }

    #[inline(always)]
    fn remove_piece(&mut self, sq: Square, zobrist: bool) -> (Color, Piece) {
        let (color, piece) = self.mailbox[sq.idx()].unwrap();
        self.bitboards[color.idx()][piece.idx()] ^= sq.into();
        self.mailbox[sq.idx()] = None;

        let mask = sq.into();
        self.occ.colored[color.idx()] |= mask;
        self.occ.all ^= mask;
        self.occ.free ^= mask;

        if zobrist {
            self.state.zobrist ^= Zobrist::from((color, piece, sq));
        }

        (color, piece)
    }

    #[inline(always)]
    fn replace_piece(&mut self, color: Color, piece: Piece, sq: Square, zobrist: bool) -> (Color, Piece){
        let (old_color, old_piece) = self.mailbox[sq.idx()].unwrap();

        self.mailbox[sq.idx()] = Some((color, piece));

        let mask = sq.into();
        self.occ.colored[color.idx()] |= mask;
        self.occ.colored[old_color.idx()] ^= mask;

        self.state.zobrist ^= Zobrist::from((color, piece, sq));
        self.state.zobrist ^= Zobrist::from((old_color, old_piece, sq));

        if zobrist {
            self.state.zobrist ^= Zobrist::from((color, piece, sq));
            self.state.zobrist ^= Zobrist::from((old_color, old_piece, sq));
        }

        (old_color, old_piece)
    }

    #[inline(always)]
    fn displace_piece(&mut self, from: Square, to: Square, zobrist: bool) -> (Color, Piece) {
        let (color, piece) = self.remove_piece(from, zobrist);
        self.place_piece(color, piece, to, zobrist);
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
        macro_rules! write_if_not_zero {
            ($i: expr) => {
                if $i != 0 {
                    write!(f, "{}", ('0' as u8 + $i) as char)?
                }
            };
        }
        
        for y in (0..8).rev() {
            let mut streak = 0;

            for x in 0..8 {
                if let Some((color, piece)) = self.get_piece(Square::from((x, y))) {
                    write_if_not_zero!(streak);
                    write!(f, "{}", match color {
                        Color::White => piece.to_string().to_uppercase(),
                        Color::Black => piece.to_string(),
                    })?;
                    streak = 0;
                } else {
                    streak += 1;
                }
            }

            write_if_not_zero!(streak);
            if y != 0 {
                write!(f, "/")?;
            }
        }

        write!(
            f, 
            " {} {} {} {} {}", 
            self.state.side_to_move,
            self.state.castle_rights,
            self.state.ep_square,
            self.state.halfmove,
            self.fullmove,
        )?;

        Ok(())
    }
}

impl FromStr for Board {
    type Err = ParseFenError;

    /// Tries to parse a board from a string in fen representation.
    fn from_str(s: &str) -> Result<Board, ParseFenError> {        
        let split = s.split(" ").into_iter().collect::<Vec<_>>();
        
        if split.len() != 6 {
            return Err(ParseFenError::new(format!("not enough arguments in fen string {:?}", s)));
        }

        let side_to_move = Color::from_str(split[1])?;
        let castle_rights = CastleRights::from_str(split[2])?;
        let ep_square = EnPassantSquare::from_str(split[3])?;
        let halfmove = u8::from_str(split[4])?;
        let fullmove = u16::from_str(split[5])?;

        let mut board = Board {
            fullmove,

            bitboards: [[BitBoard::EMPTY; 6]; 2],
            mailbox: [None; 64],
            occ: Occupancy::default(),

            state: StateInfo {
                castle_rights,
                ep_square,
                halfmove,
                side_to_move,
                zobrist: Zobrist::default(),
            },
            prev_states: Vec::new(),
        };

        let ranks = split[0].split("/").into_iter().collect::<Vec<_>>();

        if ranks.len() != 8 {
            return Err(ParseFenError::new(format!("not enough ranks in fen board {:?}", s)));
        }

        for (y, rank) in ranks.iter().enumerate() {
            let mut x = 0;
            let y = (7 - y) as i8;

            for c in rank.chars() {
                match c {
                    '1'..='8' => x += c as i8 - '1' as i8,
                    _ => {
                        let (color, piece) = Piece::from_char(c)?;
                        let sq = Square::from((x, y));
                        board.place_piece(color, piece, sq, true);
                    }
                }

                x += 1;
                if x > 8 {
                    return Err(ParseFenError::new(format!("rank {:?} is too large in fen string", rank)))
                }
            }

            if x != 8 {
                return Err(ParseFenError::new(format!("rank {:?} is too small fen string", rank)))
            }
        }

        Ok(board)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn do_and_undo() {
        use Square::*;

        let moves = vec![
            Move::double_push(D2, D4),
            Move::quiet(B8, C6),
            Move::quiet(D4, D5),
            Move::quiet(G7, G6),
            Move::quiet(C1, H6),
            Move::capture(F8, H6, Piece::Bishop),
            Move::quiet(D1, D3),
            Move::double_push(E7, E5),
            Move::en_passant(D5, E6),
            Move::quiet(G8, F6),
            Move::quiet(B1, C3),
            Move::king_castle(Color::Black),
            Move::quiet(E2, E5),
            Move::double_push(B7, B5),
            Move::queen_castle(Color::White),
            Move::quiet(B5, B4),
            Move::capture(E6, D7, Piece::Pawn),
            Move::quiet(B4, B3),
            Move::promote_capture(D7, C8, Piece::Bishop, Piece::Knight),
            Move::capture(B3, A2, Piece::Pawn),
            Move::quiet(C8, B6),
            Move::promote(A2, A1, Piece::Queen),
        ];

        let mut board = Board::default();

        
        for &mv in moves.iter() {
            eprintln!("move: {}", mv);
            board.do_move(mv);
            eprintln!("{}\n{}", board.pretty_print(), board);
        }

        for &mv in moves.iter().rev() {
            board.undo_move(mv);
        }

        let default = Board::default();

        for color in Color::COLORS {
            for piece in Piece::PIECES {
                assert_eq!(
                    default.bitboards[color.idx()][piece.idx()],
                    board.bitboards[color.idx()][piece.idx()],
                )
            }
        }
        assert_eq!(default.occ.all, board.occ.all);
        assert_eq!(default.occ.colored, board.occ.colored);
        assert_eq!(default.occ.free, board.occ.free);
    }
}