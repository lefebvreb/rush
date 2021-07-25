/// The size of the transposition table in bytes. Not exact.
pub(crate) const TABLE_SIZE: usize = 16777216;

/// The number of search threads used.
pub(crate) const NUM_SEARCH_THREAD: usize = 3;

/// The aspiration window used by the engine.
// pub(crate) const ASPIRATION_WINDOW: &[f32] = &[0.01, 0.05, 2.5, f32::INFINITY];
pub(crate) const ASPIRATION_WINDOW: &[f32] = &[10.0, 50.0, 250.0, f32::INFINITY];

/// The maximum search depth.
pub(crate) const MAX_DEPTH: usize = 32;

/// Used during quiescient search for delta pruning.
pub(crate) const DELTA: f32 = 2.0;