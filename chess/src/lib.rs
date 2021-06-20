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

pub fn init() {
    unsafe {
        bitboard::init_shifts();
        zobrist::init_zobrist();
    }
}