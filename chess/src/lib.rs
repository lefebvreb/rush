#![allow(dead_code, unused_variables, unused_macros)]
// For zobrist keys initialization
#![feature(const_mut_refs)]
// For lazy move generation
#![feature(generator_trait, generators)]

/*
TODO:

- Incremental Zobrist hashing:
    - Add a zobrist key to the board and update it by doing move
    - Update zobrist key along ep rights and castle rights
    - Have only one zobrist key for color (black, xor it at every halfturn)
- 50 move and 3-fold repetition clock
*/

// Modules
mod attacks;
#[macro_use]
mod bitboard;
mod bmi2;
mod board;
mod castle_rights;
mod clock;
mod color;
mod en_passant;
mod errors;
mod game;
mod move_gen;
mod moves;
mod piece;
mod square;
mod zobrist;

// Exports
pub use bitboard::BitBoard;
pub use board::Board;
pub use color::Color;
pub use game::Game;
pub use moves::Move;
pub use move_gen::MoveGenerator;
pub use piece::Piece;
pub use square::Square;