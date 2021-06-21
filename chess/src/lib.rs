#![allow(dead_code, unused_variables, unused_macros)]

mod bitboard;
mod castle_rights;
mod color;
mod en_passant;
mod errors;
mod moves;
mod piece;
mod position;
mod square;
mod zobrist;

/// Initialize the components of the chess lib
pub fn init() {
    static mut DONE: bool = false;

    unsafe {
        if DONE {
            return;
        }
        DONE = true;

        bitboard::init_shifts();
        zobrist::init_zobrist();
    }
}