mod bmi2;
use bmi2::{BISHOP_BMI2, ROOK_BMI2};

mod non_sliders_attacks;
use non_sliders_attacks::*;
mod slider_attacks;
use slider_attacks::SLIDER_ATTACKS;

use crate::bitboard::BitBoard;
use crate::bits::{pext, pdep};
use crate::color::Color;
use crate::piece::Piece;
use crate::square::Square;

//#################################################################################################
//
//                                           Pawns
//
//#################################################################################################

/// Return the attacks BitBoard of a Pawn of Color color located on square sq with Board occupancy occ
#[inline(always)]
fn pawn_attacks(color: Color, sq: Square) -> BitBoard {
    BitBoard(match color {
        Color::White => WHITE_PAWN_ATTACKS[sq as usize],
        Color::Black => BLACK_PAWN_ATTACKS[sq as usize],
    })
}

#[inline(always)]
fn pawn_push(color: Color, sq: Square) -> Square {
    Square::from(match color {
        Color::White => WHITE_PAWN_PUSHES[sq as usize],
        Color::Black => BLACK_PAWN_PUSHES[sq as usize],
    })
}

#[inline(always)]
fn double_push(color: Color, sq: Square) -> Square {
    Square::from(match color {
        Color::White => WHITE_PAWN_DOUBLE_PUSHES[sq as usize],
        Color::Black => BLACK_PAWN_DOUBLE_PUSHES[sq as usize],
    })
}

//#################################################################################################
//
//                              Bishops, Rooks, Queens
//
//#################################################################################################

/// Return the attacks BitBoard of a Bishop located on square sq, with Board occupancy occ
#[inline(always)]
fn bishop_attacks(color: Color, sq: Square, occ: BitBoard) -> BitBoard {
    let bmi2 = BISHOP_BMI2[sq as usize];
    BitBoard(pdep(SLIDER_ATTACKS[bmi2.0 + pext(occ.0, bmi2.1) as usize] as u64, bmi2.2))
}

/// Return the attacks BitBoard of a Rook located on square sq, with Board occupancy occ
#[inline(always)]
fn rook_attacks(color: Color, sq: Square, occ: BitBoard) -> BitBoard {
    let bmi2 = ROOK_BMI2[sq as usize];
    BitBoard(pdep(SLIDER_ATTACKS[bmi2.0 + pext(occ.0, bmi2.1) as usize] as u64, bmi2.2))
}

/// Return the attacks BitBoard of a Queen located on square sq, with Board occupancy occ
#[inline(always)]
fn queen_attacks(color: Color, sq: Square, occ: BitBoard) -> BitBoard {
    bishop_attacks(color, sq, occ) | rook_attacks(color, sq, occ)
}

//#################################################################################################
//
//                                 King, Knight
//
//#################################################################################################

/// Return the attacks BitBoard of a King located on square sq
#[inline(always)]
fn king_attacks(color: Color, sq: Square) -> BitBoard {
    BitBoard(KING_ATTACKS[sq as usize])
}

/// Return the attacks BitBoard of a Knight located on square sq
#[inline(always)]
fn knight_attacks(color: Color, sq: Square) -> BitBoard {
    BitBoard(KNIGHT_ATTACKS[sq as usize])
}

//#################################################################################################
//
//                                 Generate attack
//
//#################################################################################################

/// Generate the attacks of a given piece, with the corresponding color and on square sq
#[inline(always)]
pub fn attacks(color: Color, piece: Piece, sq: Square, occ: BitBoard) -> BitBoard {
    match piece {
        Piece::Pawn => pawn_attacks(color, sq),
        Piece::Rook => rook_attacks(color, sq, occ),
        Piece::Knight => knight_attacks(color, sq),
        Piece::Bishop => bishop_attacks(color, sq, occ),
        Piece::Queen => queen_attacks(color, sq, occ),
        Piece::King => king_attacks(color, sq),
    }
}