use std::fmt;

use crate::squares;
use crate::attacks::attacks;
use crate::bitboard::BitBoard;
use crate::color::Color;
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

#[derive(Clone, PartialEq, Debug)]
pub struct Occupancy {
    pub white: BitBoard,
    pub black: BitBoard,
    pub all: BitBoard,
    pub free: BitBoard,
}

impl Occupancy {
    #[inline(always)]
    pub fn update(&mut self, color: Color, mask: BitBoard) {
        match color {
            Color::White => self.white ^= mask,
            Color::Black => self.black ^= mask,
        }
        self.all ^= mask;
        self.free ^= mask;
    }

    #[inline(always)]
    pub fn by_color(&self, color: Color) -> BitBoard {
        match color {
            Color::White => self.white,
            Color::Black => self.black,
        }
    }
}

/// Represent the state of a single square, with the attack and defend maps for.
/// attack is the bitboard of the pieces attacking that square
/// defend is the bitboard of the squares attacked by the piece
#[repr(u8)]
#[derive(Clone, PartialEq, Debug)]
pub enum SquareInfo {
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

//#################################################################################################
//
//                                    Implementation
//
//#################################################################################################

impl Board {
    // ==================================== Accessers ======================================

    /// Return the BitBoard associated to that Color and Piece
    #[inline(always)]
    pub fn get_bitboard(&self, color: Color, piece: Piece) -> BitBoard {
        self.bitboards[color as usize][piece as usize]
    }

    /// Return the Piece and it's Color present on that Square 
    #[inline(always)]
    pub fn get_info(&self, square: Square) -> &SquareInfo {
        &self.mailbox[square as usize]
    }

    /// Return the occupancy BitBoard associated to that color
    #[inline(always)]
    pub fn get_occupancy(&self) -> &Occupancy {
        &self.occ
    }

    // ================================ Helper methods =====================================

    // The three following methods were made with compiler optimisations
    // in mind. That is the reason why removing a piece from the board returns
    // it's color and type: so that when calling those methods together the
    // compiler will see it can bundle some bitwise operations.

    // Place a piece on the board
    #[inline(always)]
    fn set_piece(&mut self, color: Color, piece: Piece, on: Square) {
        /*self.mailbox[on as usize] = Some((color, piece));
        let bitboard = on.into();
        self.bitboards[color as usize][piece as usize] ^= bitboard;
        self.occupancy[color as usize] ^= bitboard;*/
        todo!()
    }

    /// Remove a piece from the board and return it
    #[inline(always)]
    fn remove_piece(&mut self, on: Square) -> (Color, Piece) {
        /*let (color, piece) = self.mailbox[on as usize].expect("trying to remove nothing");
        self.mailbox[on as usize] = None;
        let bitboard = on.into();
        self.bitboards[color as usize][piece as usize] ^= bitboard;
        self.occupancy[color as usize] ^= bitboard;
        (color, piece)*/
        todo!()
    }

    /// Remove a piece on the board, then remove it and
    #[inline(always)]
    fn reset_piece(&mut self, color: Color, piece: Piece, on: Square) -> (Color, Piece) {
        /*let swap = self.remove_piece(on);
        self.set_piece(color, piece, on);
        swap*/
        todo!()
    }

    /*#[inline(always)]
    fn get_color_piece_unchecked(&self, square: Square) -> (Color, Piece) {
        match self.mailbox[square as usize] {
            SquareInfo::Occupied {color, piece, ..} => (color, piece),
            _ => unsafe {unreachable_unchecked()}
        }
    }

    #[inline(always)]
    fn update_attackers(&mut self, attack: BitBoard) {
        for from_bitboard in attack.iter_bitboards() {
            let from = from_bitboard.least_significant_bit();

            let (color, piece, defend, to_update) = match self.mailbox[from as usize] {
                SquareInfo::Occupied {color, piece, ref mut defend, ..} => {
                    let new_defend = attacks(color, piece, from, &self.occupancy);
                    let to_update = *defend ^ new_defend;
                    *defend = new_defend;
                    (color, piece, defend, to_update)
                },
                _ => unsafe {unreachable_unchecked()},
            };

            for to in to_update.iter_squares() {
                match self.mailbox[to as usize] {
                    SquareInfo::Occupied {ref mut attack, ..} => *attack ^= from_bitboard,
                    SquareInfo::Unoccupied {ref mut attack} => *attack ^= from_bitboard,
                }
            }
        }
    }*/

