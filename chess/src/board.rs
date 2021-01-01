use std::fmt;
use std::str::FromStr;

use crate::attacks::attacks;
use crate::bitboard::BitBoard;
use crate::color::Color;
use crate::errors::ParseFenError;
use crate::moves::Move;
use crate::piece::Piece;
use crate::square::Square;

/// Represent a complete chess board
#[derive(Clone, Debug)]
pub struct Board {
    bitboards: [[BitBoard; 6]; 2],
    mailbox: [SquareInfo; 64],
    occ: Occupancy,
}

// A struct holding all necessary occupancy informations
#[derive(Clone, Debug)]
struct Occupancy {
    white: BitBoard,
    black: BitBoard,
    all: BitBoard,
    free: BitBoard,
}

impl Occupancy {
    /// Update the occupancy according to the given color and mask
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

/// Represent the state of a single square, with the attack and defend maps for.
/// attack is the bitboard of the pieces attacking that square
/// defend is the bitboard of the squares attacked by the piece
#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Debug)]
enum SquareInfo {
    Occupied {
        piece: Piece,
        color: Color,
        attack: BitBoard,
        defend: BitBoard,
    },
    Unoccupied {
        attack: BitBoard,
    },
}

impl Board {
    // ==================================== Accessers ======================================

    /// Return the BitBoard associated to that Color and Piece
    #[inline(always)]
    pub fn get_bitboard(&self, color: Color, piece: Piece) -> BitBoard {
        self.bitboards[color as usize][piece as usize]
    }

    /// Return the monochrome occupancy of the board
    #[inline(always)]
    pub fn get_occupancy(&self) -> BitBoard {
        self.occ.all
    }

    /// Return the occupancy of the board corresponding to the given color
    #[inline(always)]
    pub fn get_color_occupancy(&self, color: Color) -> BitBoard {
        match color {
            Color::White => self.occ.white,
            Color::Black => self.occ.black,
        }
    }

    /// Return the free squares of the board
    #[inline(always)]
    pub fn get_free(&self) -> BitBoard {
        self.occ.free
    }

    /// Return the attacks to that square
    #[inline(always)]
    pub fn get_attacks(&self, sq: Square) -> BitBoard {
        match self.mailbox[sq as usize] {
            SquareInfo::Occupied {attack, ..} |
            SquareInfo::Unoccupied {attack} => attack,
        }
    }

    /// Return true if the square sq is occupied
    #[inline(always)]
    pub fn is_occupied(&self, sq: Square) -> bool {
        match self.mailbox[sq as usize] {
            SquareInfo::Occupied {..} => true,
            _ => false,
        }
    }

    /// Return true if the square sq is empty
    #[inline(always)]
    pub fn is_empty(&self, sq: Square) -> bool {
        match self.mailbox[sq as usize] {
            SquareInfo::Unoccupied {..} => true,
            _ => false,
        }
    }

    /// Return, if it exists, the piece and it's color present on that square
    pub fn get_piece(&self, sq: Square) -> Option<(Color, Piece)> {
        match self.mailbox[sq as usize] {
            SquareInfo::Occupied {color, piece, ..} => Some((color, piece)),
            _ => None,
        }
    }

    // ================================ crate accessers =====================================

    // Return the attacks from that square, assuming there is a piece there
    #[inline(always)]
    pub(crate) fn get_defend_unchecked(&self, sq: Square) -> BitBoard {
        match self.mailbox[sq as usize] {
            SquareInfo::Occupied {defend, ..} => defend,
            _ => unreachable!(),
        }
    }

    // Return the piece present at the given square, should not be called when there are no pieces there
    #[inline(always)]
    pub(crate) fn get_piece_unchecked(&self, square: Square) -> Piece {
        match self.mailbox[square as usize] {
            SquareInfo::Occupied {piece, ..} => piece,
            _ => unreachable!()
        }
    }

    // ================================ Helper methods =====================================

    // Update the attack map of the square sq with the given bitboard
    #[inline(always)]
    fn update_attacks(&mut self, sq: Square, mask: BitBoard) {
        match self.mailbox[sq as usize] {
            SquareInfo::Unoccupied {ref mut attack, ..} | 
            SquareInfo::Occupied {ref mut attack, ..} => {
                *attack ^= mask;
            }
        }
    }

