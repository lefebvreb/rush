use chess::board::Board;
use chess::moves::Move;
use chess::piece::Piece;
use chess::prelude::Color;

use crate::utils;

/// Returns the heuristic value of a piece, in pawns.
#[inline]
pub(crate) const fn value_of(piece: Piece) -> f32 {
    match piece {
        Piece::Pawn => 1.0,
        Piece::Rook => 5.0,
        Piece::Knight => 3.2,
        Piece::Bishop => 3.3,
        Piece::Queen => 9.0,
        Piece::King => 1000.0,
    }
}

//#################################################################################################
//
//                                       struct GlobalInfo
//
//#################################################################################################

/// A struct designed to handle evaluation of the board.
#[derive(Copy, Clone, Default, Debug)]
pub(crate) struct Eval;

// ================================ pub(crate) impl

impl Eval {
    /// Evaluates a given position.
    pub(crate) fn new(board: &Board) -> Eval {
        todo!()
    }

    /// Updates the evaluation score from the given move played
    /// the given position. Much cheaper than a real evaluation.
    #[inline]
    pub(crate) fn update(&mut self, board: &Board, mv: Move) {
        todo!()
    }

    /// Returns the value of the evaluation.
    #[inline]
    pub(crate) fn get(&self) -> f32 {
        todo!()
    }
}

// ================================ traits impl


//#################################################################################################
//
//                                         classical evaluation
//
//#################################################################################################
/*
static mut QK_DIST: [[f32; 64]; 64] = [[0.0; 64]; 64];
static mut RK_DIST: [[f32; 64]; 64] = [[0.0; 64]; 64];
static mut NK_DIST: [[f32; 64]; 64] = [[0.0; 64]; 64];
static mut BK_DIST: [[f32; 64]; 64] = [[0.0; 64]; 64];

pub(crate) unsafe fn init() {
    let mut diag_nw = [0; 64];
    let mut diag_ne = [0; 64];

    for x in 0..8 {
        for y in 0..8 {
            let i = x + 8 * y;
            diag_nw[i] = (x + y) as i8;
            diag_ne[i] = (7 - x + y) as i8;
        }
    }

    const DIAG_BONUS: [f32; 15] = [5.0, 4.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];

    for sq1 in Square::SQUARES {
        for sq2 in Square::SQUARES {
            let dist_bonus = (f32::from(sq1.x() - sq2.x()) + f32::from(sq1.y() - sq2.y())).abs();

            let i = usize::from(sq1);
            let j = usize::from(sq2);

            QK_DIST[i][j] = dist_bonus * 2.5;
            RK_DIST[i][j] = dist_bonus * 0.5;
            NK_DIST[i][j] = dist_bonus;

            BK_DIST[i][j] += DIAG_BONUS[(diag_ne[i] - diag_ne[j]).abs() as usize];
            BK_DIST[i][j] += DIAG_BONUS[(diag_nw[i] - diag_nw[j]).abs() as usize];
        }
    }
}
*/

const PAWNS: [f32; 64] = [
    0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    0.05, 0.1, 0.1, -0.2, -0.2, 0.1, 0.1, 0.05,
    0.05, -0.05, -0.1, 0.0, 0.0, -0.1, -0.05, 0.05,
    0.0, 0.0, 0.0, 0.2, 0.2, 0.0, 0.0, 0.0,
    0.05, 0.05, 0.1, 0.25, 0.25, 0.1, 0.05, 0.05,
    0.1, 0.1, 0.2, 0.3, 0.3, 0.2, 0.1, 0.1,
    0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5,
    0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
];

const ROOKS: [f32; 64] = [
    0.0, 0.0, 0.0, 0.05, 0.05, 0.0, 0.0, 0.0,
    -0.05, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -0.05,
    -0.05, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -0.05,
    -0.05, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -0.05,
    -0.05, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -0.05,
    -0.05, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -0.05,
    0.05, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.05,
    0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
];

const KNIGHTS: [f32; 64] = [
    -0.5, -0.4, -0.3, -0.3, -0.3, -0.3, -0.4, -0.5,
    -0.4, -0.2, 0.0, 0.05, 0.05, 0.0, -0.2, -0.4,
    -0.3, 0.05, 0.1, 0.15, 0.15, 0.1, 0.05, -0.3,
    -0.3, 0.0, 0.15, 0.2, 0.2, 0.15, 0.0, -0.3,
    -0.3, 0.05, 0.15, 0.2, 0.2, 0.15, 0.05, -0.3,
    -0.3, 0.0, 0.1, 0.15, 0.15, 0.1, 0.0, -0.3,
    -0.4, -0.2, 0.0, 0.0, 0.0, 0.0, -0.2, -0.4,
    -0.5, -0.4, -0.3, -0.3, -0.3, -0.3, -0.4, -0.5,
];