    /// Update the attack map of the square sq with the given bitboard
    #[inline(always)]
    fn update_attacks(&mut self, sq: Square, mask: BitBoard) {
        match self.mailbox[sq as usize] {
            SquareInfo::Unoccupied {ref mut attack, ..} | 
            SquareInfo::Occupied {ref mut attack, ..} => {
                *attack ^= mask;
            }
        }
    }

    /// Update the attacks of the piece on sq (if there are any) and
    /// also update the squares attacked by that piece
    #[inline(always)]
    fn update_attacker(&mut self, sq: Square) {
        match self.mailbox[sq as usize] {
            SquareInfo::Occupied {color, piece, ref mut defend, ..} => {
                let mut diff = *defend;
                *defend = attacks(color, piece, sq, &self.occ);
                diff ^= *defend;

                let mask = sq.into();
                for sq in diff.iter_squares() {
                    self.update_attacks(sq, mask);
                }
            }
            _ => ()
        }
    }

    #[inline(always)]
    fn get_piece(&self, square: Square) -> Piece {
        match self.mailbox[square as usize] {
            SquareInfo::Occupied {piece, ..} => piece,
            _ => unreachable!()
        }
    }

    #[inline(always)]
    fn update_bitboards(&mut self, color: Color, piece: Piece, mask: BitBoard) {
        self.bitboards[color as usize][piece as usize] ^= mask;
        self.occ.update(color, mask);
    }

    #[inline(always)]
    fn occupy_mailbox(&mut self, color: Color, piece: Piece, sq: Square) {
        self.mailbox[sq as usize] = match self.mailbox[sq as usize] {
            SquareInfo::Occupied {attack, defend, ..} => SquareInfo::Occupied {
                color,
                piece,
                attack,
                defend,
            },
            SquareInfo::Unoccupied {attack} =>  SquareInfo::Occupied {
                color,
                piece,
                attack,
                defend: BitBoard(0),
            },
        };
    }

    #[inline(always)]
    fn unoccupy_mailbox(&mut self, sq: Square) {
        let attack = match self.mailbox[sq as usize] {
            SquareInfo::Occupied {attack, defend, ..} => {
                let mask = sq.into();
                for sq in defend.iter_squares() {
                    self.update_attacks(sq, mask);
                }
                
                self.mailbox[sq as usize] = SquareInfo::Unoccupied {attack};
            }
            _ => unreachable!()
        };
    }

    #[inline(always)]
    fn update_occupied(&mut self, sq: Square) {
        match self.mailbox[sq as usize] {
            SquareInfo::Occupied {piece, color, attack, ref mut defend} => {
                *defend = attacks(color, piece, sq, &self.occ);

                let mask = sq.into();
                for sq in defend.iter_squares() {
                    self.update_attacks(sq, mask);
                }

                for sq in attack.iter_squares() {
                    self.update_attacker(sq);
                }
            }
            _ => unreachable!()
        }
    }

    #[inline(always)]
    fn update_unoccupied(&mut self, sq: Square) {
        match self.mailbox[sq as usize] {
            SquareInfo::Unoccupied {attack} => {
                for sq in attack.iter_squares() {
                    self.update_attacker(sq);
                }
            }
            _ => unreachable!()
        }
    }

    #[inline(always)]
    fn castle(&mut self, color: Color, rook_from: Square, rook_to: Square, king_from: Square, king_to: Square) {
        self.update_bitboards(color, Piece::Rook, squares!(rook_from, rook_to));
        self.update_bitboards(color, Piece::King, squares!(king_from, king_to));

        self.unoccupy_mailbox(rook_from);
        self.unoccupy_mailbox(king_from);
        self.occupy_mailbox(color, Piece::Rook, rook_to);
        self.occupy_mailbox(color, Piece::King, king_to);

        self.update_unoccupied(rook_from);
        self.update_unoccupied(king_from);
        self.update_occupied(rook_to);
        self.update_occupied(king_to);
    }

    // ===================================== do & undo =====================================