    // Update the attacks of the piece on sq (if there are any) and
    // also update the squares attacked by that piece
    #[inline(always)]
    fn update_attacker(&mut self, sq: Square, updated: &mut BitBoard) {
        let mask = sq.into();

        if (*updated & mask).is_empty() {
            *updated |= mask;

            match self.mailbox[sq as usize] {
                SquareInfo::Occupied {color, piece, ref mut defend, ..} => {
                    let mut diff = *defend;
                    *defend = attacks(color, piece, sq, self.occ.all);
                    diff ^= *defend;
    
                    let mask = sq.into();
                    for sq in diff.iter_squares() {
                        self.update_attacks(sq, mask);
                    }
                }
                _ => ()
            }
        }
    }

    // Update all bitboards with the given mask, color and piece
    #[inline(always)]
    fn update_bitboards(&mut self, color: Color, piece: Piece, mask: BitBoard) {
        self.bitboards[color as usize][piece as usize] ^= mask;
        self.occ.update(color, mask);
    }

    // Fill a mailbox slot with a new piece
    #[inline(always)]
    fn occupy_mailbox(&mut self, color: Color, piece: Piece, sq: Square) {
        let mailbox = &mut self.mailbox[sq as usize];

        let attack = match *mailbox {
            SquareInfo::Unoccupied {attack} => attack,
            _ => unreachable!(),
        };

        *mailbox = SquareInfo::Occupied {
            color,
            piece,
            attack,
            defend: BitBoard::EMPTY,
        };
    }

    // Replace the previous occupant of that mailbox slot with a new one
    #[inline(always)]
    fn reoccupy_mailbox(&mut self, color: Color, piece: Piece, sq: Square) {
        let mailbox = &mut self.mailbox[sq as usize];

        let (attack, defend) = match *mailbox {
            SquareInfo::Occupied {attack, defend, ..} => (attack, defend),
            _ => unreachable!(),
        };

        *mailbox = SquareInfo::Occupied {
            color,
            piece,
            attack,
            defend,
        };
    }

    // Empty a slot of the mailbox, discarding it's defend map and updating the attackers'
    #[inline(always)]
    fn unoccupy_mailbox(&mut self, sq: Square) {
        match self.mailbox[sq as usize] {
            SquareInfo::Occupied {attack, defend, ..} => {
                let mask = sq.into();
                for sq in defend.iter_squares() {
                    self.update_attacks(sq, mask);
                }
                
                self.mailbox[sq as usize] = SquareInfo::Unoccupied {attack};
            }
            _ => unreachable!()
        }
    }

    // Updates the attack and defend maps of an occupied Square
    #[inline(always)]
    fn update_occupied(&mut self, sq: Square, updated: &mut BitBoard) {
        match self.mailbox[sq as usize] {
            SquareInfo::Occupied {piece, color, attack, ref mut defend} => {
                self.update_attacker(sq, updated);

                for sq in attack.iter_squares() {
                    self.update_attacker(sq, updated);
                }
            }
            _ => unreachable!()
        }
    }

    // Update the attack map of an unoccupied Square
    #[inline(always)]
    fn update_unoccupied(&mut self, sq: Square, updated: &mut BitBoard) {
        match self.mailbox[sq as usize] {
            SquareInfo::Unoccupied {attack} => {
                for sq in attack.iter_squares() {
                    self.update_attacker(sq, updated);
                }
            }
            _ => unreachable!()
        }
    }

    // Moves a king and a rook the proper way
    #[inline(always)]
    fn castle(&mut self, color: Color, rook_from: Square, rook_to: Square, king_from: Square, king_to: Square) {
        self.update_bitboards(color, Piece::Rook, squares!(rook_from, rook_to));
        self.update_bitboards(color, Piece::King, squares!(king_from, king_to));

        self.unoccupy_mailbox(rook_from);
        self.unoccupy_mailbox(king_from);
        self.occupy_mailbox(color, Piece::Rook, rook_to);
        self.occupy_mailbox(color, Piece::King, king_to);

        let mut updated = BitBoard::EMPTY;
        self.update_unoccupied(rook_from, &mut updated);
        self.update_unoccupied(king_from, &mut updated);
        self.update_occupied(rook_to, &mut updated);
        self.update_occupied(king_to, &mut updated);
    }

