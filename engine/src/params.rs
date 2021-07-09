// The size of the transposition table in bytes. Not exact.
pub(crate) const TABLE_SIZE: usize = 16777216;

// The number of search threads used.
pub(crate) const NUM_SEARCH_THREAD: usize = 8;

// The aspiration window used by the engine.
pub(crate) const ASPIRATION_WINDOW: &'static [f32] = &[10.0, 50.0, 250.0, f32::INFINITY];