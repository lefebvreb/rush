use std::fmt;
use std::str::FromStr;

use crate::attacks;
use crate::bitboard::BitBoard;
use crate::castle_rights::CastleMask;
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

// The state of the board at a given turn.
#[derive(Clone, Debug)]
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
        self.colored[color.idx()]
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
    fullmove: u16,

    bitboards: [[BitBoard; 6]; 2],
    mailbox: [Option<(Color, Piece)>; 64],
    occ: Occupancy,

    state: StateInfo,
    prev_states: Vec<StateInfo>,
}

// ================================ pub impl

impl Board {
    /// Tries to parse the fen string into a board.
    pub fn from_fen(fen: &str) -> Result<Board, ParseFenError> {
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
    pub fn get_fullmove(&self) -> u16 {
        self.fullmove
    }

    /// Gets the bitboard corresponding to that color and piece type.
    #[inline]
    pub fn get_bitboard(&self, color: Color, piece: Piece) -> BitBoard {
        self.bitboards[color.idx()][piece.idx()]
    }

    /// Gets the (maybe) piece and it's color at that square.
    #[inline]
    pub fn get_piece(&self, sq: Square) -> Option<(Color, Piece)> {
        self.mailbox[sq.idx()]
    }

    /// Returns the occupancy object associated to that board.
    #[inline]
    pub fn get_occupancy(&self) -> &Occupancy {
        &self.occ
    }

    /// Clears the history of the board, making it impossible to 
    /// undo the previous moves but freeing a bit of memory.
    #[inline]
    pub fn clear_history(&mut self) {
        self.prev_states.clear()
    }

    // ================================ Methods

    // Returns the square the king of the side to move is occupying. 
    #[inline]
    pub fn king_sq(&self) -> Square {
        self.get_bitboard(self.get_side_to_move(), Piece::King).as_square_unchecked()
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
                // For every rook on that very same rank.
                for rook_sq in (self.get_bitboard(them, Piece::Rook) & rank).iter_squares() {
                    let between = BitBoard::between(king_sq, rook_sq);
                    // If the ep square is exactly between the king and the rook, 
                    // and there is nothing else than the two pawns, then it is an
                    // (incredibly rare) double pin.
                    if between.contains(ep_square) && between.count() == 2 {
                        return false;
                    }
                }
            }
        } else if from == self.king_sq() {
            let new_occ = self.get_occupancy().all() ^ (BitBoard::from(from) | BitBoard::from(to));
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
                match checkers.count() {
                    // One checker, the piece moving must either block or capture the enemy piece.
                    1 => {
                        let blocking_zone = BitBoard::between(self.king_sq(), checkers.as_square_unchecked());
                        verify!((blocking_zone | checkers).contains(to));
                    },
                    // Two checkers, the piece moving must be the king.
                    2 => return false,
                    _ => (),
                }
            }