    // ===================================== do & undo =====================================

    /// Perform the move and modify the board accordingly
    #[inline]
    pub(crate) fn do_move(&mut self, color: Color, mv: Move) {
        match mv {
            Move::Quiet {from, to} => {
                let piece = self.get_piece_unchecked(from);

                self.update_bitboards(color, piece, squares!(from, to));

                self.unoccupy_mailbox(from);
                self.occupy_mailbox(color, piece, to);

                let mut updated = BitBoard::EMPTY;
                self.update_unoccupied(from, &mut updated);
                self.update_occupied(to, &mut updated);
            }
            Move::Capture {from, to, capture} => {
                let piece = self.get_piece_unchecked(from);

                self.update_bitboards(color, piece, squares!(from, to));
                self.update_bitboards(color.invert(), capture, to.into());

                self.unoccupy_mailbox(from);
                self.reoccupy_mailbox(color, piece, to);

                let mut updated = BitBoard::EMPTY;
                self.update_unoccupied(from, &mut updated);
                self.update_occupied(to, &mut updated);
            }
            Move::Promote {from, to, promote} => {
                let piece = self.get_piece_unchecked(from);

                self.update_bitboards(color, piece, from.into());
                self.update_bitboards(color, promote, to.into());

                self.unoccupy_mailbox(from);
                self.occupy_mailbox(color, promote, to);

                let mut updated = BitBoard::EMPTY;
                self.update_unoccupied(from, &mut updated);
                self.update_occupied(to, &mut updated);
            }
            Move::PromoteCapture {from, to, capture, promote} => {
                let piece = self.get_piece_unchecked(from);

                self.update_bitboards(color, piece, from.into());
                self.update_bitboards(color, promote, to.into());
                self.update_bitboards(color.invert(), capture, to.into());

                self.unoccupy_mailbox(from);
                self.reoccupy_mailbox(color, promote, to);

                let mut updated = BitBoard::EMPTY;
                self.update_unoccupied(from, &mut updated);
                self.update_occupied(to, &mut updated);
            }
            Move::EnPassant {from, to} => {
                let mid = Square::from((to.x(), from.y()));

                self.update_bitboards(color, Piece::Pawn, squares!(from, to));
                self.update_bitboards(color.invert(), Piece::Pawn, mid.into());

                self.unoccupy_mailbox(from);
                self.unoccupy_mailbox(mid);
                self.occupy_mailbox(color, Piece::Pawn, to);

                let mut updated = BitBoard::EMPTY;
                self.update_unoccupied(from, &mut updated);
                self.update_unoccupied(mid, &mut updated);
                self.update_occupied(to, &mut updated);
            }
            Move::DoublePush {from, to} => {
                self.update_bitboards(color, Piece::Pawn, squares!(from, to));

                self.unoccupy_mailbox(from);
                self.occupy_mailbox(color, Piece::Pawn, to);

                let mut updated = BitBoard::EMPTY;
                self.update_unoccupied(from, &mut updated);
                self.update_occupied(to, &mut updated);
            }
            Move::KingCastle {..} => match color {
                Color::White => self.castle(Color::White, Square::H1, Square::F1, Square::E1, Square::G1),
                Color::Black => self.castle(Color::Black, Square::H8, Square::F8, Square::E8, Square::G8),
            }
            Move::QueenCastle {..} => match color {
                Color::White => self.castle(Color::White, Square::A1, Square::C1, Square::E1, Square::B1),
                Color::Black => self.castle(Color::Black, Square::A8, Square::C8, Square::E8, Square::B8),
            }
            _ => (),
        }
    }

