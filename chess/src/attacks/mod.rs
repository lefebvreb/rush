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
use crate::board::Occupancy;
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
fn pawn_attacks(color: Color, sq: Square, occ: &Occupancy) -> BitBoard {
    match color {
        Color::White => BitBoard(WHITE_PAWN_ATTACKS[sq as usize]) & occ.black,
        Color::Black => BitBoard(BLACK_PAWN_ATTACKS[sq as usize]) & occ.white,
    }
}

/// Return the attacks BitBoard of a Pawn of Color color located on square sq with Board occupancy occ
#[inline(always)]
fn pawn_pushes(color: Color, sq: Square, occ: &Occupancy) -> (BitBoard, BitBoard) {
    match color {
        Color::White => {
            let single = BitBoard(WHITE_PAWN_PUSHES[sq as usize]) & occ.free;
            (single, BitBoard(if single.is_empty() {
                0
            } else {
                WHITE_PAWN_DOUBLE_PUSHES[sq as usize]
            }) & occ.free)
        }
        Color::Black => {
            let single = BitBoard(BLACK_PAWN_PUSHES[sq as usize]) & occ.free;
            (single, BitBoard(if single.is_empty() {
                0
            } else {
                BLACK_PAWN_DOUBLE_PUSHES[sq as usize]
            }) & occ.free)
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
fn bishop_attacks(color: Color, sq: Square, occ: &Occupancy) -> BitBoard {
    let bmi2 = BISHOP_BMI2[sq as usize];
    BitBoard(pdep(SLIDER_ATTACKS[bmi2.0 + pext(occ.all.0, bmi2.1) as usize] as u64, bmi2.2)) & occ.by_color(color)
}

/// Return the attacks BitBoard of a Rook located on square sq, with Board occupancy occ
#[inline(always)]
fn rook_attacks(color: Color, sq: Square, occ: &Occupancy) -> BitBoard {
    let bmi2 = ROOK_BMI2[sq as usize];
    BitBoard(pdep(SLIDER_ATTACKS[bmi2.0 + pext(occ.all.0, bmi2.1) as usize] as u64, bmi2.2)) & occ.by_color(color)
}

/// Return the attacks BitBoard of a Queen located on square sq, with Board occupancy occ
#[inline(always)]
fn queen_attacks(color: Color, sq: Square, occ: &Occupancy) -> BitBoard {
    bishop_attacks(color, sq, occ) | rook_attacks(color, sq, occ)
}

//#################################################################################################
//
//                                 King, Knight
//
//#################################################################################################

/// Return the attacks BitBoard of a King located on square sq
#[inline(always)]
fn king_attacks(color: Color, sq: Square, occ: &Occupancy) -> BitBoard {
    BitBoard(KING_ATTACKS[sq as usize]) & occ.by_color(color)
}

/// Return the attacks BitBoard of a Knight located on square sq
#[inline(always)]
fn knight_attacks(color: Color, sq: Square, occ: &Occupancy) -> BitBoard {
    BitBoard(KNIGHT_ATTACKS[sq as usize]) & occ.by_color(color)
}

//#################################################################################################
//
//                                 Generate attack
//
//#################################################################################################

#[inline(always)]
pub fn attacks(color: Color, piece: Piece, sq: Square, occ: &Occupancy) -> BitBoard {
    match piece {
        Piece::Pawn => pawn_attacks(color, sq, occ),
        Piece::Rook => rook_attacks(color, sq, occ),
        Piece::Knight => knight_attacks(color, sq, occ),
        Piece::Bishop => bishop_attacks(color, sq, occ),
        Piece::Queen => queen_attacks(color, sq, occ),
        Piece::King => king_attacks(color, sq, occ),
    }
}