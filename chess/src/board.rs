use std::fmt;
use std::str::FromStr;

use crate::attacks;
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
    side_to_move: Color,
    halfmove: u8,
    checkers: BitBoard,
    pinned: BitBoard,
    castle_rights: CastleRights,
    ep_square: EnPassantSquare,
    zobrist: Zobrist,
}

//#################################################################################################
//
//                                      struct Occupancy
//
//#################################################################################################

// A struct holding all necessary occupancy informations
#[derive(Clone, Debug)]
pub struct Occupancy {
    all: BitBoard,
    colored: [BitBoard; 2],
    free: BitBoard,
}

// ================================ pub impl

impl Occupancy {
    #[inline]
    pub fn all(&self) -> BitBoard {
        self.all
    }

    #[inline]
    pub fn colored(&self, color: Color) -> BitBoard {
        self.colored[color.idx()]
    }

    #[inline]
    pub fn free(&self) -> BitBoard {
        self.free
    }
}

// ================================ impl

impl Occupancy {
    #[inline]
    fn update_colored(&mut self, color: Color, mask: BitBoard) {
        self.colored[color.idx()] ^= mask;
    }
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
    #[inline]
    pub fn bitboard(&self, color: Color, piece: Piece) -> BitBoard {
        self.bitboards[color.idx()][piece.idx()]
    }

    #[inline]
    pub fn piece_at(&self, sq: Square) -> Option<(Color, Piece)> {
        self.mailbox[sq.idx()]
    }

    #[inline]
    pub fn occupancy(&self) -> &Occupancy {
        &self.occ
    }

    #[inline]
    pub fn king_sq(&self) -> Square {
        self.bitboard(self.state.side_to_move, Piece::King).as_square_unchecked()
    }

    pub fn is_legal(&self, mv: Move) -> bool {
        let (from, to) = mv.squares();

        if mv.is_castle() {
            // If the move is castle, we must check that the squares the king
            // passes are safe.
            let can_castle = |sq1, sq2| {
                (self.attackers_to(sq1) | self.attackers_to(sq2)).empty()
            };

            return match to {
                Square::G1 => can_castle(Square::H1, Square::F1),
                Square::G8 => can_castle(Square::H8, Square::F8),
                Square::C1 => can_castle(Square::A1, Square::D1),
                Square::C8 => can_castle(Square::A8, Square::D8),
                _ => unreachable!(),
            };
        } else if mv.is_en_passant() {
            // If the move is en passant, we must check that there is no double pin.
            let ep_square = self.state.ep_square.unwrap();
            let rank = ep_square.rank();
            let king_sq = self.king_sq();

            // If the king is on the same rank as the ep square (very rare).
            if rank.contains(king_sq) {
                let them = self.state.side_to_move.invert();
                // For every rook on that very same rank.
                for rook_sq in (self.bitboard(them, Piece::Rook) & rank).iter_squares() {
                    let between = BitBoard::between(king_sq, rook_sq);
                    // If the ep square is exactly between the king and the rook, 
                    // and there is nothing else than the two pawns, then it is an
                    // (incredibly rare) double pin.
                    if between.contains(ep_square) && between.count() == 2 {
                        return false;
                    }
                }
            }
        }

        // Any move is valid if the piece is not pinned or if it is moving in the squares 
        // projected from the king and onward.
        !self.state.pinned.contains(from) || BitBoard::ray_mask(self.king_sq(), from).contains(to)
    }

    pub fn is_pseudo_legal(&self, mv: Move) -> bool {
        // Piece is on our side, it does not go on a friendly piece
        // If it's en passant, there must be ep square
        // If it's castling, between squares must be cleared (use is_path_clear !)
        // If |checkers| == 1 => Don't forget to check that either the king is moving or the move is blocking/capturing the checker
        // If |checkers| == 2 => It must be a king move
        todo!()
    }

