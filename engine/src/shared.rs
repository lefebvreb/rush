use std::{mem, sync::atomic::Ordering};
use std::sync::atomic::AtomicU8;

use chess::{Game, Move};
use chess::Zobrist;

use crate::params::{HASHTABLE_MEM_SIZE, NUM_SEARCH_THREADS};

// Represent the result of the last search of that node: an alpha cut-off,
// a beta cut-off or an exact value
#[repr(u8)]
#[derive(Clone, Copy)]
pub(crate) enum NodeFlag {
    Alpha = 0,
    Beta  = 1,
    Exact = 2,
}

// A struct representing an Entry in the hashmap: 16 bytes on my machine.
#[derive(Clone, Copy)]
pub(crate) struct Entry {
    pub(crate) mv: Move,
    pub(crate) score: f32,
    pub(crate) age: u16,
    pub(crate) depth: u8,
    pub(crate) flag: NodeFlag,
}

// The size in buckets of the table
const SIZE: usize = HASHTABLE_MEM_SIZE / mem::size_of::<Option<Entry>>();

// A struct designed to be used in a singleton manner, and to
// hold entries for the threads to share what they do during the
// search
struct Table {
    buckets: [Option<Entry>; SIZE],
}

// The global hashtable
static mut TABLE: Table = Table {
    buckets: [None; SIZE],
};

//#################################################################################################
//
//                                  global hashtable accessers
//
//#################################################################################################

// Try to get the entry corresponding to that key
#[inline(always)]
pub(crate) fn table_get(zobrist: Zobrist) -> Option<Entry> {
    let i = zobrist.index::<SIZE>();
    unsafe {TABLE.buckets[i]}
}

// Try to insert a new entry in the hashtable
#[inline(always)]
pub(crate) fn table_insert(zobrist: Zobrist, entry: Entry) {
    let i = zobrist.index::<SIZE>();

    if let Some(prev) = unsafe {TABLE.buckets[i]} {
        let replace_score = 
            entry.depth as i32 - prev.depth as i32 + 
            entry.age   as i32 - prev.age   as i32 +
            entry.flag  as i32 - prev.flag  as i32;

        if replace_score < 0 {
            return;
        }
    }

    unsafe {
        TABLE.buckets[i] = Some(entry);
    }
}

// Probe a node by it's zobrist key, and see what information about it's
// value we can get, according to our current bounds and depth
#[inline(always)]
pub(crate) fn table_probe(zobrist: Zobrist, alpha: f32, beta: f32, depth: u8) -> Option<(Entry, f32)> {
    table_get(zobrist)
        .filter(|entry| entry.depth >= depth)
        .and_then(|entry| {
            let score = entry.score;
            match entry.flag {
                NodeFlag::Exact => Some((entry, score)),
                NodeFlag::Alpha if score <= alpha => Some((entry, alpha)),
                NodeFlag::Beta  if score >= beta  => Some((entry, beta)),
                _ => None
            }
        })
}

//#################################################################################################
//
//                            global search info accessers
//
//#################################################################################################

// The game to be searched
static mut GAME: Option<Game> = None;
// The current search depth
static mut SEARCH_DEPTH: AtomicU8 = AtomicU8::new(0);
// A counter telling threads what depth they need to search to
static mut SEARCH_ID: AtomicU8 = AtomicU8::new(0);
// The current best move
static mut BEST_MOVE: Option<Move> = None;
// A bool signaling whether or not the search should end
static mut STOP_SEARCH: bool = false;

// Reset the search infos to the defaults, preparing a new search
#[inline(always)]
pub(crate) fn reset_infos(game: Game) {
    unsafe {
        GAME         = Some(game);
        SEARCH_DEPTH = AtomicU8::new(0);
        SEARCH_ID    = AtomicU8::new(0);
        BEST_MOVE    = None;
        STOP_SEARCH  = false;
    }
}

// The depth a thread should search to
#[inline(always)]
pub(crate) fn search_depth() -> u8 {
    unsafe {
        SEARCH_DEPTH.load(Ordering::Relaxed) + 1 + SEARCH_ID.fetch_update(
            Ordering::Release, 
            Ordering::Acquire, 
            |id| Some((id + 1) % NUM_SEARCH_THREADS)
        ).unwrap().trailing_zeros() as u8
    }
}

// Report the move found and increment the global depth counter,
// if the depth provided is the best yet
#[inline(always)]
pub (crate) fn report_move(mv: Move, depth: u8) {
    unsafe {
        SEARCH_DEPTH.fetch_update(
            Ordering::Release,
            Ordering::Acquire,
            |cur_depth| if depth <= cur_depth {
                None
            } else {
                BEST_MOVE = Some(mv);
                Some(depth)
            }
        ).ok();
    }    
}

// Stop the search
#[inline(always)]
pub (crate) fn stop_search() {
    unsafe {
        STOP_SEARCH = true;
    }
}

// Check whether or not the search should end
#[inline(always)]
pub (crate) fn should_stop() -> bool {
    unsafe {
        STOP_SEARCH
    }
}

// Get a clone of the game to search
#[inline(always)]
pub (crate) fn game() -> Game {
    unsafe {
        GAME.clone().unwrap()
    }
}

// Get the best move found
#[inline(always)]
pub (crate) fn get_best_move() -> Option<Move> {
    unsafe {
        BEST_MOVE
    }
}