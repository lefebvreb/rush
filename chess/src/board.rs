use std::fmt;
use std::str::FromStr;

use anyhow::{Error, Result};

use crate::attacks;
use crate::bitboard::BitBoard;
use crate::castle_rights::CastleMask;
use crate::castle_rights::CastleRights;
use crate::color::Color;
use crate::cuckoo;
use crate::en_passant::EnPassantSquare;
use crate::movegen;
use crate::moves::Move;
use crate::piece::Piece;
use crate::square::Square;
use crate::zobrist::Zobrist;

//#################################################################################################
//
//                                    struct StateInfo
//
//#################################################################################################

/// An enum representing the status of a game.
#[derive(Debug)]
pub enum Status {
    Playing,
    Draw,
    Win(Color),
}

// ================================ pub impl

impl Status {
    /// Returns true if the status is Status::Playing.
    pub fn is_playing(&self) -> bool {
        matches!(self, Status::Playing)
    }
}

//#################################################################################################
//
//                                    struct StateInfo
//
//#################################################################################################

/// The state of the board at a given turn.
#[derive(Clone, Default, Debug)]
pub(crate) struct StateInfo {
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

/// A struct holding all necessary occupancy informations of a boad.
#[derive(Clone, Default, Debug)]
pub struct Occupancy {
    all: BitBoard,
    colored: [BitBoard; 2],
}

// ================================ pub impl

impl Occupancy {
    /// The monochrome occupancy bitboard.
    #[inline]
    pub fn all(&self) -> BitBoard {
        self.all
    }

    /// The colored occupancy bitboards.
    #[inline]
    pub fn colored(&self, color: Color) -> BitBoard {
        self.colored[usize::from(color)]
    }

    /// The free squares of the board.
    #[inline]
    pub fn free(&self) -> BitBoard {
        !self.all
    }
}

//#################################################################################################
//
//                                         struct Board
//
//#################################################################################################

/// A struct representing a complete position of chess, with many accessers and
/// methods to manipulate it.
#[derive(Clone, Debug)]
pub struct Board {
    ply: u16,

    bitboards: [[BitBoard; 6]; 2],
    mailbox: [Option<(Color, Piece)>; 64],
    occ: Occupancy,

    state: StateInfo,
    prev_states: Vec<StateInfo>,
}

// ================================ pub impl

impl Board {
    /// Tries to parse the fen string into a board.
    pub fn new(fen: &str) -> Result<Board> {
        Board::from_str(fen)
    }

    // ================================ Accessers

    /// Returns the color of the side to move.
    #[inline]
    pub fn get_side_to_move(&self) -> Color {
        self.state.side_to_move
    }

    /// Returns the color of the side to move.
    #[inline]
    pub fn get_other_side(&self) -> Color {
        self.state.side_to_move.invert()
    }

    /// Returns the bitboard containing the checkers in the current position.
    #[inline]
    pub fn get_checkers(&self) -> BitBoard {
        self.state.checkers
    }

    /// Returns the bitboard containing the checkers in the current position.
    #[inline]
    pub fn get_pinned(&self) -> BitBoard {
        self.state.pinned
    }

    /// Returns the halfmove counter.
    #[inline]
    pub fn get_halfmove(&self) -> u8 {
        self.state.halfmove
    }

    /// Returns the halfmove counter.
    #[inline]
    pub fn get_ply(&self) -> u16 {
        self.ply
    }

    /// Gets the bitboard corresponding to that color and piece type.
    #[inline]
    pub fn get_bitboard(&self, color: Color, piece: Piece) -> BitBoard {
        self.bitboards[usize::from(color)][usize::from(piece)]
    }

    /// Gets the (maybe) piece and it's color at that square.
    #[inline]
    pub fn get_piece(&self, sq: Square) -> Option<(Color, Piece)> {
        self.mailbox[usize::from(sq)]
    }

    /// Returns the occupancy object associated to that board.
    #[inline]
    pub fn get_occupancy(&self) -> &Occupancy {
        &self.occ
    }

    /// The zobrist hash of the current board.
    #[inline]
    pub fn get_zobrist(&self) -> Zobrist {
        self.state.zobrist
    }

    /// Clears the history of the board, making it impossible to 
    /// undo the previous moves but freeing a bit of memory.
    #[inline]
    pub fn clear_history(&mut self) {
        self.prev_states.clear()
    }

