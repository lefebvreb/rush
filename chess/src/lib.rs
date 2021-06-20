#![allow(dead_code, unused_variables, unused_macros)]

mod bitboard;
mod color;
mod errors;
mod square;

pub fn init() {
    unsafe {
        bitboard::init_shifts();
    }
}