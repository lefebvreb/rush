#![allow(dead_code, unused_variables)]

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

TESTS

- perft

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