    /// Returns the type of the piece present at the given square.
    /// Panics if there are no pieces there.
    #[inline]
    pub fn get_piece_unchecked(&self, sq: Square) -> Piece {
        self.mailbox[usize::from(sq)].unwrap().1
    }

    // ================================ Methods

    // Returns the square the king of the side to move is occupying. 
    #[inline]
    pub fn king_sq(&self) -> Square {
        let king_bb = self.get_bitboard(self.get_side_to_move(), Piece::King);
        // SAFE: there is always a king on the board
        unsafe {king_bb.as_square_unchecked()}
    }

    /// Returns the status of the current game. Must be called every turn to be accurate.
    pub fn status(&self) -> Status {
        let halfmoves = self.get_halfmove();

        if halfmoves >= 50 {
            return Status::Draw;
        } else if halfmoves >= 3 {
            let repetitions = self.prev_states.iter().rev()
                .take(usize::from(self.get_halfmove()))
                .filter(|state| state.zobrist == self.state.zobrist)
                .count();

            if repetitions >= 3 {
                return Status::Draw;
            }
        }

        let mut legals = Vec::new();
        movegen::legals(self, &mut legals);
        if legals.len() == 0 {
            if self.get_checkers().empty() {
                return Status::Draw;
            } else {
                return Status::Win(self.get_other_side());
            }
        }

        Status::Playing
    }

    /// Returns true if that pseudo-legal move is legal.
    /// In particular, checks whether or not the move does not violate pin
    /// (or double pin for en passant moves), or, if it is a castling move,
    /// whether or not the squares the king traverses are safe.
    pub fn is_legal(&self, mv: Move) -> bool {
        let (from, to) = mv.squares();

        if mv.is_castle() {
            // If the move is castle, we must check that the squares the king
            // passes are safe.
            let can_castle = |sq1, sq2| {
                let occ = self.get_occupancy().all();
                (self.attackers_to(sq1, occ) | self.attackers_to(sq2, occ)).empty()
            };

            return match to {
                Square::G1 => can_castle(Square::F1, Square::G1),
                Square::G8 => can_castle(Square::F8, Square::G8),
                Square::C1 => can_castle(Square::C1, Square::D1),
                Square::C8 => can_castle(Square::C8, Square::D8),
                _ => unreachable!(),
            };
        } else if mv.is_en_passant() {
            // If the move is en passant, we must check that there is no double pin.
            let ep_square = self.get_ep_square().unwrap();
            let rank = ep_square.rank();
            let king_sq = self.king_sq();

            // If the king is on the same rank as the ep square (very rare).
            if rank.contains(king_sq) {
                let them = self.get_other_side();
                let sliders = self.get_bitboard(them, Piece::Rook) | self.get_bitboard(them, Piece::Queen);

                // For every rook or queen on that very same rank.
                for slider_sq in (sliders & rank).iter_squares() {
                    let between = BitBoard::between(king_sq, slider_sq);
                    let occ = self.get_occupancy().all();
                    
                    // If the ep square is exactly between the king and the rook, 
                    // and there is nothing else than the two pawns, then it is an
                    // (incredibly rare) double pin.
                    if between.contains(ep_square) && (between & occ).is_two() {
                        return false;
                    }
                }
            }
        } else if from == self.king_sq() {
            let new_occ = (self.get_occupancy().all() | BitBoard::from(to)) ^ BitBoard::from(from);
            // If the move is done by the king, check the square it is moving to is safe.
            return self.attackers_to(to, new_occ).empty();
        }

        // Any move is valid if the piece is not pinned or if it is moving in the squares 
        // projected from the king and onward.
        !self.get_pinned().contains(from) || BitBoard::ray_mask(self.king_sq(), from).contains(to)
    }