    /// Do the move without checking anything about it's legality.
    /// Returns true if the move is irreversible.
    pub fn do_move(&mut self, mv: Move) -> bool {
        // Store previous state and increment fullmove counter.
        self.prev_states.push(self.state.clone());
        if self.state.side_to_move == Color::Black {
            self.fullmove += 1;
        }

        // Invert the side to move.
        self.state.side_to_move = self.state.side_to_move.invert();

        // Extract base move infos and remove piece from it's starting position.
        let (from, to) = mv.squares();
        let (color, mut piece) = self.remove_piece::<true>(from);

        // Determine if the move is reversible or not.
        let reversible = mv.is_quiet() && piece != Piece::Pawn;

        if mv.is_castle() {
            // If the move is castling, move the rook as well.
            match to {
                Square::G1 => self.displace_piece::<true>(Square::H1, Square::F1),
                Square::G8 => self.displace_piece::<true>(Square::H8, Square::F8),
                Square::C1 => self.displace_piece::<true>(Square::A1, Square::D1),
                Square::C8 => self.displace_piece::<true>(Square::A8, Square::D8),
                _ => unreachable!(),
            };
        } else if mv.is_en_passant() {
            // If the move is en passant, remove the pawn at the en passant square.
            self.remove_piece::<true>(self.state.ep_square.unwrap());
        } else {
            // If the move is a capture, remove the enemy piece from the destination square.
            if mv.is_capture() {
                self.remove_piece::<true>(to);
            }
    
            // If the move is a promotion, 
            if mv.is_promote() {
                piece = mv.get_promote();
            }
        }

        // Finally, place the piece at it's destination.
        self.place_piece::<true>(color, piece, to);

        // Determine checkers and pinned bitboard.
        self.state.checkers = self.checkers();
        self.state.pinned = self.pinned();

        // Update castling rights and en passant square.
        self.state.castle_rights.update(from, to);
        if mv.is_double_push() {
            self.state.ep_square = EnPassantSquare::Some(to);
        } else {
            self.state.ep_square = EnPassantSquare::None;
        }

        // Update the halfmove clock.
        if reversible {
            self.state.halfmove += 1;
        } else {
            self.state.halfmove = 0;
        }

        // Invert the zobrist key, as we change color.
        self.state.zobrist = !self.state.zobrist;

        reversible
    }

    pub fn undo_move(&mut self, mv: Move) {
        // Them color.
        let them = self.state.side_to_move;

        // Restore the previous state and decrement the fullmove counter.
        self.state = self.prev_states.pop().unwrap();
        if self.state.side_to_move == Color::Black {
            self.fullmove -= 1;
        }

        // Extract basic move info and remove the piece from it's destination.
        let (from, to) = mv.squares();
        let (color, mut piece) = self.remove_piece::<false>(to);

        if mv.is_castle() {
            // If the move was castling, move the rook back as well.
            match to {
                Square::G1 => self.displace_piece::<true>(Square::F1, Square::H1),
                Square::G8 => self.displace_piece::<true>(Square::F8, Square::H8),
                Square::C1 => self.displace_piece::<true>(Square::D1, Square::A1),
                Square::C8 => self.displace_piece::<true>(Square::D8, Square::A8),
                _ => unreachable!(),
            };
        } else if mv.is_en_passant() {
            // If the move was en passant, place the enemy pawn back as well.
            self.place_piece::<false>(them, Piece::Pawn, self.state.ep_square.unwrap());
        } else {
            // If the move was a capture, replace the taken enemy piece in it's place.
            if mv.is_capture() {
                self.place_piece::<false>(them, mv.get_capture(), to);
            }
    
            // If the move was a promotion, the original piece was a pawn.
            if mv.is_promote() {
                piece = Piece::Pawn;
            }
        }

        self.place_piece::<false>(color, piece, from);
    }

    #[inline]
    pub fn attackers_to(&self, sq: Square) -> BitBoard {
        let us   = self.state.side_to_move;
        let them = us.invert();
        let occ  = self.occ.all;

        attacks::pawns(us, sq) & self.bitboard(them, Piece::Pawn) 
        | attacks::rook(sq, occ) & self.bitboard(them, Piece::Rook) 
        | attacks::knight(sq) & self.bitboard(them, Piece::Knight) 
        | attacks::bishop(sq, occ) & self.bitboard(them, Piece::Bishop) 
        | attacks::queen(sq, occ) & self.bitboard(them, Piece::Queen) 
        | attacks::king(sq) & self.bitboard(them, Piece::King)
    }

    #[inline]
    pub fn checkers(&self) -> BitBoard {
        self.attackers_to(self.king_sq())
    }

    #[inline]
    pub fn pinned(&self) -> BitBoard {
        let us = self.state.side_to_move;
        let them = us.invert();
        let queens = self.bitboard(them, Piece::Queen);
        let king_sq = self.king_sq();

        let mut pinned = BitBoard::EMPTY;

        for sq in (self.bitboard(them, Piece::Rook) | queens).iter_squares() {
            let between = BitBoard::between_straight(king_sq, sq);
            if (between & self.occ.all).count() == 1 {
                pinned |= between & self.occ.colored(us);
            }
        }

        for sq in (self.bitboard(them, Piece::Bishop) | queens).iter_squares() {
            let between = BitBoard::between_diagonal(king_sq, sq);
            if (between & self.occ.all).count() == 1 {
                pinned |= between & self.occ.colored(us);
            }
        }

        pinned
    }

