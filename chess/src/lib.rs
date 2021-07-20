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
    pub use crate::board::{Board, Status}; 
    pub use crate::color::Color;
    pub use crate::moves::Move;
    pub use crate::movegen;
    pub use crate::square::Square;
}

/// Initializes the components of the chess lib.
/// Must be called before using the methods of the chess lib.
#[cold]
pub fn init() {
    use std::sync::Once;

    static INIT: Once = Once::new();

    INIT.call_once(|| unsafe {
        bitboard::init();
        zobrist::init();
        attacks::init();
        cuckoo::init();
    });
}