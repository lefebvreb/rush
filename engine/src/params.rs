use chess::piece::Piece;

/// The size of the transposition table in bytes. Not exact.
pub(crate) const TABLE_SIZE: usize = 16777216;

/// The number of search threads used.
pub(crate) const NUM_SEARCH_THREAD: usize = 3;

/// The aspiration window used by the engine.
pub(crate) const ASPIRATION_WINDOW: &[f32] = &[10.0, 50.0, 250.0, f32::INFINITY];

/// The maximum search depth.
pub(crate) const MAX_DEPTH: u8 = 32;

/// Used during quiescient search for move generation.
pub(crate) const DELTA: f32 = 2.0;

/// Returns the heuristic value of a piece, in pawns.
#[inline]
pub const fn value_of(piece: Piece) -> f32 {
    match piece {
        Piece::Pawn => 1.0,
        Piece::Rook => 5.0,
        Piece::Knight => 3.2,
        Piece::Bishop => 3.3,
        Piece::Queen => 9.0,
        Piece::King => 200.0,
    }
}