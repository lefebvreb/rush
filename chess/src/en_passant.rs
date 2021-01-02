use crate::attacks::squares_between;
use crate::bitboard::BitBoard;
use crate::board::Board;
use crate::color::Color;
use crate::piece::Piece;
use crate::square::Square;

// A type to represent en passant availability
#[repr(u8)]
#[derive(Debug)]
pub enum EnPassantAvailability {
    None,
    Left(Square),
    Right(Square),
    Both {
        left: Square,
        right: Square,
    }
}

impl EnPassantAvailability {
    // Get the en passant availability of a position
    pub fn get(color: Color, color_inv: Color, pawn_sq: Square, king_sq: Square, board: &Board) -> EnPassantAvailability {
        const FILES: [BitBoard; 2] = [
            BitBoard(0xFF00000000),
            BitBoard(0xFF000000),
        ];

        let x = pawn_sq.x();

        if x == 0 {
            // Left-most
            let right = pawn_sq.get_right_unchecked();

            match board.get_piece(right) {
                Some((c, Piece::Pawn)) if c == color => EnPassantAvailability::Right(right),
                _ => EnPassantAvailability::None,
            }
        } else if x == 7 {
            // Right-most
            let left = pawn_sq.get_left_unchecked();

            match board.get_piece(left) {
                Some((c, Piece::Pawn)) if c == color => EnPassantAvailability::Left(left),
                _ => EnPassantAvailability::None,
            }
        } else {
            // Not on the side
            let left = pawn_sq.get_left_unchecked();
            let right = pawn_sq.get_right_unchecked();

            let mut flags = 0u8;

            match board.get_piece(left) {
                Some((c, Piece::Pawn)) if c == color => flags |= 0b01,
                _ => (),
            }
            match board.get_piece(right) {
                Some((c, Piece::Pawn)) if c == color => flags |= 0b10,
                _ => (),
            }

            let queens = board.get_bitboard(color_inv, Piece::Queen);

            // Check if capturing may reveal a slider from the side
            match flags {
                0b00 => return EnPassantAvailability::None,
                0b01 | 0b10 => if pawn_sq.y() == king_sq.y() {
                    let sliders = board.get_bitboard(color_inv, Piece::Rook) | queens;

                    for sq in sliders.iter_squares() {
                        if sq.y() == king_sq.y() {
                            let between = squares_between(king_sq, sq);

                            if between.contains(pawn_sq) && between.count_bits() == 2 {
                                return EnPassantAvailability::None;
                            }
                        }
                    }
                }
                _ => (),
            }

            let sliders = board.get_bitboard(color_inv, Piece::Bishop) | queens;

            // Check if capturing may reveal a diagonal slider
            for sq in sliders.iter_squares() {
                let between = squares_between(king_sq, sq);

                if between.contains(pawn_sq) && between.count_bits() == 1 {
                    return EnPassantAvailability::None;
                }
            }

            match flags {
                0b01 => EnPassantAvailability::Left(left),
                0b10 => EnPassantAvailability::Right(right),
                0b11 => EnPassantAvailability::Both {left, right},
                _ => unreachable!()
            }
        }
    }
}