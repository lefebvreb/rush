mod bmi2_tables;
use bmi2_tables::*;

mod non_sliders_attacks_table;
use non_sliders_attacks_table::*;

mod pin_tables;
use pin_tables::*;

mod slider_attacks_tables;
use slider_attacks_tables::*;

use crate::bitboard::BitBoard;
use crate::bits::{pext, pdep};
use crate::board::Board;
use crate::color::Color;
use crate::piece::Piece;
use crate::square::Square;

// Return the attacks BitBoard of a Pawn of Color color located on square sq with Board occupancy occ
#[inline(always)]
fn pawn_attacks(color: Color, sq: Square) -> BitBoard {
    BitBoard(match color {
        Color::White => WHITE_PAWN_ATTACKS[sq as usize],
        Color::Black => BLACK_PAWN_ATTACKS[sq as usize],
    })
}

// Return the pawn push destination from that square
#[inline(always)]
pub fn get_pawn_push(color: Color, sq: Square) -> Option<Square> {
    match match color {
        Color::White => WHITE_PAWN_PUSHES[sq as usize],
        Color::Black => BLACK_PAWN_PUSHES[sq as usize],
    } {
        255 => None,
        n => Some(Square::from(n)),
    }
}

// Return the pawn double push destination from that square
#[inline(always)]
pub fn get_pawn_double_push(color: Color, sq: Square) -> Option<Square> {
    match match color {
        Color::White => WHITE_PAWN_DOUBLE_PUSHES[sq as usize],
        Color::Black => BLACK_PAWN_DOUBLE_PUSHES[sq as usize],
    } {
        255 => None,
        n => Some(Square::from(n)),
    }
}

// Return the attacks BitBoard of a Rook located on square sq, with Board occupancy occ
#[inline(always)]
fn rook_attacks(sq: Square, occ: BitBoard) -> BitBoard {
    let (shift, mask1, mask2) = ROOK_BMI2[sq as usize];
    BitBoard(pdep(SLIDER_ATTACKS[shift + pext(occ.0, mask1) as usize] as u64, mask2))
}

// Return the attacks BitBoard of a Knight located on square sq
#[inline(always)]
fn knight_attacks(sq: Square) -> BitBoard {
    BitBoard(KNIGHT_ATTACKS[sq as usize])
}

// Return the attacks BitBoard of a Bishop located on square sq, with Board occupancy occ
#[inline(always)]
fn bishop_attacks(sq: Square, occ: BitBoard) -> BitBoard {
    let (shift, mask1, mask2) = BISHOP_BMI2[sq as usize];
    BitBoard(pdep(SLIDER_ATTACKS[shift + pext(occ.0, mask1) as usize] as u64, mask2))
}

// Return the attacks BitBoard of a Queen located on square sq, with Board occupancy occ
#[inline(always)]
fn queen_attacks(sq: Square, occ: BitBoard) -> BitBoard {
    bishop_attacks(sq, occ) | rook_attacks(sq, occ)
}

// Return the attacks BitBoard of a King located on square sq
#[inline(always)]
fn king_attacks(sq: Square) -> BitBoard {
    BitBoard(KING_ATTACKS[sq as usize])
}

// Generate the attacks of a given piece, with the corresponding color and on square sq
#[inline(always)]
pub fn attacks(color: Color, piece: Piece, sq: Square, occ: BitBoard) -> BitBoard {
    match piece {
        Piece::Pawn => pawn_attacks(color, sq),
        Piece::Rook => rook_attacks(sq, occ),
        Piece::Knight => knight_attacks(sq),
        Piece::Bishop => bishop_attacks(sq, occ),
        Piece::Queen => queen_attacks(sq, occ),
        Piece::King => king_attacks(sq),
    }
}

// Return a bitboard containing all pinned pieces
#[inline(always)]
pub fn get_pinned(color: Color, board: &Board) -> BitBoard {
    let king_offset = board.get_bitboard(color, Piece::King).as_square_unchecked() as usize * 64;
    let color_inv = color.invert();
    let queens = board.get_bitboard(color_inv, Piece::Queen);
    let occ = board.get_occupancy();
    let us = board.get_color_occupancy(color);

    let mut pinned = BitBoard::EMPTY;

    macro_rules! detect_pins {
        ($piece: ident, $table: ident) => {
            for sq in (board.get_bitboard(color_inv, Piece::$piece) | queens).iter_squares() {
                let between = BitBoard($table[king_offset + sq as usize]);
        
                if (occ & between).count_bits() == 1 {
                    pinned |= occ & between;
                }
            }
        }
    }

    detect_pins!(Rook, SQUARES_BETWEEN_STRAIGHT);
    detect_pins!(Bishop, SQUARES_BETWEEN_DIAGONAL);

    pinned
}

// Return a mask in which the pinned piece can move freely
#[inline(always)]
pub fn get_projected_mask(from: Square, to: Square) -> BitBoard {
    let i = from as usize * 64 + to as usize;
    BitBoard(SQUARES_MASK[i])
}

// Return the squares strictly contained between the two arguments
#[inline(always)]
pub fn squares_between(sq1: Square, sq2: Square) -> BitBoard {
    let i = sq1 as usize + 64 * sq2 as usize;
    BitBoard(SQUARES_BETWEEN_STRAIGHT[i] | SQUARES_BETWEEN_DIAGONAL[i])
}

// Return the squares strictly contained between the two arguments, if there are on a diagonal
#[inline(always)]
pub fn squares_between_diagonal(sq1: Square, sq2: Square) -> BitBoard {
    let i = sq1 as usize + 64 * sq2 as usize;
    BitBoard(SQUARES_BETWEEN_DIAGONAL[i])
}