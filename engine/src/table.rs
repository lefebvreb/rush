use chess::board::Board;
use chess::moves::Move;
use chess::zobrist::Zobrist;

use crate::params;

//#################################################################################################
//
//                                         struct Entry
//
//#################################################################################################

/// An enum representing the type of a node.
#[derive(Copy, Clone, Debug)]
pub(crate) enum TableEntryFlag {
    Alpha = 0,
    Beta  = 1,
    Exact = 2,
}

/// A struct representing an entry of the table.
#[derive(Copy, Clone, Debug)]
pub(crate) struct TableEntry {
    zobrist: Zobrist,
    age: u16,
    pub(crate) mv: Move,
    pub(crate) score: f32,
    depth: u8,
    flag: TableEntryFlag,
}

// ================================ pub(crate) impl

impl TableEntry {
    /// Creates a new table entry based with the given values.
    #[inline]
    pub(crate) fn new(board: &Board, mv: Move, score: f32, depth: u8, flag: TableEntryFlag) -> TableEntry {
        TableEntry {
            zobrist: board.get_zobrist(), 
            age: board.get_ply(), 
            mv, 
            score, 
            depth,
            flag,
        }
    }
}

//#################################################################################################
//
//                                     struct TranspositionTable
//
//#################################################################################################

/// The type of a bucket in the map.
type Bucket = Option<TableEntry>;

/// The size in buckets of the table. It is a power of two for
/// faster indexing.
const NUM_BUCKETS: usize = (params::TABLE_SIZE / std::mem::size_of::<Bucket>()).next_power_of_two();

/// The struct representing an access to a transposition table.
/// A transposition table is a lock-less memory-efficient concurrent hashmap.
/// It's only default is that it is lossy and may rarely corrupt some of it's data.
#[repr(transparent)]
#[derive(Clone, Debug)]
pub(crate) struct TranspositionTable(*mut Bucket);

// ================================ pub(crate) impl

impl TranspositionTable {
    /// Creates a new transposition table, from leaking a vector.
    pub(crate) fn new() -> TranspositionTable {
        let mut vec = vec![None; NUM_BUCKETS];
        let ptr = vec.as_mut_ptr();
        vec.leak();

        TranspositionTable(ptr)
    }
    
    /// Inserts into the hashtable, or not depending on the replacement strategy.
    #[inline]
    pub(crate) fn insert(&self, entry: TableEntry) {
        let i = entry.zobrist.idx::<NUM_BUCKETS>();

        // SAFE: not inherently unsafe, at worst we risk getting a currupted entry.
        if let Some(prev) = unsafe {*self.0.offset(i)} {
            let replace_score = 
                entry.depth as i32 - prev.depth as i32 + 
                entry.age   as i32 - prev.age   as i32 +
                entry.flag  as i32 - prev.flag  as i32;

            if replace_score < 0 {
                return;
            }
        }

        // SAFE: not inherently unsafe, at worst we risk corrupting an entry.
        unsafe {*self.0.offset(i) = Some(entry)};
    }

    /// Probes the hashmap and gets any pertinent information available.
    #[inline]
    pub(crate) fn probe(&self, zobrist: Zobrist, alpha: f32, beta: f32, depth: u8) -> Option<(Move, f32)> {
        let i = zobrist.idx::<NUM_BUCKETS>();
        
        // SAFE: not inherently unsafe, at worst we risk getting a currupted entry.
        if let Some(entry) = unsafe {*self.0.offset(i)} {
            if entry.zobrist == zobrist && entry.depth >= depth {
                let mv = entry.mv;
                let score = entry.score;

                return match entry.flag {
                    TableEntryFlag::Exact => Some((mv, score)),
                    TableEntryFlag::Alpha if score <= alpha => Some((mv, alpha)),
                    TableEntryFlag::Beta if score >= beta => Some((mv, beta)),
                    _ => None,
                };
            }
        }

        None
    }
}

// ================================ traits impl

impl Drop for TranspositionTable {
    /// TranspositionTable needs to be manually dropped.
    fn drop(&mut self) {
        unsafe {Box::from_raw(self.0)};
    }
}

// rustc correctly assesses that our TranspositionTable is not thread-safe.
// Let us turn a blind eye to that.
unsafe impl Send for TranspositionTable {}
unsafe impl Sync for TranspositionTable {}