    /// Perform the move and modify the board accordingly
    #[inline]
    pub(crate) fn do_move(&mut self, color: Color, mv: Move) {
        match mv {
            Move::Quiet {from, to} => {
                let piece = self.get_piece(from);

                self.update_bitboards(color, piece, squares!(from, to));

                self.unoccupy_mailbox(from);
                self.occupy_mailbox(color, piece, to);

                self.update_unoccupied(from);
                self.update_occupied(to);
            }
            Move::Capture {from, to, capture} => {
                let piece = self.get_piece(from);

                self.update_bitboards(color, piece, squares!(from, to));
                self.update_bitboards(color.invert(), capture, to.into());

                self.unoccupy_mailbox(from);
                self.occupy_mailbox(color, piece, to);

                self.update_unoccupied(from);
                self.update_occupied(to);
            }
            Move::Promote {from, to, promote} => {
                let piece = self.get_piece(from);

                self.update_bitboards(color, piece, from.into());
                self.update_bitboards(color, promote, to.into());

                self.unoccupy_mailbox(from);
                self.occupy_mailbox(color, promote, to);

                self.update_unoccupied(from);
                self.update_occupied(to);
            }
            Move::PromoteCapture {from, to, capture, promote} => {
                let piece = self.get_piece(from);

                self.update_bitboards(color, piece, from.into());
                self.update_bitboards(color, promote, to.into());
                self.update_bitboards(color.invert(), capture, to.into());

                self.unoccupy_mailbox(from);
                self.occupy_mailbox(color, promote, to);

                self.update_unoccupied(from);
                self.update_occupied(to);
            }
            Move::EnPassant {from, to} => {
                let mid = Square::from((to.x(), from.y()));

                self.update_bitboards(color, Piece::Pawn, squares!(from, to));
                self.update_bitboards(color.invert(), Piece::Pawn, to.into());

                self.unoccupy_mailbox(from);
                self.unoccupy_mailbox(mid);
                self.occupy_mailbox(color, Piece::Pawn, to);

                self.update_unoccupied(from);
                self.update_unoccupied(mid);
                self.update_occupied(to);
            }
            Move::DoublePush {from, to} => {
                self.update_bitboards(color, Piece::Pawn, squares!(from, to));

                self.unoccupy_mailbox(from);
                self.occupy_mailbox(color, Piece::Pawn, to);

                self.update_unoccupied(from);
                self.update_occupied(to);
            }
            Move::KingCastle => match color {
                Color::White => self.castle(Color::White, Square::H1, Square::F1, Square::E1, Square::G1),
                Color::Black => self.castle(Color::White, Square::H8, Square::F8, Square::E8, Square::G8),
            }
            Move::QueenCastle => match color {
                Color::White => self.castle(Color::White, Square::A1, Square::C1, Square::E1, Square::B1),
                Color::Black => self.castle(Color::White, Square::A8, Square::C8, Square::E8, Square::B8),
            }
            _ => (),
        }
    }

