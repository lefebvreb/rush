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

#[cfg(not(target_feature = "bmi2"))]
fn _intrinsic_check() {
    // Comment out that function once the warning has been acknoledged.
    compile_error!("bmi2 extension not found: move generation will be slower if compiled.");
}