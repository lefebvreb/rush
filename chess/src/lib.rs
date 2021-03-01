// For lazy move generation
#![feature(generator_trait, generators)]

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
pub use clock::ThreefoldCounter;
pub use color::Color;
pub use errors::ParseFenError;
pub use game::{Game, GameStatus};
pub use moves::{EncodedMove, Move};
pub use move_gen::MoveGenerator;
pub use piece::Piece;
pub use square::Square;
pub use zobrist::{Position, Zobrist};