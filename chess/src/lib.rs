#![no_std]
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
#[cold]
pub fn init() {
    unsafe {
        bitboard::init();
        zobrist::init();
        attacks::init();
        cuckoo::init();
    }
}