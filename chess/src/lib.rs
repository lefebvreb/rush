#![allow(dead_code, unused_variables, unused_macros)]

mod attacks;
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
#[cold]
pub fn init() {
    unsafe {
        square::init();
        bitboard::init();
        zobrist::init();
        attacks::init();
    }
}

#[cfg(not(target_feature = "bmi2"))]
fn _intrinsic_check() {
    // Comment out that function once the warning has been acknoledged
    compile_error!("bmi2 extension not found. Program will be slower if compiled.");
}

#[test]
fn size() {
    use std::sync::atomic::AtomicU64;

    eprintln!("{}", std::mem::size_of::<AtomicU64>());
    eprintln!("{}", std::mem::size_of::<u64>());
}