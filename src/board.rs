use std::fmt;

use crate::squares;
use crate::bitboard::BitBoard;
use crate::color::Color;
use crate::piece::Piece;
use crate::square::Square;

/// Represent a complete chess board
#[derive(Clone, Debug)]
pub struct Board {
    bitboards: [[BitBoard; 6]; 2],
    mailbox: [Option<(Color, Piece)>; 64],
}

impl Board {
    /// Return the BitBoard associated to that Color and Piece
    #[inline]
    pub const fn get_bitboard(&self, color: Color, piece: Piece) -> BitBoard {
        self.bitboards[color as usize][piece as usize]
    }

    /// Return the Piece and it's Color present on that Square 
    #[inline]
    pub const fn get_piece(&self, square: Square) -> Option<(Color, Piece)> {
        self.mailbox[square as usize]
    }
}

impl Default for Board {
    /// Return a new Board with the default chess position
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
        for color in &Color::COLORS {
            for piece in &Piece::PIECES {
                for square in bitboards[*color as usize][*piece as usize].iter_squares() {
                    mailbox[square as usize] = Some((*color, *piece));
                }
            }
        }

        Board {bitboards, mailbox}
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const CHARS: [[char; 6]; 2] = [
            ['♙', '♖', '♘', '♗', '♕', '♔'],
            ['♟', '♜', '♞', '♝', '♛', '♚'],
        ];

        writeln!(f, " a b c d e f g h").unwrap();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display() {
        let board = Board::default();
        println!("{}", board);
    }
}