    /// Returns true if that random move is pseudo-legal. Only assumes that the
    /// move was created through one of the Move type's metods.
    pub fn is_pseudo_legal(&self, mv: Move) -> bool {
        macro_rules! verify {($cond: expr) => {if !($cond) {return false;}}}

        let (from, to) = mv.squares();

        // Verify that the from square is occupied.
        if let Some((color, piece)) = self.get_piece(from) {
            // Verify it is one of our pieces.
            verify!(color == self.get_side_to_move());

            // Verify to square occupied <=> move is a capture and the square 
            // is occupied by the piece stored in the move.
            if let Some((color, piece)) = self.get_piece(to) {
                verify!(mv.is_capture() && color != self.get_side_to_move() && piece == mv.get_capture());
            } else {
                verify!(!mv.is_capture());
            }

            let checkers = self.get_checkers();

            // Special case for the king.
            if piece == Piece::King {
                // If the move is castling.
                if mv.is_castle() {
                    let can_castle = |king_sq, rook_sq, mask| {
                        self.get_piece(rook_sq) == Some((color, Piece::Rook)) &&
                        self.is_path_clear(king_sq, rook_sq) && 
                        self.get_castle_rights().has(mask)
                    };

                    // The king must not be in check and the path between the king and the rook must be clear.
                    // Plus, there must be a rook on the rook square and we must possess the adequate
                    // castling rights.
                    return checkers.empty() && match color {
                        Color::White => match (from, to) {
                            (Square::E1, Square::G1) => can_castle(Square::E1, Square::H1, CastleMask::WhiteOO),
                            (Square::E1, Square::C1) => can_castle(Square::E1, Square::A1, CastleMask::WhiteOOO),
                            _ => return false,
                        },
                        Color::Black => match (from, to) {
                            (Square::E8, Square::G8) => can_castle(Square::E8, Square::H8, CastleMask::BlackOO),
                            (Square::E8, Square::C8) => can_castle(Square::E8, Square::A8, CastleMask::BlackOOO),
                            _ => return false,
                        },
                    };
                }

                // Checking wether the square the king is valid for a king.
                return attacks::king(from).contains(to);
            } else {
                // The move can't be a castle if the piece moving is not the king.
                verify!(!mv.is_castle());

                // If there are any checkers.
                if checkers.not_empty() {
                    // Two checkers, the piece moving must be the king.
                    verify!(!checkers.more_than_one());

                    // One checker, the piece moving must either block or capture the enemy piece.
                    // SAFE: there is always a king on the board
                    let checker = unsafe {checkers.as_square_unchecked()};
                    let blocking_zone = BitBoard::between(self.king_sq(), checker);
                    verify!((blocking_zone | checkers).contains(to));
                }
            }

            // Special case for pawns.
            if piece == Piece::Pawn {
                if mv.is_en_passant() {
                    // There must be an en passant square.
                    verify!(self.get_ep_square().is_some());
                    let ep_square = self.get_ep_square().unwrap();

                    // The ep square must be between the move's squares.
                    // SAFE: 0 <= to.x() < 8 and 0 <= from.y() < 8
                    verify!(ep_square == unsafe {Square::from_xy_unchecked(to.x(), from.y())});

                    // It is a valid pawn attack too.
                    return attacks::pawn(color, from).contains(to);
                } else {
                    // If the move is a promotion, it must go to the first or last rank.
                    verify!(to.y() == 0 || to.y() == 7 || !mv.is_promote());

                    // Verify that the move is legal for a pawn.
                    if mv.is_capture() {
                        return attacks::pawn(color, from).contains(to);
                    } else {
                        return if mv.is_double_push() {
                            attacks::pawn_double_push(color, from)
                        } else {
                            attacks::pawn_push(color, from)
                        } == Some(to);
                    };
                }
            } else {
                // If the piece is not a pawn, the move can't be of any of those types. 
                verify!(!mv.is_en_passant() && !mv.is_double_push() && !mv.is_promote());
            }

            // Monochrome occupancy.
            let occ = self.get_occupancy().all();

            // For any other piece, verify the move would be valid on an empty board.
            return match piece {
                Piece::Rook => attacks::rook(from, occ),
                Piece::Knight => attacks::knight(from),
                Piece::Bishop => attacks::bishop(from, occ),
                Piece::Queen => attacks::queen(from, occ),
                _ => unreachable!(),
            }.contains(to);
        }

        false
    }

    /// Do the move without checking anything about it's legality.
    /// Returns true if the move is irreversible.
    pub fn do_move(&mut self, mv: Move) -> bool {
        // Clone the previous state to store it later.
        let old_state = self.state.clone();

        // Undo the zobrist hashing of the ep square and castle rights.
        self.state.zobrist ^= Zobrist::from(old_state.ep_square);
        self.state.zobrist ^= Zobrist::from(old_state.castle_rights);

        // Store previous state and increment fullmove counter.
        self.prev_states.push(old_state);
        self.ply += 1;

        // Invert the side to move.
        self.state.side_to_move = self.get_other_side();

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
            self.remove_piece::<true>(self.get_ep_square().unwrap());
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
        self.state.zobrist ^= Zobrist::from(self.state.castle_rights);

        if mv.is_double_push() {
            let ep_square = EnPassantSquare::Some(to);
            self.state.ep_square = ep_square;
            self.state.zobrist ^= Zobrist::from(ep_square);
        } else {
            self.state.ep_square = EnPassantSquare::None;
        }

        // Update the halfmove clock.
        if reversible {
            self.state.halfmove += 1;
        } else {
            self.state.halfmove = 0;
        }

        // Invert zobrist since we change side.
        self.state.zobrist = !self.state.zobrist;
    
        reversible
    }

