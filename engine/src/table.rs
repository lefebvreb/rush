use std::mem::size_of;

use chess::{EncodedMove, Move, Zobrist};

// 16 MB of ram for the hashtable
const MEM_SIZE: usize = 16777216;

//#################################################################################################
//
//                                         enum NodeType
//
//#################################################################################################

/// Represent the result of the last search of that node: an alpha cut-off,
/// a beta cut-off or an exact value
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum NodeFlag {
    Alpha = 0,
    Beta  = 1,
    Exact = 2,
}

// ================================ impl

impl NodeFlag {
    // An array containing all of NodeFlag's variants
    const NODE_FLAGS: [NodeFlag; 3] = [
        NodeFlag::Alpha,
        NodeFlag::Beta, 
        NodeFlag::Exact,
    ];
}

//#################################################################################################
//
//                                        struct Node
//
//#################################################################################################

/// A struct representing an Entry in the hashmap
#[repr(packed)]
#[derive(Clone, Copy)]
pub struct Entry {
    mv: EncodedMove,
    score: f32,
    age: u16,
    depth: u8,
    flag: NodeFlag,
}

// ================================ pub impl

impl Entry {
    /// Construct a new Entry
    #[inline(always)]
    pub fn new(mv: Move, score: f32, age: u16, depth: u8, flag: NodeFlag) -> Entry {
        Entry {
            mv: EncodedMove::from(mv),
            score,
            age,
            depth,
            flag,
        }
    }

    /// Return the best move found for that node in the last search
    #[inline(always)]
    pub fn mv(&self) -> Move {
        self.mv.into()
    }

    /// Return the score of the entry computed in the last search
    #[inline(always)]
    pub fn score(&self) -> f32 {
        self.score
    }

    /// The age of the entry
    #[inline(always)]
    pub fn age(&self) -> u16 {
        self.age
    }

    /// The search depth of the entry
    #[inline(always)]
    pub fn depth(&self) -> u8 {
        self.depth
    }

    /// The flag associated to that entry
    #[inline(always)]
    pub fn flag(&self) -> NodeFlag {
        self.flag
    }
}

impl Entry {
    // Compute a checksum of that entry, to help with sync problems
    #[inline(always)]
    fn checksum(&self) -> u32 {
        self.mv.get_raw() ^ self.score.to_bits() ^ (self.age as u32) ^ (self.depth as u32) ^ (self.flag as u32)
    }
}

//#################################################################################################
//
//                                        struct Table
//
//#################################################################################################

// The type of a bucket
type Bucket = Option<(u32, Entry)>;

// The size in buckets of the table
const SIZE: usize = MEM_SIZE / size_of::<Bucket>();

/// A struct designed to be used in a singleton manner, and to
/// hold entries for the threads to share what they do during the
/// search.
pub struct Table {
    buckets: [Bucket; SIZE],
}

// The global hashtable.
static mut TABLE: Table = Table {
    buckets: [None; SIZE],
};

// ================================ pub impl

impl Table {
    /// Try to get the entry corresponding to that zobrist key.
    #[inline(always)]
    pub fn get(zobrist: Zobrist) -> Option<Entry> {
        let i = zobrist.index::<SIZE>();

        if let Some((checksum, entry)) = unsafe {TABLE.buckets[i]} {
            if entry.checksum() == checksum {
                return Some(entry);
            }
        }

        None
    }

    /// Try to insert a new entry in the hashtable.
    #[inline(always)]
    pub fn insert(zobrist: Zobrist, entry: Entry) {
        let i = zobrist.index::<SIZE>();

        if let Some((checksum, prev)) = unsafe {TABLE.buckets[i]} {
            if prev.checksum() == checksum {
                let replace_score = entry.depth() as i32 - prev.depth() as i32
                    + (entry.age() as i32 - prev.age() as i32)
                    + (entry.flag() as i32 - prev.flag() as i32);

                if replace_score < 0 {
                    return;
                }
            }
        }

        let bucket = Some((entry.checksum(), entry));
        unsafe {
            TABLE.buckets[i] = bucket;
        }
    }

    /// Probes a node by it's zobrist key, and see what information about it's
    /// value we can get, according to our current bounds and depth.
    #[inline(always)]
    pub fn probe(zobrist: Zobrist, alpha: f32, beta: f32, depth: u8) -> Option<(Entry, f32)> {
        if let Some(entry) = Table::get(zobrist) {
            if entry.depth() >= depth {
                let score = entry.score();

                return match entry.flag() {
                    NodeFlag::Exact => Some((entry, score)),
                    NodeFlag::Alpha if score <= alpha => Some((entry, alpha)),
                    NodeFlag::Beta if score >= beta => Some((entry, beta)),
                    _ => None
                };
            }
        }

        None
    }
}