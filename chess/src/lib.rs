// temporary
#![allow(dead_code, unused_variables, unused_macros)]

// Utilitary modules.
pub mod errors;
pub mod list;

// Primitive types.
pub mod bitboard;
pub mod color;
pub mod moves;
pub mod piece;
pub mod square;
pub mod zobrist;

// Logic modules.
mod attacks;
mod castle_rights;
mod en_passant;
mod cuckoo;
mod movegen;

// Board type.
pub mod board;

/*pub mod prelude {
    pub use crate::color::Color; 
    pub use crate::moves::Move; 
    pub use crate::piece::Piece; 
    pub use crate::square::Square; 
    pub use crate::board::Board; 
}*/

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