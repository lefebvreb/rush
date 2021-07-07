use chess::moves::Move;
use chess::zobrist::Zobrist;

use crate::params;

//#################################################################################################
//
//                                         struct Entry
//
//#################################################################################################

// An enum representing the type of a node.
#[derive(Copy, Clone, Debug)]
enum NodeFlag {
    Alpha = 0,
    Beta  = 1,
    Exact = 2,
}

// A struct representing an entry of the table.
#[derive(Copy, Clone, Debug)]
pub(crate) struct Entry {
    mv: Move,
    score: f32,
    age: u16,
    depth: u8,
    flag: NodeFlag,
}

//#################################################################################################
//
//                                     struct TranspositionTable
//
//#################################################################################################

// The type of a bucket in the map.
type Bucket = Option<Entry>;

// The size in buckets of the table.
const NUM_BUCKETS: usize = params::TABLE_SIZE / std::mem::size_of::<Bucket>();

// The struct representing an access to a transposition table.
// A transposition table is a lock-less memory-efficient concurrent hashmap.
// It's only default is that it is lossy and may rarely corrupt some of it's data.
#[repr(transparent)]
#[derive(Clone, Debug)]
pub(crate) struct TranspositionTable(*mut Bucket);

impl TranspositionTable {
    // Creates a new transposition table, from leaking a vector.
    pub(crate) fn new() -> TranspositionTable {
        let mut vec = vec![None; NUM_BUCKETS];
        let ptr = vec.as_mut_ptr();
        vec.leak();

        TranspositionTable(ptr)
    }

    // Gets the bucket corresponding to a key, or None if there is none.
    #[inline]
    pub(crate) unsafe fn get(&self, zobrist: Zobrist) -> Bucket {
        let i = zobrist.idx::<NUM_BUCKETS>();
        *self.0.offset(i)  
    }
    
    // Inserts into the hashtable, or not depending on the replacement strategy.
    #[inline]
    pub(crate) unsafe fn insert(&mut self, zobrist: Zobrist, entry: Entry) {
        let i = zobrist.idx::<NUM_BUCKETS>();

        if let Some(prev) = *self.0.offset(i) {
            let replace_score = 
                entry.depth as i32 - prev.depth as i32 + 
                entry.age   as i32 - prev.age   as i32 +
                entry.flag  as i32 - prev.flag  as i32;

            if replace_score < 0 {
                return;
            }
        }

        *self.0.offset(i) = Some(entry)
    }

    // Probes the hashmap and gets any pertinent information available.
    #[inline]
    pub(crate) unsafe fn probe(&self, zobrist: Zobrist, alpha: f32, beta: f32, depth: u8) -> Option<(Entry, f32)> {
        self.get(zobrist)
            .filter(|entry| entry.depth >= depth)
            .and_then(|entry| {
                let score = entry.score;
                match entry.flag {
                    NodeFlag::Exact => Some((entry, score)),
                    NodeFlag::Alpha if score <= alpha => Some((entry, alpha)),
                    NodeFlag::Beta  if score >= beta  => Some((entry, beta)),
                    _ => None,
                }
            })
    }
}

// ================================ traits impl

impl Drop for TranspositionTable {
    // TranspositionTable needs to be manually dropped.
    fn drop(&mut self) {
        unsafe {Box::from_raw(self.0)};
    }
}

// rustc correctly assesses that our TranspositionTable is not thread-safe.
// Let us turn a blind eye to that.
unsafe impl Send for TranspositionTable {}
unsafe impl Sync for TranspositionTable {}