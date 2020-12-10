#![allow(dead_code, unused_variables, unused_macros)]

/* ======== MEMO ===========

1. Represent a valid game with accessers
2. Generate all legal moves
3. Encapsulate ?
4. Be clean
5. Be EFFICIENT

========================= */



/* ======== MEMO ===========

  a b c d e f g h      
8 ♜ ♞ ♝ ♛ ♚ ♝ ♞ ♜ 
7 ♟ ♟ ♟ ♟ ♟ ♟ ♟ ♟ 
6 - - - - - - - - 
5 - - - - - - - - 
4 - - - - - - - - 
3 - - - - - - - - 
2 ♙ ♙ ♙ ♙ ♙ ♙ ♙ ♙ 
1 ♖ ♘ ♗ ♕ ♔ ♗ ♘ ♖ 
     
========================= */



/* ======== TODO ===========

IMPLEMENTATION

- Implement the gen_legal_moves function of Game as a state machine (seperate file ?)

OPTIMISATIONS

-

TESTS

- MoveGen (perft)

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