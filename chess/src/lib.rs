// Utilitary modules.
pub mod errors;

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

// Board type.
pub mod board;
pub mod movegen;

pub mod prelude {
    pub use crate::board::Board; 
    pub use crate::moves::Move;
    pub use crate::movegen;
}

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