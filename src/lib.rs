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

mod bitboard;
mod board;
mod castle_rights;
mod color;
mod game;
mod moves;
mod piece;
mod square;