    // Perform the move in reverse and modify the board accordingly
    #[inline]
    pub(crate) fn undo_move(&mut self, color: Color, mv: Move) {
        match mv {
            Move::Quiet {from, to} => {
                let piece = self.get_piece_unchecked(to);

                self.update_bitboards(color, piece, squares!(from, to));

                self.occupy_mailbox(color, piece, from);
                self.unoccupy_mailbox(to);

                let mut updated = BitBoard::EMPTY;
                self.update_occupied(from, &mut updated);
                self.update_unoccupied(to, &mut updated);
            }
            Move::Capture {from, to, capture} => {
                let piece = self.get_piece_unchecked(to);

                self.update_bitboards(color, piece, squares!(from, to));
                self.update_bitboards(color.invert(), capture, to.into());

                self.occupy_mailbox(color, piece, from);
                self.reoccupy_mailbox(color.invert(), capture, to);

                let mut updated = BitBoard::EMPTY;
                self.update_occupied(from, &mut updated);
                self.update_occupied(to, &mut updated);
            }
            Move::Promote {from, to, promote} => {
                self.update_bitboards(color, Piece::Pawn, from.into());
                self.update_bitboards(color, promote, to.into());

                self.occupy_mailbox(color, Piece::Pawn, from);
                self.unoccupy_mailbox(to);

                let mut updated = BitBoard::EMPTY;
                self.update_occupied(from, &mut updated);
                self.update_unoccupied(to, &mut updated);
            }
            Move::PromoteCapture {from, to, capture, promote} => {
                self.update_bitboards(color, Piece::Pawn, from.into());
                self.update_bitboards(color, promote, to.into());
                self.update_bitboards(color.invert(), capture, to.into());

                self.reoccupy_mailbox(color.invert(), capture, to);
                self.occupy_mailbox(color, Piece::Pawn, from);

                let mut updated = BitBoard::EMPTY;
                self.update_occupied(from, &mut updated);
                self.update_occupied(to, &mut updated);
            }
            Move::EnPassant {from, to} => {
                let mid = Square::from((to.x(), from.y()));

                self.update_bitboards(color, Piece::Pawn, squares!(from, to));
                self.update_bitboards(color.invert(), Piece::Pawn, mid.into());

                self.occupy_mailbox(color, Piece::Pawn, from);
                self.occupy_mailbox(color.invert(), Piece::Pawn, mid);
                self.unoccupy_mailbox(to);

                let mut updated = BitBoard::EMPTY;
                self.update_occupied(from, &mut updated);
                self.update_occupied(mid, &mut updated);
                self.update_unoccupied(to, &mut updated);
            }
            Move::DoublePush {from, to} => {
                self.update_bitboards(color, Piece::Pawn, squares!(from, to));

                self.occupy_mailbox(color, Piece::Pawn, from);
                self.unoccupy_mailbox(to);

                let mut updated = BitBoard::EMPTY;
                self.update_occupied(from, &mut updated);
                self.update_unoccupied(to, &mut updated);
            }
            Move::KingCastle {..} => match color {
                Color::White => self.castle(Color::White, Square::F1, Square::H1, Square::G1, Square::E1),
                Color::Black => self.castle(Color::Black, Square::F8, Square::H8, Square::G8, Square::E8),
            }
            Move::QueenCastle {..} => match color {
                Color::White => self.castle(Color::White, Square::C1, Square::A1, Square::B1, Square::E1),
                Color::Black => self.castle(Color::Black, Square::C8, Square::A8, Square::B8, Square::E8),
            }
            _ => (),
        }
    }

    /// Pretty-prints the board to stdout, using utf-8 characters
    /// to represent the pieces
    pub fn pretty_print(&self) {
        const CHARS: [[char; 6]; 2] = [
            ['♙', '♖', '♘', '♗', '♕', '♔'],
            ['♟', '♜', '♞', '♝', '♛', '♚'],
        ];

        println!("  a b c d e f g h");
        for y in (0..8).rev() {
            print!("{} ", y + 1);
            for x in 0..8 {
                if let SquareInfo::Occupied {piece, color, ..} = self.mailbox[x + 8*y] {
                    print!("{} ", CHARS[color as usize][piece as usize]);
                } else {
                    print!("- ");
                }
            }
            println!()
        }
    }
}

impl Default for Board {
    // Return a new Board with the default chess position
    fn default() -> Board {
        Board::from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR").unwrap()
    }
}

impl fmt::Display for Board {
    // Give the FEN notation of the board
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
                } else {
                    streak += 1;
                }
            }

            write_if_not_zero!(streak);
            if y != 0 {
                write!(f, "/")?;
            }
        }

        Ok(())
    }
}

impl FromStr for Board {
    type Err = ParseFenError;

