use std::time::Duration;

/// The number of sub-threads
pub const NUM_SEARCH_THREADS: u8 = 8;

/// 16 MB of ram for the hashtable
pub const HASHTABLE_MEM_SIZE: usize = 16777216;

pub const SEARCH_DURATION: Duration = Duration::from_millis(5000);