    /// Undoes the move, reverting the board to it's previous state.
    pub fn undo_move(&mut self, mv: Move) {
        // Them color.
        let them = self.get_side_to_move();

        // Restore the previous state and decrement the fullmove counter.
        self.state = self.prev_states.pop().unwrap();
        self.ply -= 1;

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
            self.place_piece::<false>(them, Piece::Pawn, self.get_ep_square().unwrap());
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

    /// Efficiently tests for an upcoming repetition on the line,
    /// using cuckoo hashing.
    pub fn test_upcoming_repetition(&self) -> bool {
        if self.get_halfmove() < 4 {
            return false;
        }

        let cur_zobrist = self.state.zobrist;
        let nth_zobrist = |n: u8| {
            self.prev_states[self.prev_states.len() - usize::from(n)].zobrist
        };

        let mut other = !(cur_zobrist ^ nth_zobrist(1));

        let n = 1 + usize::from(self.get_halfmove()).min(self.prev_states.len()) as u8;
        for d in (3..n).step_by(2) {
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

    /// Parses the move, checking the legality of the move.
    pub fn parse_move(&self, s: &str) -> Result<Move> {
        let mv = match s.len() {
            4 => {
                let from = Square::from_str(&s[0..2])?;
                let to = Square::from_str(&s[2..4])?;

                match self.get_piece(from) {
                    Some((_, Piece::Pawn)) => {
                        if from.x() == to.x() {
                            if (to.y() - from.y()).abs() == 2 {
                                Move::double_push(from, to)
                            } else {
                                Move::quiet(from, to)
                            }
                        } else if let Some((_, capture)) = self.get_piece(to) {
                            Move::capture(from, to, capture)
                        } else {
                            Move::en_passant(from, to)
                        }
                    },
                    Some((_, Piece::King)) => {
                        if (to.x() - from.x()).abs() == 2 {
                            Move::castle(from, to)
                        } else if let Some((_, capture)) = self.get_piece(to) {
                            Move::capture(from, to, capture)
                        } else {
                            Move::quiet(from, to)
                        }
                    },
                    _ => {
                        if let Some((_, capture)) = self.get_piece(to) {
                            Move::capture(from, to, capture)
                        } else {
                            Move::quiet(from, to)
                        }
                    },
                }
            },
            5 => {
                let from = Square::from_str(&s[0..2])?;
                let to = Square::from_str(&s[2..4])?;

                let promote = match s.chars().nth(4).unwrap() {
                    'r' => Piece::Rook,
                    'n' => Piece::Knight,
                    'b' => Piece::Bishop,
                    'q' => Piece::Queen,
                    _ => return Err(Error::msg("Unrecognized promotion.")),
                };
    
                if let Some((_, capture)) = self.get_piece(to) {
                    Move::promote_capture(from, to, capture, promote)
                } else {
                    Move::promote(from, to, promote)
                }
            },
            _ => return Err(Error::msg("A move should be encoded in pure algebraic coordinate notation.")),
        };

        if self.is_pseudo_legal(mv) && self.is_legal(mv) {
            Ok(mv)
        } else {
            Err(Error::msg("Move is illegal in this context."))
        }
    }

    /// Pretty-prints the board into a terminal, with emojis for pieces and ansi colors for squares.
    pub fn pretty_print(&self) -> String {
        const RESET: &str = "\x1b[0m";
        const BLACK: &str = "\x1b[40;1m";
        const CHARS: [[char; 6]; 2] = [
            ['♙', '♘', '♗', '♖', '♕', '♔'],
            ['♟', '♞', '♝', '♜', '♛', '♚'],
        ];

        let mut res = String::new();

        res.extend("  a b c d e f g h\n".chars());
        for y in (0..8).rev() {
            let rankc = char::from('1' as u8 + y);
            res.push(rankc);

            for x in 0..8 {
                res.push(' ');

                let sq = Square::from((x, y as i8));
                let ch = match self.get_piece(sq) {
                    Some((color, piece)) => CHARS[usize::from(color)][usize::from(piece)],
                    None => ' ',
                };

                if sq.parity() == Color::Black {
                    res.extend(format!("{}{}{}", BLACK, ch, RESET).chars());
                } else {
                    res.push(ch);
                }
            }

            res.push(rankc);
            if y != 0 {
                res.push('\n');
            }
        }
        res.extend("\n  a b c d e f g h".chars());

        res
    }
}

// ================================ pub(crate) impl

impl Board {
    /// Returns the castling rights in the current position.
    #[inline]
    pub(crate) fn get_castle_rights(&self) -> CastleRights {
        self.state.castle_rights
    }

    /// Returns the en passant square of the current position.
    #[inline]
    pub(crate) fn get_ep_square(&self) -> EnPassantSquare {
        self.state.ep_square
    }

    /// Returns true from and to are not aligned, or if the squares
    /// between them are empty.
    #[inline]
    pub(crate) fn is_path_clear(&self, from: Square, to: Square) -> bool {
        (BitBoard::between(from, to) & self.occ.all).empty()
    }

    /// Returns the bitboard of all the attackers to that square. Does not take
    /// en passant into account.
    #[inline]
    pub(crate) fn attackers_to(&self, sq: Square, occ: BitBoard) -> BitBoard {
        let us = self.get_side_to_move();
        let them = self.get_other_side();

        let queens = self.get_bitboard(them, Piece::Queen);

        attacks::pawn(us, sq) & self.get_bitboard(them, Piece::Pawn) 
        | attacks::rook(sq, occ) & (self.get_bitboard(them, Piece::Rook) | queens)
        | attacks::knight(sq) & self.get_bitboard(them, Piece::Knight) 
        | attacks::bishop(sq, occ) & (self.get_bitboard(them, Piece::Bishop) | queens)
        | attacks::king(sq) & self.get_bitboard(them, Piece::King)
    }
}

// ================================ impl

impl Board {
    /// Places a piece of the given color on the given square. If ZOBRIST is true, 
    /// updates the zobrist key accordingly.
    #[inline]
    fn place_piece<const ZOBRIST: bool>(&mut self, color: Color, piece: Piece, sq: Square) {
        self.mailbox[usize::from(sq)] = Some((color, piece));
        
        let mask = sq.into();
        self.bitboards[usize::from(color)][usize::from(piece)] ^= mask;
        self.occ.all ^= mask;
        self.occ.colored[usize::from(color)] ^= mask;

        if ZOBRIST {
            self.state.zobrist ^= Zobrist::from((color, piece, sq));
        }
    }

    /// Removes the piece on the given square. If ZOBRIST is true, updates the
    /// zobrist key accordingly.
    #[inline]
    fn remove_piece<const ZOBRIST: bool>(&mut self, sq: Square) -> (Color, Piece) {
        let (color, piece) = self.mailbox[usize::from(sq)].unwrap();
        self.mailbox[usize::from(sq)] = None;
        
        let mask = sq.into();
        self.bitboards[usize::from(color)][usize::from(piece)] ^= mask;
        self.occ.all ^= mask;
        self.occ.colored[usize::from(color)] ^= mask;

        if ZOBRIST {
            self.state.zobrist ^= Zobrist::from((color, piece, sq));
        }

        (color, piece)
    }

    /// Dispalces a piece between the two given squares. If ZOBRIST is true, updates the
    /// zobrist key accordingly.
    #[inline]
    fn displace_piece<const ZOBRIST: bool>(&mut self, from: Square, to: Square) -> (Color, Piece) {
        let (color, piece) = self.remove_piece::<ZOBRIST>(from);
        self.place_piece::<ZOBRIST>(color, piece, to);
        (color, piece)
    }

    /// The bitboard of the checkers to the current king.
    #[inline]
    fn checkers(&self) -> BitBoard {
        let occ = self.get_occupancy().all();
        self.attackers_to(self.king_sq(), occ)
    }

    /// The bitboard of the currently pinned pieces.
    #[inline]
    fn pinned(&self) -> BitBoard {
        let us = self.get_side_to_move();
        let occ_us = self.occ.colored(us);
        let them = self.get_other_side();
        let queens = self.get_bitboard(them, Piece::Queen);
        let king_sq = self.king_sq();

        let mut pinned = BitBoard::EMPTY;

        for sq in (self.get_bitboard(them, Piece::Rook) | queens).iter_squares() {
            let between = BitBoard::between_straight(king_sq, sq);
            if (between & self.occ.all).is_one() {
                pinned |= between & occ_us;
            }
        }

        for sq in (self.get_bitboard(them, Piece::Bishop) | queens).iter_squares() {
            let between = BitBoard::between_diagonal(king_sq, sq);
            if (between & self.occ.all).is_one() {
                pinned |= between & occ_us;
            }
        }

        pinned
    }
}

// ================================ traits impl

impl Default for Board {
    /// Returns an empty board.
    fn default() -> Board {
        Board {
            ply: 0,

            bitboards: Default::default(),
            mailbox: [None; 64],
            occ: Occupancy::default(),
        
            state: StateInfo::default(),
            prev_states: Vec::new(),
        }
    }
}

impl fmt::Display for Board {
    /// Formats the board to it's fen representation.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // fen string
        macro_rules! write_if_not_zero {
            ($i: expr) => {
                if $i != 0 {
                    write!(f, "{}", ('0' as u8 + $i) as char)?;
                }
            };
        }
        
        for y in (0..8).rev() {
            let mut streak = 0;

            for x in 0..8 {
                if let Some((color, piece)) = self.get_piece(Square::from((x, y))) {
                    write_if_not_zero!(streak);
                    write!(f, "{}", piece.as_char(color))?;
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

        write!(f, " {} {} {} {} {}", 
            self.get_side_to_move(),
            self.get_castle_rights(),
            self.get_ep_square(),
            self.get_halfmove(),
            1 + self.get_ply() / 2,
        )?;

        Ok(())
    }
}

impl<'a> FromStr for Board {
    type Err = Error;

    /// Tries to parse a board from a string in fen representation.
    fn from_str(s: &str) -> Result<Board> {
        let mut split = s.split(' ');

        // Closure to get the next arg, or return an error if there is not.
        let mut next_arg = || split.next().ok_or_else(|| Error::msg("not enough arguments in fen string"));

        // Parse the fen string later.
        let ranks: Vec<_> = next_arg()?.split('/').collect();
        if ranks.len() != 8 {
            return Err(Error::msg("Invalid number of ranks in fen string."));
        }

        // An empty board.
        let mut board = Board::default();

        // Parse the state arguments.
        board.state.side_to_move = Color::from_str(next_arg()?)?;
        board.state.castle_rights = CastleRights::from_str(next_arg()?)?;
        board.state.ep_square = EnPassantSquare::from_str(next_arg()?)?;
        board.state.halfmove = u8::from_str(next_arg()?)?;
        board.ply = u16::from_str(next_arg()?)?;

        if split.next().is_some() {
            return Err(Error::msg("Too many arguments in fen string."));
        }

        // Parse the fen board.
        for (y, &rank) in ranks.iter().enumerate() {           
            let mut x = 0;
            for c in rank.chars() {
                match c {
                    '1'..='8' => x += c.to_digit(10).unwrap(),
                    _ => {
                        let (color, piece) = Piece::from_char(c)?;
                        let sq = Square::from((x as i8, 7 - y as i8));
                        board.get_bitboard(Color::White, Piece::Pawn);
                        board.place_piece::<true>(color, piece, sq);
                        x += 1;
                    }
                }
                
                if x > 8 {
                    return Err(Error::msg("Rank too large in fen string."));
                }
            }

            if x != 8 {
                return Err(Error::msg("Rank too small in fen string."));
            }
        }

        // Check that both sides have only one king
        for color in Color::COLORS {
            if !board.get_bitboard(color, Piece::King).is_one() {
                return Err(Error::msg("Invalid number of kings on the board."));
            }
        }

        // Check that the side to move only has at most two checkers.
        board.state.checkers = board.checkers();
        if board.get_checkers().count() > 2 {
            return Err(Error::msg("Too many checkers for the side to move."));
        }
        // Check that the other side's king is not in check.
        board.state.side_to_move = board.get_other_side();
        if board.checkers().not_empty() {
            return Err(Error::msg("Other side's king is under check, which is illegal."));
        }
        board.state.side_to_move = board.get_other_side();

        // Compute the pinned pieces of the board.
        board.state.pinned = board.pinned();

        // TODO: further checks ?
 
        Ok(board)
    }
}