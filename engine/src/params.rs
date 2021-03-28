use std::time::Duration;

use chess::Piece;

// Represent a value
pub(crate) type PawnValue = f32;

// ================================ Threading and shared data constants

// The number of sub-threads
pub(crate) const NUM_SEARCH_THREADS: u8 = 8;

// The duration of the search
pub(crate) const SEARCH_DURATION: Duration = Duration::from_secs(10);

// 16 MB of ram for the hashtable
pub(crate) const HASHTABLE_MEM_SIZE: usize = 0x1000000;

// ================================ Search constants

// The maximum depth a thread can reach
pub(crate) const MAX_DEPTH: u8 = 128;

// Constant used for delta pruning in quiescent search
pub(crate) const DELTA: PawnValue = 2.0;

// The aspiration windows
pub(crate) const ASPIRATION_WINDOW: [PawnValue; 4] = [10.0, 50.0, 250.0, f32::INFINITY];

// The value given to pieces
#[inline(always)]
pub(crate) fn value_of(piece: Piece) -> PawnValue {
	match piece {
		Piece::Pawn   => 1.0,
		Piece::Rook   => 5.0,
		Piece::Knight => 3.2,
		Piece::Bishop => 3.3,
		Piece::Queen  => 9.0,
		Piece::King   => 200.0,
	}
}