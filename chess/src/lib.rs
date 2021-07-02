// If compiling to wasm32, disable std use.
#![cfg_attr(target_arch = "wasm32", no_std)]

// temporary
#![allow(dead_code, unused_variables, unused_macros)]

mod attacks;
mod bitboard;
mod board;
mod castle_rights;
mod color;
mod cuckoo;
mod en_passant;
mod errors;
mod list;
mod movegen;
mod moves;
mod piece;
mod square;
mod zobrist;

/// Initializes the components of the chess lib.
/// Must be called before using the methods of the chess lib.
#[cold]
pub fn init() {
    static mut DONE: bool = false;

    unsafe {
        if DONE {
            return;
        } else {
            DONE = true;
            bitboard::init();
            zobrist::init();
            attacks::init();
            cuckoo::init();
        }
    }
}