            // Special case for pawns.
            if piece == Piece::Pawn {
                if mv.is_en_passant() {
                    // There must be an en passant square.
                    verify!(self.get_ep_square().is_some());
                    let ep_square = self.get_ep_square().unwrap();

                    // The ep square must be between the move's squares.
                    return ep_square == Square::from((to.x(), from.y())) &&
                        attacks::pawn(color, from).contains(to);
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
        // Store previous state and increment fullmove counter.
        self.prev_states.push(self.state.clone());
        if self.get_side_to_move() == Color::Black {
            self.fullmove += 1;
        }

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

    /// Undoes the move, reverting the board to it's previous state.
    pub fn undo_move(&mut self, mv: Move) {
        // Them color.
        let them = self.get_side_to_move();

        // Restore the previous state and decrement the fullmove counter.
        self.state = self.prev_states.pop().unwrap();
        if self.get_side_to_move() == Color::Black {
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
        if self.get_halfmove() < 3 {
            return false;
        }

        let cur_zobrist = self.state.zobrist;
        let nth_zobrist = |n: u8| {
            self.prev_states[self.prev_states.len() - usize::from(n)].zobrist
        };

        let mut other = !(cur_zobrist ^ nth_zobrist(1));

        for d in (3..=self.get_halfmove()).step_by(2) {
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
    pub fn parse_move(&self, s: &str) -> Result<Move, ParseFenError> {
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
                    _ => return Err(ParseFenError::new("unrecognized promotion")),
                };
    
                if let Some((_, capture)) = self.get_piece(to) {
                    Move::promote_capture(from, to, capture, promote)
                } else {
                    Move::promote(from, to, promote)
                }
            },
            _ => return Err(ParseFenError::new("a move should be encoded in pure algebraic coordinate notation")),
        };

        if self.is_pseudo_legal(mv) && self.is_legal(mv) {
            Ok(mv)
        } else {
            Err(ParseFenError::new("move is illegal in this context"))
        }
    }
}

// ================================ pub(crate) impl

impl Board {
    // Returns the castling rights in the current position.
    #[inline]
    pub(crate) fn get_castle_rights(&self) -> CastleRights {
        self.state.castle_rights
    }

    // Returns the en passant square of the current position.
    #[inline]
    pub(crate) fn get_ep_square(&self) -> EnPassantSquare {
        self.state.ep_square
    }

    // Returns true from and to are not aligned, or if the squares
    // between them are empty.
    #[inline]
    pub(crate) fn is_path_clear(&self, from: Square, to: Square) -> bool {
        (BitBoard::between(from, to) & self.occ.all).empty()
    }

    // Returns the type of the piece present at the given square.
    #[inline]
    pub(crate) fn piece_unchecked(&self, sq: Square) -> Piece {
        self.mailbox[sq.idx()].unwrap().1
    }

    // Returns the bitboard of all the attackers to that square. Does not take
    // en passant into account.
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
    // Places a piece of the given color on the given square. If ZOBRIST is true, 
    // updates the zobrist key accordingly.
    #[inline]
    fn place_piece<const ZOBRIST: bool>(&mut self, color: Color, piece: Piece, sq: Square) {
        self.mailbox[sq.idx()] = Some((color, piece));
        
        let mask = sq.into();
        self.bitboards[color.idx()][piece.idx()] ^= mask;
        self.occ.all ^= mask;
        self.occ.colored[color.idx()] ^= mask;

        if ZOBRIST {
            self.state.zobrist ^= Zobrist::from((color, piece, sq));
        }
    }

    // Removes the piece on the given square. If ZOBRIST is true, updates the
    // zobrist key accordingly.
    #[inline]
    fn remove_piece<const ZOBRIST: bool>(&mut self, sq: Square) -> (Color, Piece) {
        let (color, piece) = self.mailbox[sq.idx()].unwrap();
        self.mailbox[sq.idx()] = None;
        
        let mask = sq.into();
        self.bitboards[color.idx()][piece.idx()] ^= mask;
        self.occ.all ^= mask;
        self.occ.colored[color.idx()] ^= mask;

        if ZOBRIST {
            self.state.zobrist ^= Zobrist::from((color, piece, sq));
        }

        (color, piece)
    }

    // Dispalces a piece between the two given squares. If ZOBRIST is true, updates the
    // zobrist key accordingly.
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
            if (between & self.occ.all).count() == 1 {
                pinned |= between & occ_us;
            }
        }

        for sq in (self.get_bitboard(them, Piece::Bishop) | queens).iter_squares() {
            let between = BitBoard::between_diagonal(king_sq, sq);
            if (between & self.occ.all).count() == 1 {
                pinned |= between & occ_us;
            }
        }

        pinned
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
    /// Formats the board to it's fen representation: println!("{}", board);
    /// Use the # modifier to pretty-print the board: println!("{:#}", board);
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if f.alternate() {
            const CHARS: [[char; 6]; 2] = [
                ['♙', '♖', '♘', '♗', '♕', '♔'],
                ['♟', '♜', '♞', '♝', '♛', '♚'],
            ];

            writeln!(f, "  a b c d e f g h")?;
            for y in (0..8).rev() {
                write!(f, "{}", y+1)?;
                for x in 0..8 {
                    if let Some((color, piece)) = self.get_piece(Square::from((x, y))) {
                        write!(f, " {}", CHARS[color.idx()][piece.idx()])?;
                    } else {
                        write!(f, "  ")?;
                    }
                }
                if y != 0 {
                    writeln!(f)?;
                }
            }
        } else {
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
                        match color {
                            Color::White => write!(f, "{:#}", piece),
                            Color::Black => write!(f, "{}", piece),
                        }?;
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
                self.get_fullmove(),
            )?;
        }

        Ok(())
    }
}

impl<'a> FromStr for Board {
    type Err = ParseFenError;

    /// Tries to parse a board from a string in fen representation.
    fn from_str(s: &str) -> Result<Board, ParseFenError> {
        let mut split = s.split(' ');

        let mut next_arg = || split.next().ok_or(ParseFenError::new("not enough arguments in fen string"));

        let ranks = next_arg()?;

        let side_to_move = Color::from_str(next_arg()?)?;
        let castle_rights = CastleRights::from_str(next_arg()?)?;
        let ep_square = EnPassantSquare::from_str(next_arg()?)?;
        let halfmove = u8::from_str(next_arg()?)?;
        let fullmove = u16::from_str(next_arg()?)?;

        if split.next().is_some() {
            return Err(ParseFenError::new("too many arguments in fen string"));
        }

        let mut board = Board {
            fullmove,
            bitboards: [[BitBoard::EMPTY; 6]; 2],
            mailbox: [None; 64],
            occ: Occupancy::default(),
            state: StateInfo {
                side_to_move,
                halfmove,
                checkers: BitBoard::EMPTY,
                pinned: BitBoard::EMPTY,
                castle_rights,
                ep_square,
                zobrist: Zobrist::default(),
            },
            prev_states: Vec::new(),
        };

        let mut y = 0;
        let ranks = ranks.split('/');

        for rank in ranks {
            if y == 8 {
                return Err(ParseFenError::new("too many ranks in fen string"));
            }
            
            let mut x = 0;
            for c in rank.chars() {
                match c {
                    '1'..='8' => x += c as i8 - '1' as i8,
                    _ => {
                        let (color, piece) = Piece::from_char(c)?;
                        let sq = Square::from((x, 7 - y));
                        board.get_bitboard(Color::White, Piece::Pawn);
                        board.place_piece::<true>(color, piece, sq);
                    }
                }

                x += 1;
                if x > 8 {
                    return Err(ParseFenError::new("rank too large in fen string"));
                }
            }

            if x != 8 {
                return Err(ParseFenError::new("rank too small in fen string"));
            }
            y += 1;
        }

        if y != 8 {
            return Err(ParseFenError::new("not enough ranks in fen string"));
        }

        board.state.checkers = board.checkers();
        board.state.pinned = board.pinned();

        Ok(board)
    }
}