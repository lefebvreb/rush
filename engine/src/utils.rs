use std::time::SystemTime;

use chess::bitboard::BitBoard;
use chess::board::Board;
use chess::color::Color;
use chess::piece::Piece;
use chess::square::Square;

/// Returns a random seed based on the current time.
#[inline]
pub(crate) fn seed() -> u32 {
    (SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).expect("Cannot get system time.").as_nanos() & 0xFFFFFFFF) as u32
}

/// The xorshift32 algorithm, producing 32 bits non-crypographic numbers.
#[inline]
pub(crate) fn xorshift32(seed: &mut u32) -> u32 {
    *seed ^= seed.wrapping_shl(13);
    *seed ^= seed.wrapping_shr(17);
    *seed ^= seed.wrapping_shl(5);
    *seed
}

/// Returns true if the board is in pseudo-draw because of either the
/// 50 move rule or an incoming threefold repetition.
#[inline]
pub(crate) fn is_pseudo_draw(board: &Board, alpha: f32, root: bool) -> bool {
    board.get_halfmove() >= 100 || (!root && alpha < 0.0 && board.test_upcoming_repetition())
}

/// Returns true if the board can be considered in endgame.
#[inline]
pub(crate) fn is_endgame(board: &Board) -> bool {
    Color::COLORS.iter().all(|&color| {
        let queens = board.get_bitboard(color, Piece::Queen);
        let rooks = board.get_bitboard(color, Piece::Rook);
        let occ = board.get_occupancy().colored(color);

        queens.empty() || (queens.is_one() && rooks.empty() && occ.count() < 3)
    })
}

/// Returns the color the king is of that color is standing on.
#[inline]
pub(crate) fn king_sq_color(board: &Board, color: Color) -> Square {
    // SAFE: there is always a king on the board.
    unsafe {board.get_bitboard(color, Piece::King).as_square_unchecked()}
}

/// Returns true if any of our pawn may promote this turn.
#[inline]
pub(crate) fn may_promote(board: &Board) -> bool {
    let us = board.get_side_to_move();
    (board.get_bitboard(us, Piece::Pawn) & BitBoard::promote_rank(us)).not_empty()
}

/// Returns a pseudo-random draw value, to avoid threefold repetitions.
#[inline]
pub(crate) fn prng_draw_value(seed: &mut u32) -> f32 {
    f32::from(2 * (xorshift32(seed) & 1) as i8 - 1)
}