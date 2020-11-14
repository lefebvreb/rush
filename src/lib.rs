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

- Create a check system
- Refactor for clarity
- Implement the gen_legal_moves function of Game as a state machine (seperate file ?)

OPTIMISATIONS

-

TESTS

- Castling system

========================= */

/* Note on move generation

- If twice in check -> move king away from atk
- If once in check -> move king or capture attacker or block attacker
- Else, do whatever you want as long as you don't leave the king in check
  - Determine pinned pieces you can't move
  - Can move the king as long as not in danger

*/

mod bitboard;
mod bits;
mod board;
mod castle_rights;
mod color;
mod game;
mod moves;
mod piece;
mod ply;
mod square;