    /// Efficiently tests for an upcoming repetition on the line,
    /// using cuckoo hashing.
    pub fn test_upcoming_repetition(&self) -> bool {
        if self.state.halfmove < 3 {
            return false;
        }

        let cur_zobrist = self.state.zobrist;
        let nth_zobrist = |n: u8| {
            self.prev_states[self.prev_states.len() - usize::from(n)].zobrist
        };

        let mut other = !(cur_zobrist ^ nth_zobrist(1));

        for d in (3..=self.state.halfmove).step_by(2) {
            other ^= !(nth_zobrist(d-1) ^ nth_zobrist(d));

            if other != Zobrist::ZERO {
                continue;
            }

            let diff = cur_zobrist ^ nth_zobrist(d);

            if cuckoo::is_hash_of_legal_move(self, diff) {
                return true;
            }
        }

        false
    }

    pub fn parse_move(&self, s: &str) -> Result<Move, ParseFenError> {
        todo!()
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
                if let Some((color, piece)) = self.piece_at(Square::from((x, y))) {
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
    #[inline]
    pub(crate) fn is_path_clear(&self, from: Square, to: Square) -> bool {
        (BitBoard::between(from, to) & self.occ.all).empty()
    }
}

// ================================ impl

impl Board {
    #[inline]
    fn place_piece<const ZOBRIST: bool>(&mut self, color: Color, piece: Piece, sq: Square) {
        self.bitboards[color.idx()][piece.idx()] ^= sq.into();
        self.mailbox[sq.idx()] = Some((color, piece));

        let mask = sq.into();
        self.occ.update_colored(color, mask);
        self.occ.all ^= mask;
        self.occ.free ^= mask;

        if ZOBRIST {
            self.state.zobrist ^= Zobrist::from((color, piece, sq));
        }
    }

    #[inline]
    fn remove_piece<const ZOBRIST: bool>(&mut self, sq: Square) -> (Color, Piece) {
        let (color, piece) = self.mailbox[sq.idx()].unwrap();
        self.bitboards[color.idx()][piece.idx()] ^= sq.into();
        self.mailbox[sq.idx()] = None;

        let mask = sq.into();
        self.occ.update_colored(color, mask);
        self.occ.all ^= mask;
        self.occ.free ^= mask;

        if ZOBRIST {
            self.state.zobrist ^= Zobrist::from((color, piece, sq));
        }

        (color, piece)
    }

    #[inline]
    fn displace_piece<const ZOBRIST: bool>(&mut self, from: Square, to: Square) -> (Color, Piece) {
        let (color, piece) = self.remove_piece::<ZOBRIST>(from);
        self.place_piece::<ZOBRIST>(color, piece, to);
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
                if let Some((color, piece)) = self.piece_at(Square::from((x, y))) {
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
        let split = s.split(' ').into_iter().collect::<Vec<_>>();
        
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
            bitboards:  [[BitBoard::EMPTY; 6]; 2],
            mailbox:    [None; 64],
            occ:        Occupancy::default(),
            state: StateInfo {
                side_to_move,
                halfmove,
                checkers:   BitBoard::EMPTY,
                pinned:    BitBoard::EMPTY,
                castle_rights,
                ep_square,
                zobrist:    Zobrist::default(),
            },
            prev_states: Vec::new(),
        };

        let ranks = split[0].split('/').into_iter().collect::<Vec<_>>();

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
                        board.bitboard(Color::White, Piece::Pawn);
                        board.place_piece::<true>(color, piece, sq);
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

        board.state.checkers = board.checkers();
        board.state.pinned   = board.pinned();

        Ok(board)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn do_and_undo() {
        use Square::*;

        crate::init();

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
            board.do_move(mv);
        }

        for &mv in moves.iter().rev() {
            board.undo_move(mv);
        }

        let default = Board::default();

        for color in Color::COLORS {
            for piece in Piece::PIECES {
                assert_eq!(
                    default.bitboard(color, piece),
                    board.bitboard(color, piece),
                )
            }
        }
        assert_eq!(default.occ.all, board.occ.all);
        assert_eq!(default.occ.colored, board.occ.colored);
        assert_eq!(default.occ.free, board.occ.free);
        assert_eq!(default.state.zobrist, board.state.zobrist);
    }
}