const BISHOPS: [f32; 64] = [
    -0.2, -0.1, -0.1, -0.1, -0.1, -0.1, -0.1, -0.2,
    -0.1, 0.05, 0.0, 0.0, 0.0, 0.0, 0.05, -0.1,
    -0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, -0.1,
    -0.1, 0.0, 0.1, 0.1, 0.1, 0.1, 0.0, -0.1,
    -0.1, 0.05, 0.05, 0.1, 0.1, 0.05, 0.05, -0.1,
    -0.1, 0.0, 0.05, 0.1, 0.1, 0.05, 0.0, -0.1,
    -0.1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -0.1,
    -0.2, -0.1, -0.1, -0.1, -0.1, -0.1, -0.1, -0.2,
];

const QUEENS: [f32; 64] = [
    -0.2, -0.1, -0.1, -0.05, -0.05, -0.1, -0.1, -0.2,
    -0.1, 0.0, 0.05, 0.0, 0.0, 0.0, 0.0, -0.1,
    -0.1, 0.05, 0.05, 0.05, 0.05, 0.05, 0.0, -0.1,
    0.0, 0.0, 0.05, 0.05, 0.05, 0.05, 0.0, -0.05,
    -0.05, 0.0, 0.05, 0.05, 0.05, 0.05, 0.0, -0.05,
    -0.1, 0.0, 0.05, 0.05, 0.05, 0.05, 0.0, -0.1,
    -0.1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -0.1,
    -0.2, -0.1, -0.1, -0.05, -0.05, -0.1, -0.1, -0.2,
];

const KINGS_EARLY: [f32; 64] = [
    0.2, 0.3, 0.1, 0.0, 0.0, 0.1, 0.3, 0.2,
    0.2, 0.2, 0.0, 0.0, 0.0, 0.0, 0.2, 0.2,
    -0.1, -0.2, -0.2, -0.2, -0.2, -0.2, -0.2, -0.1,
    -0.2, -0.3, -0.3, -0.4, -0.4, -0.3, -0.3, -0.2,
    -0.3, -0.4, -0.4, -0.5, -0.5, -0.4, -0.4, -0.3,
    -0.3, -0.4, -0.4, -0.5, -0.5, -0.4, -0.4, -0.3,
    -0.3, -0.4, -0.4, -0.5, -0.5, -0.4, -0.4, -0.3,
    -0.3, -0.4, -0.4, -0.5, -0.5, -0.4, -0.4, -0.3,
];


const KINGS_ENDGAME: [f32; 64] = [
    -0.5, -0.3, -0.3, -0.3, -0.3, -0.3, -0.3, -0.5,
    -0.3, -0.3, 0.0, 0.0, 0.0, 0.0, -0.3, -0.3,
    -0.3, -0.1, 0.2, 0.3, 0.3, 0.2, -0.1, -0.3,
    -0.3, -0.1, 0.3, 0.4, 0.4, 0.3, -0.1, -0.3,
    -0.3, -0.1, 0.3, 0.4, 0.4, 0.3, -0.1, -0.3,
    -0.3, -0.1, 0.2, 0.3, 0.3, 0.2, -0.1, -0.3,
    -0.3, -0.2, -0.1, 0.0, 0.0, -0.1, -0.2, -0.3,
    -0.5, -0.4, -0.3, -0.2, -0.2, -0.3, -0.4, -0.5,
];

const TABLES: [[f32; 64]; 5] = [
    PAWNS, ROOKS, KNIGHTS,
    BISHOPS, QUEENS
];

/// The evaluation function.
pub(crate) fn eval(board: &Board) -> f32 {
    let mut score = 0.0;

    for &piece in &Piece::PIECES[..5] {
        for sq in board.get_bitboard(Color::White, piece).iter_squares() {
            score += value_of(piece) + TABLES[usize::from(piece)][usize::from(sq)];
        }
        
        for sq in board.get_bitboard(Color::Black, piece).iter_squares() {
            score -= value_of(piece) + TABLES[usize::from(piece)][63 - usize::from(sq)];
        }  
    }

    let white_king_sq = utils::king_sq_color(board, Color::White);
    let black_king_sq = utils::king_sq_color(board, Color::Black);

    if utils::is_endgame(board) {
        score += KINGS_ENDGAME[usize::from(white_king_sq)];
        score -= KINGS_ENDGAME[63 - usize::from(black_king_sq)];
    } else {
        score += KINGS_EARLY[usize::from(white_king_sq)];
        score -= KINGS_EARLY[63 - usize::from(black_king_sq)];
    }

    if board.get_side_to_move() == Color::Black {
        score = -score;
    }

    score
}
