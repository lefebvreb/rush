use chess::board::Board;
use chess::color::Color;
use chess::piece::Piece;
use chess::square::Square;

// Returns true if the board is in pseudo-draw because of either the
// 50 move rule or an incoming threefold repetition.
#[inline]
pub(crate) fn is_pseudo_draw(board: &Board) -> bool {
    board.get_halfmove() == 100 || board.test_upcoming_repetition()
}

// Returns true if the board can be considered in endgame.
#[inline]
pub(crate) fn is_endgame(board: &Board) -> bool {
    Color::COLORS.iter().all(|&color| {
        let queens = board.get_bitboard(color, Piece::Queen);
        let rooks  = board.get_bitboard(color, Piece::Rook);
        let occ    = board.get_occupancy().colored(color);

        queens.empty() || (queens.count() == 1 && rooks.empty() && occ.count() < 3)
    })
}

// Returns the color the king is of that color is standing on.
#[inline]
pub(crate) fn king_sq_color(board: &Board, color: Color) -> Square {
    unsafe {board.get_bitboard(color, Piece::King).as_square_unchecked()}
}