#![feature(generator_trait, generators)]

/* ======== TODO ===========

OPTIMISATIONS

- replace all `unreachable!()` by `unsafe {unreacheable_unchecked()}`
- replace all table lookup by unchecked accesses (`get_unchecked`)
- replace unwrap() by unwrap_unchecked()

- store pin masks inside of the board
- use the Piece type in argument to `update_occupied` so to save a match

========================= */

// Modules
mod attacks;
#[macro_use]
mod bitboard;
mod bits;
mod board;
mod castle_rights;
mod color;
mod en_passant;
mod errors;
mod game;
mod move_gen;
mod moves;
mod piece;
mod history;
mod square;

// Exports
pub use bitboard::BitBoard;
pub use board::Board;
pub use color::Color;
pub use game::{FullGame, Game, SearchGame};
pub use moves::Move;
pub use move_gen::MoveGenerator;
pub use piece::Piece;
pub use square::Square;