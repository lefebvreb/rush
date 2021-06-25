#![allow(dead_code, unused_variables, unused_macros)]

mod attacks;
mod bitboard;
mod board;
mod castle_rights;
mod color;
mod cuckoo;
mod en_passant;
mod errors;
mod moves;
mod piece;
mod square;
mod zobrist;

pub mod prelude {
    pub use crate::board::Board;
    pub use crate::color::Color;
    pub use crate::moves::Move;
    pub use crate::piece::Piece;
    pub use crate::square::Square;
}

/// Initializes the components of the chess lib.
#[cold]
pub fn init() {
    unsafe {
        square::init();
        bitboard::init();
        zobrist::init();
        attacks::init();
        cuckoo::init();
    }
}