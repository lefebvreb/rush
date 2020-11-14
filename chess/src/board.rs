use std::fmt;

use crate::squares;
use crate::bitboard::BitBoard;
use crate::color::Color;
use crate::moves::Move;
use crate::piece::Piece;
use crate::square::Square;

/// Represent a complete chess board
#[derive(Clone, Debug)]
pub struct Board {
    bitboards: [[BitBoard; 6]; 2],
    mailbox: [Option<(Color, Piece)>; 64],
    occupancy: [BitBoard; 2],
}

//#################################################################################################
//
//                                    Implementation
//
//#################################################################################################

impl Board {
    // ===================================== Accessers =====================================

    /// Return the BitBoard associated to that Color and Piece
    #[inline]
    pub fn get_bitboard(&self, color: Color, piece: Piece) -> BitBoard {
        self.bitboards[color as usize][piece as usize]
    }

    /// Return the Piece and it's Color present on that Square 
    #[inline]
    pub fn get_piece(&self, square: Square) -> Option<(Color, Piece)> {
        self.mailbox[square as usize]
    }

    /// Return the occupancy BitBoard associated to that color
    #[inline]
    pub fn get_occupancy(&self, color: Color) -> BitBoard {
        self.occupancy[color as usize]
    }

    // ===================================== Helper =====================================

    // The three following methods were made with compiler optimisations
    // in mind. That is the reason why removing a piece from the board returns
    // it's color and type: so that when calling those methods together the
    // compiler will see it can bundle some bitwise operations.

    // Place a piece on the board
    #[inline(always)]
    fn set_piece(&mut self, color: Color, piece: Piece, on: Square) {
        self.mailbox[on as usize] = Some((color, piece));
        let bitboard = on.into();
        self.bitboards[color as usize][piece as usize] ^= bitboard;
        self.occupancy[color as usize] ^= bitboard;
    }

    // Remove a piece from the board and return it
    #[inline(always)]
    fn remove_piece(&mut self, on: Square) -> (Color, Piece) {
        let (color, piece) = self.mailbox[on as usize].expect("trying to remove nothing");
        self.mailbox[on as usize] = None;
        let bitboard = on.into();
        self.bitboards[color as usize][piece as usize] ^= bitboard;
        self.occupancy[color as usize] ^= bitboard;
        (color, piece)
    }

    // Remove a piece on the board, then remove it and
    #[inline(always)]
    fn reset_piece(&mut self, color: Color, piece: Piece, on: Square) -> (Color, Piece) {
        let swap = self.remove_piece(on);
        self.set_piece(color, piece, on);
        swap
    }

    // ===================================== do & undo =====================================

    /// Perform the move and modify the board accordingly
    #[inline]
    pub(crate) fn do_move(&mut self, color: Color, mv: Move) {
        match mv {
            Move::Quiet {from, to} => {
                let (color, piece) = self.remove_piece(from);
                self.set_piece(color, piece, to);
            }
            Move::Capture {from, to, ..} => {
                let (color, piece) = self.remove_piece(from);
                self.reset_piece(color, piece, to);
            }
            Move::Promote {from, to, promote} => {
                let (color, _) = self.remove_piece(from);
                self.set_piece(color, promote, to);
            }
            Move::PromoteCapture {from, to, promote, ..} => {
                let (color, _) = self.remove_piece(from);
                self.reset_piece(color, promote, to);
            }
            Move::EnPassant {from, to} => {
                let (color, piece) = self.remove_piece(from);
                self.remove_piece(Square::from((to.x(), from.y())));
                self.set_piece(color, piece, to);
            }
            Move::DoublePush {from, to} => {
                let (color, piece) = self.remove_piece(from);
                self.set_piece(color, piece, to);
            }
            Move::KingCastle => match color {
                Color::White => {
                    self.remove_piece(Square::H1);
                    self.set_piece(Color::White, Piece::Rook, Square::F1);
                    self.remove_piece(Square::E1);
                    self.set_piece(Color::White, Piece::King, Square::G1);
                }
                Color::Black => {
                    self.remove_piece(Square::H8);
                    self.set_piece(Color::Black, Piece::Rook, Square::F8);
                    self.remove_piece(Square::E8);
                    self.set_piece(Color::Black, Piece::King, Square::G8);
                }
            }
            Move::QueenCastle => match color {
                Color::White => {
                    self.remove_piece(Square::A1);
                    self.set_piece(Color::White, Piece::Rook, Square::C1);
                    self.remove_piece(Square::E1);
                    self.set_piece(Color::White, Piece::King, Square::B1);
                }
                Color::Black => {
                    self.remove_piece(Square::A8);
                    self.set_piece(Color::Black, Piece::Rook, Square::C8);
                    self.remove_piece(Square::E8);
                    self.set_piece(Color::Black, Piece::King, Square::B8);
                }
            }
        }
    }

    /// Perform the move in reverse and modify the board accordingly
    #[inline]
    pub(crate) fn undo_move(&mut self, color: Color, mv: Move) {
        match mv {
            Move::Quiet {from, to} => {
                let (color, piece) = self.remove_piece(to);
                self.set_piece(color, piece, from);
            }
            Move::Capture {from, to, capture} => {
                let (color, piece) = self.reset_piece(color.invert(), capture, to);
                self.set_piece(color, piece, from);
            }
            Move::Promote {from, to, ..} => {
                let (color, _) = self.remove_piece(to);
                self.set_piece(color, Piece::Pawn, from);
            }
            Move::PromoteCapture {from, to, capture, ..} => {
                let (color, _) = self.reset_piece(color.invert(), capture, to);
                self.set_piece(color, Piece::Pawn, from);
            }
            Move::EnPassant {from, to} => {
                let (color, piece) = self.remove_piece(to);
                self.set_piece(color, piece, from);
                self.set_piece(color.invert(), Piece::Pawn, Square::from((to.x(), from.y())));
            }
            Move::DoublePush {from, to} => {
                let (color, piece) = self.remove_piece(to);
                self.set_piece(color, piece, from);
            }
            Move::KingCastle => match color {
                Color::White => {
                    self.remove_piece(Square::F1);
                    self.set_piece(Color::White, Piece::Rook, Square::H1);
                    self.remove_piece(Square::G1);
                    self.set_piece(Color::White, Piece::King, Square::E1);
                }
                Color::Black => {
                    self.remove_piece(Square::F8);
                    self.set_piece(Color::Black, Piece::Rook, Square::H8);
                    self.remove_piece(Square::G8);
                    self.set_piece(Color::Black, Piece::King, Square::E8);
                }
            }
            Move::QueenCastle => match color {
                Color::White => {
                    self.remove_piece(Square::C1);
                    self.set_piece(Color::White, Piece::Rook, Square::A1);
                    self.remove_piece(Square::B1);
                    self.set_piece(Color::White, Piece::King, Square::E1);
                }
                Color::Black => {
                    self.remove_piece(Square::C8);
                    self.set_piece(Color::Black, Piece::Rook, Square::A8);
                    self.remove_piece(Square::B8);
                    self.set_piece(Color::Black, Piece::King, Square::E8);
                }
            }
        }
    }
}

impl Default for Board {
    /// Return a new Board with the default chess position
    #[cold]
    fn default() -> Board {
        let bitboards = [[ // White bitboards
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

        let mut mailbox = [None; 64];
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

        Board {bitboards, mailbox, occupancy}
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
                if let Some((color, piece)) = self.mailbox[x + 8*y] {
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
        assert_eq!(default.occupancy[0], board.occupancy[0]);
        assert_eq!(default.occupancy[1], board.occupancy[1]);
    }
}