    // Try to construct a board from a fen notation
    fn from_str(s: &str) -> Result<Board, ParseFenError> {
        let ranks = s.split("/").into_iter().collect::<Vec<_>>();

        if ranks.len() != 8 {
            return Err(ParseFenError::new(format!("not enough ranks in FEN board {:?}", s)));
        }

        let mut board = Board {
            bitboards: [[BitBoard::EMPTY; 6]; 2],
            mailbox: [SquareInfo::Unoccupied {attack: BitBoard::EMPTY}; 64],
            occ: Occupancy {
                white: BitBoard::EMPTY,
                black: BitBoard::EMPTY,
                all: BitBoard::EMPTY,
                free: BitBoard::FULL,
            }
        };

        for (i, rank) in ranks.iter().enumerate() {
            let mut j = 0;

            for c in rank.chars() {
                match c {
                    '1'..='8' => j += c as u8 - '1' as u8,
                    _ => {
                        let (color, piece) = Piece::from_char(c)?;
                        let sq = Square::from(j + (7 - i as u8) * 8);
    
                        board.update_bitboards(color, piece, sq.into());
                        board.occupy_mailbox(color, piece, sq);

                        j += 1;
                    }
                }

                if j > 8 {
                    return Err(ParseFenError::new(format!("rank #{} is too large in FEN board {:?}", i, s)))
                }
            }
        }

        let mut updated = BitBoard::EMPTY;
        for sq in &Square::SQUARES {
            if board.is_occupied(*sq) {
                board.update_occupied(*sq, &mut updated);
            }
        }

        Ok(board)
    }
}

//#################################################################################################
//
//                                            Tests
//
//#################################################################################################

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn do_and_undo_moves() {
        let mut board = Board::default();
        let mut color = Color::White;

        // List of moves to do and then undo
        let moves = vec![
            Move::DoublePush {from: Square::D2, to: Square::D4},
            Move::Quiet {from: Square::B8, to: Square::C6},
            Move::Quiet {from: Square::D4, to: Square::D5},
            Move::Quiet {from: Square::G7, to: Square::G6},
            Move::Quiet {from: Square::C1, to: Square::H6},
            Move::Capture {from: Square::F8, to: Square::H6, capture: Piece::Bishop},
            Move::Quiet {from: Square::D1, to: Square::D3},
            Move::DoublePush {from: Square::E7, to: Square::E5},
            Move::EnPassant {from: Square::D5, to: Square::E6},
            Move::Quiet {from: Square::G8, to: Square::F6},
            Move::Quiet {from: Square::B1, to: Square::C3},
            Move::KingCastle {color: Color::Black},
            Move::Quiet {from: Square::E2, to: Square::E5},
            Move::DoublePush {from: Square::B7, to: Square::B5},
            Move::QueenCastle {color: Color::White},
            Move::Quiet {from: Square::B5, to: Square::B4},
            Move::Capture {from: Square::E6, to: Square::D7, capture: Piece::Pawn},
            Move::Quiet {from: Square::B4, to: Square::B3},
            Move::PromoteCapture {from: Square::D7, to: Square::C8, capture: Piece::Bishop, promote: Piece::Knight},
            Move::Capture {from: Square::B3, to: Square::A2, capture: Piece::Pawn},
            Move::Quiet {from: Square::C8, to: Square::B6},
            Move::Promote {from: Square::A2, to: Square::A1, promote: Piece::Queen},
        ];

        for mv in moves.iter() {
            board.do_move(color, *mv);
            color = color.invert();
        }

        for mv in moves.iter().rev() {
            color = color.invert();
            board.undo_move(color, *mv);
        }

        let default = Board::default();
        
        // Compare new board with the default one, they should be identical
        for i in 0..64 {
            assert_eq!(default.mailbox[i], board.mailbox[i]);
        }
        for i in 0..2 {
            for j in 0..6 {
                assert_eq!(
                    default.bitboards[i][j],
                    board.bitboards[i][j],
                )
            }
        }
        assert_eq!(default.occ.white, board.occ.white);
        assert_eq!(default.occ.black, board.occ.black);
        assert_eq!(default.occ.all, board.occ.all);
        assert_eq!(default.occ.free, board.occ.free);
    }

}