    /// Perform the move in reverse and modify the board accordingly
    #[inline]
    pub(crate) fn undo_move(&mut self, color: Color, mv: Move) {
        match mv {
            Move::Quiet {from, to} => {
                let piece = self.get_piece(to);

                self.update_bitboards(color, piece, squares!(from, to));

                self.occupy_mailbox(color, piece, from);
                self.unoccupy_mailbox(to);

                self.update_occupied(from);
                self.update_unoccupied(to);
            }
            Move::Capture {from, to, capture} => {
                let piece = self.get_piece(to);

                self.update_bitboards(color, piece, squares!(from, to));
                self.update_bitboards(color.invert(), capture, to.into());

                self.occupy_mailbox(color, piece, from);
                self.occupy_mailbox(color.invert(), capture, to);

                self.update_occupied(from);
                self.update_occupied(to);
            }
            Move::Promote {from, to, promote} => {
                self.update_bitboards(color, Piece::Pawn, from.into());
                self.update_bitboards(color, promote, to.into());

                self.occupy_mailbox(color, Piece::Pawn, from);
                self.unoccupy_mailbox(to);

                self.update_occupied(from);
                self.update_unoccupied(to);
            }
            Move::PromoteCapture {from, to, capture, promote} => {
                self.update_bitboards(color, Piece::Pawn, from.into());
                self.update_bitboards(color, promote, to.into());
                self.update_bitboards(color.invert(), capture, to.into());

                self.occupy_mailbox(color.invert(), capture, to);
                self.occupy_mailbox(color, Piece::Pawn, from);

                self.update_occupied(from);
                self.update_occupied(to);
            }
            Move::EnPassant {from, to} => {
                let mid = Square::from((to.x(), from.y()));

                self.update_bitboards(color, Piece::Pawn, squares!(from, to));
                self.update_bitboards(color.invert(), Piece::Pawn, mid.into());

                self.occupy_mailbox(color, Piece::Pawn, from);
                self.occupy_mailbox(color.invert(), Piece::Pawn, mid);
                self.unoccupy_mailbox(to);

                self.update_occupied(from);
                self.update_occupied(mid);
                self.update_unoccupied(to);
            }
            Move::DoublePush {from, to} => {
                self.update_bitboards(color, Piece::Pawn, squares!(from, to));

                self.occupy_mailbox(color, Piece::Pawn, from);
                self.unoccupy_mailbox(to);

                self.update_occupied(from);
                self.update_unoccupied(to);
            }
            Move::KingCastle => match color {
                Color::White => self.castle(Color::White, Square::F1, Square::H1, Square::G1, Square::E1),
                Color::Black => self.castle(Color::White, Square::F8, Square::H8, Square::G8, Square::E8),
            }
            Move::QueenCastle => match color {
                Color::White => self.castle(Color::White, Square::C1, Square::A1, Square::B1, Square::E1),
                Color::Black => self.castle(Color::White, Square::C8, Square::A8, Square::B8, Square::E8),
            }
            _ => (),
        }
    }
}

impl Default for Board {
    /// Return a new Board with the default chess position
    #[cold]
    fn default() -> Board {
        /*let bitboards = [[ // White bitboards
                BitBoard::RANK_2,                 // Pawns
                squares!(Square::A1, Square::H1), // Rooks
                squares!(Square::B1, Square::G1), // Knights
                squares!(Square::C1, Square::F1), // Bishops
                squares!(Square::D1),             // Queen
                squares!(Square::E1),             // King
            ], [           // Black bitboards
                BitBoard::RANK_7,                 // Pawns
                squares!(Square::A8, Square::H8), // Rooks
                squares!(Square::B8, Square::G8), // Knights
                squares!(Square::C8, Square::F8), // Bishops
                squares!(Square::D8),             // Queen
                squares!(Square::E8),             // King
        ]];

        let mut mailbox = [SquareInfo::Unoccupied {defend: BitBoard(0)}; 64];
        let mut occupancy = [BitBoard(0); 2];
        
        for color in &Color::COLORS {
            for piece in &Piece::PIECES {
                let bitboard = bitboards[*color as usize][*piece as usize];

                for square in bitboard.iter_squares() {
                    mailbox[square as usize] = Some((*color, *piece));
                }
                occupancy[*color as usize] |= bitboard;
            }
        }

        Board {bitboards, mailbox, occupancy}*/
        todo!()
    }
}

//#################################################################################################
//
//                                         Format
//
//#################################################################################################

impl fmt::Display for Board {
    #[cold]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const CHARS: [[char; 6]; 2] = [
            ['♙', '♖', '♘', '♗', '♕', '♔'],
            ['♟', '♜', '♞', '♝', '♛', '♚'],
        ];

        writeln!(f, "  a b c d e f g h").unwrap();
        for y in (0..8).rev() {
            write!(f, "{} ", y+1).unwrap();
            for x in 0..8 {
                if let SquareInfo::Occupied {piece, color, ..} = self.mailbox[x + 8*y] {
                    write!(f, "{} ", CHARS[color as usize][piece as usize]).unwrap();
                } else {
                    write!(f, "- ").unwrap();
                }
            }
            writeln!(f).unwrap();
        }

        Ok(())
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
    fn moves() {
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
            Move::KingCastle,
            Move::Quiet {from: Square::E2, to: Square::E5},
            Move::DoublePush {from: Square::B7, to: Square::B5},
            Move::QueenCastle,
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

            println!("{:?}", mv);
            println!("{}", board);
        }

        for mv in moves.iter().rev() {
            color = color.invert();
            board.undo_move(color, *mv);

            println!("reverse {:?}", mv);
            println!("{}", board);
        }

        let default = Board::default();
        
        for i in 0..64 {
            assert_eq!(default.mailbox[i], board.mailbox[i]);
        }
        for color in &Color::COLORS {
            for piece in &Piece::PIECES {
                assert_eq!(
                    default.bitboards[*color as usize][*piece as usize],
                    board.bitboards[*color as usize][*piece as usize],
                )
            }
        }
        assert_eq!(default.occ, board.occ);
    }
}
