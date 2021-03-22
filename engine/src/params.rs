use std::time::Duration;

/// The number of sub-threads
pub const NUM_SEARCH_THREADS: u8 = 8;

/// 16 MB of ram for the hashtable
pub const HASHTABLE_MEM_SIZE: usize = 16777216;

pub const SEARCH_DURATION: Duration = Duration::from_millis(1000);

pub const MAX_DEPTH: u8 = 20;

pub const MATE_SCORE: f32 = 1e10;

pub const ASPIRATION_WINDOW: [f32; 4] = [10.0, 50.0, 250.0, f32::INFINITY];