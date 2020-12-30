mod bmi2_tables;
use bmi2_tables::*;

mod non_sliders_attacks;
use non_sliders_attacks::*;

mod pin_tables;
use pin_tables::*;

mod slider_attacks;
use slider_attacks::*;

use crate::bitboard::BitBoard;
use crate::bits::{pext, pdep};
use crate::board::Board;
use crate::color::Color;
use crate::piece::Piece;
use crate::square::Square;

//#################################################################################################
//
//                                           Pawns
//
//#################################################################################################

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

//#################################################################################################
//
//                              Bishops, Rooks, Queens
//
//#################################################################################################

// Return the attacks BitBoard of a Bishop located on square sq, with Board occupancy occ
#[inline(always)]
fn bishop_attacks(sq: Square, occ: BitBoard) -> BitBoard {
    let bmi2 = BISHOP_BMI2[sq as usize];
    BitBoard(pdep(SLIDER_ATTACKS[bmi2.0 + pext(occ.0, bmi2.1) as usize] as u64, bmi2.2))
}

// Return the attacks BitBoard of a Rook located on square sq, with Board occupancy occ
#[inline(always)]
fn rook_attacks(sq: Square, occ: BitBoard) -> BitBoard {
    let bmi2 = ROOK_BMI2[sq as usize];
    BitBoard(pdep(SLIDER_ATTACKS[bmi2.0 + pext(occ.0, bmi2.1) as usize] as u64, bmi2.2))
}

// Return the attacks BitBoard of a Queen located on square sq, with Board occupancy occ
#[inline(always)]
fn queen_attacks(sq: Square, occ: BitBoard) -> BitBoard {
    bishop_attacks(sq, occ) | rook_attacks(sq, occ)
}

//#################################################################################################
//
//                                 King, Knight
//
//#################################################################################################

// Return the attacks BitBoard of a King located on square sq
#[inline(always)]
fn king_attacks(sq: Square) -> BitBoard {
    BitBoard(KING_ATTACKS[sq as usize])
}

// Return the attacks BitBoard of a Knight located on square sq
#[inline(always)]
fn knight_attacks(sq: Square) -> BitBoard {
    BitBoard(KNIGHT_ATTACKS[sq as usize])
}

//#################################################################################################
//
//                                 Generate attack
//
//#################################################################################################

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

//#################################################################################################
//
//                                         Pin
//
//#################################################################################################

// Return a bitboard containing all pinned pieces
#[inline(always)]
pub fn get_pinned(color: Color, board: &Board) -> BitBoard {
    let king_offset = board.get_bitboard(color, Piece::King).as_square_unchecked() as usize * 64;
    let color_inv = color.invert();
    let queens = board.get_bitboard(color_inv, Piece::Queen);
    let occ = board.get_occupancy();
    let us = board.get_color_occupancy(color);

    let mut pinned = BitBoard::EMPTY;

    for rook_square in (board.get_bitboard(color_inv, Piece::Rook) | queens).iter_squares() {
        let between = BitBoard(SQUARES_BETWEEN_STRAIGHT[king_offset + rook_square as usize]);

        if (occ & between).count_bits() == 1 {
            let maybe_pinned = us & between;

            if maybe_pinned.is_not_empty() {
                pinned |= maybe_pinned;
            }
        }
    }

    for bishop_square in (board.get_bitboard(color_inv, Piece::Bishop) | queens).iter_squares() {
        let between = BitBoard(SQUARES_BETWEEN_DIAGONAL[king_offset + bishop_square as usize]);

        if (occ & between).count_bits() == 1 {
            let maybe_pinned = us & between;

            if maybe_pinned.is_not_empty() {
                pinned |= maybe_pinned;
            }
        }
    }

    pinned
}

// Return a mask in which the pinned piece can move freely
#[inline(always)]
pub fn get_pin_mask(king_square: Square, pinned_piece_square: Square) -> BitBoard {
    BitBoard(SQUARES_MASK[king_square as usize * 64 + pinned_piece_square as usize])
}

// Return the squares strictly contained between the two arguments
#[inline(always)]
pub fn squares_between(sq1: Square, sq2: Square) -> BitBoard {
    let i = sq1 as usize + 64 * sq2 as usize;
    BitBoard(SQUARES_BETWEEN_STRAIGHT[i] | SQUARES_BETWEEN_DIAGONAL[i])
}