mod bmi2;
use bmi2::{BISHOP_BMI2, ROOK_BMI2};

mod non_sliders_attacks;
use non_sliders_attacks::{
    BLACK_PAWN_ATTACKS, BLACK_PAWN_PUSHES, BLACK_PAWN_DOUBLE_PUSHES, 
    KING_ATTACKS, KNIGHT_ATTACKS, 
    WHITE_PAWN_ATTACKS, WHITE_PAWN_PUSHES, WHITE_PAWN_DOUBLE_PUSHES
};

mod slider_attacks;
use slider_attacks::SLIDER_ATTACKS;

use crate::bitboard::BitBoard;
use crate::bits::{pext, pdep};
use crate::color::Color;
use crate::square::Square;

//#################################################################################################
//
//                                           Pawns
//
//#################################################################################################

/// Return the attacks BitBoard of a Pawn of Color color located on square sq with Board occupancy occ
#[inline(always)]
fn pawn_attacks(sq: Square, color: Color, occ: BitBoard) -> BitBoard {
    BitBoard(match color {
        Color::White => WHITE_PAWN_ATTACKS[sq as usize],
        Color::Black => BLACK_PAWN_ATTACKS[sq as usize],
    }) & occ
}

/// Return the attacks BitBoard of a Pawn of Color color located on square sq with Board occupancy occ
#[inline(always)]
fn pawn_pushes(sq: Square, color: Color, free: BitBoard) -> (BitBoard, BitBoard) {
    match color {
        Color::White => {
            let single = BitBoard(WHITE_PAWN_PUSHES[sq as usize]) & free;
            (single, BitBoard(if single.is_empty() {
                0
            } else {
                WHITE_PAWN_DOUBLE_PUSHES[sq as usize]
            }) & free)
        }
        Color::Black => {
            let single = BitBoard(BLACK_PAWN_PUSHES[sq as usize]) & free;
            (single, BitBoard(if single.is_empty() {
                0
            } else {
                BLACK_PAWN_DOUBLE_PUSHES[sq as usize]
            }) & free)
        }
    }
}

/*#[inline(always)]
fn en_passant(board: &Board, color: Color, last_move: Move) -> (Move, Move) {
    match last_move {
        Move::DoublePush {to, ..} => {
            todo!()
        }
        _ => (Move::None, Move::None),
    }
}*/

//#################################################################################################
//
//                              Bishops, Rooks, Queens
//
//#################################################################################################

/// Return the attacks BitBoard of a Bishop located on square sq, with Board occupancy occ
#[inline(always)]
fn bishop_attacks(sq: Square, occ: BitBoard) -> BitBoard {
    let bmi2 = BISHOP_BMI2[sq as usize];
    BitBoard(pdep(SLIDER_ATTACKS[bmi2.0 + pext(occ.0, bmi2.1) as usize] as u64, bmi2.2))
}

/// Return the attacks BitBoard of a Rook located on square sq, with Board occupancy occ
#[inline(always)]
fn rook_attacks(sq: Square, occ: BitBoard) -> BitBoard {
    let bmi2 = ROOK_BMI2[sq as usize];
    BitBoard(pdep(SLIDER_ATTACKS[bmi2.0 + pext(occ.0, bmi2.1) as usize] as u64, bmi2.2))
}

/// Return the attacks BitBoard of a Queen located on square sq, with Board occupancy occ
#[inline(always)]
fn queen_attacks(sq: Square, occ: BitBoard) -> BitBoard {
    bishop_attacks(sq, occ) | rook_attacks(sq, occ)
}

//#################################################################################################
//
//                                 King, Knight
//
//#################################################################################################

/// Return the attacks BitBoard of a King located on square sq
#[inline(always)]
fn king_attacks(sq: Square) -> BitBoard {
    BitBoard(KING_ATTACKS[sq as usize])
}

/// Return the attacks BitBoard of a Knight located on square sq
#[inline(always)]
fn knight_attacks(sq: Square) -> BitBoard {
    BitBoard(KNIGHT_ATTACKS[sq as usize])
}