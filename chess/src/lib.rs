#![feature(generator_trait, generators)]
#![allow(dead_code, unused_variables, unused_macros)]

/* ======== MEMO ===========

1. Represent a valid game with accessers
2. Generate all legal moves
3. Encapsulate ?
4. Be clean
5. Be EFFICIENT

========================= */

/* ======== TODO ===========

IMPLEMENTATION

- Implement all functions of move_gen.rs

OPTIMISATIONS

- replace all `unreachable!()` by `unsafe {unreacheable_unchecked()}`
- replace all table lookup by unchecked accesses (`get_unchecked`)

TESTS

- perft

========================= */

/* ======== IDEA ===========

FOR NEXT ITERATION

- create a type `single_bit_bitboard`

========================= */

// Modules
mod attacks;
mod bitboard;
mod bits;
mod board;
mod castle_rights;
mod color;
mod game;
mod moves;
mod move_gen;
mod piece;
mod ply;
mod square;

// Exports
pub use board::Board;
pub use color::Color;
pub use game::Game;
pub use moves::Move;
pub use move_gen::MoveGenerator;
pub use piece::Piece;
pub use square::Square;

// prelude module
pub mod prelude {
    pub use super::*;
}