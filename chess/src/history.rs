use std::fmt;
use std::str::FromStr;

use crate::board::Board;
use crate::castle_rights::CastleRights;
use crate::color::Color;
use crate::errors::ParseFenError;
use crate::moves::Move;
use crate::square::Square;

// Represent a ply (half-turn) counter
#[derive(Copy, Clone, PartialEq, Default, Debug)]
pub struct MoveCounter {
    halfmoves: u8,
    fullmoves: u32,
}

impl MoveCounter {
    // Increment the counter
    #[inline(always)]
    pub fn increment(self, color: Color, mv: Move, board: &Board) -> MoveCounter {
        MoveCounter {
            halfmoves: if mv.is_reversible(board) {
                self.halfmoves + 1
            } else {
                0
            },
            fullmoves: match color {
                Color::White => self.fullmoves,
                Color::Black => self.fullmoves + 1,
            },
        }
    }

    // Parse a MoveCounter from two strings
    pub fn from_strs(s1: &str, s2: &str) -> Result<MoveCounter, ParseFenError> {
        Ok(MoveCounter {
            halfmoves: u8::from_str(s1)?,
            fullmoves: u32::from_str(s2)?,
        })
    }
}

impl fmt::Display for MoveCounter {
    // Display the counter in FEN notation
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.halfmoves, self.fullmoves)
    }
}

// A trait defining a move history aka a Vec of Move, CastleRights and MoveCounter
pub trait MoveHistory: 'static + Default {
    // Return the last move in history
    fn last_move(&self) -> Move;

    // Push another move, castle history and move counter to history
    fn push(&mut self, mv: Move, castle_rights: CastleRights, move_counter: MoveCounter);

    // Pop out the last element of the history
    fn pop(&mut self) -> (Move, CastleRights, MoveCounter);

    // Return a string representing the en passant target square, required by FEN notation
    fn en_passant_square(&self) -> String {
        match self.last_move() {
            Move::DoublePush {from, to} => Square::from((from.x(), (from.x() + from.y()) / 2)).to_string(),
            _ => "-".to_owned(),
        }
    }
}

// A type to hold a large move history, for example one
// fit for an entire game
#[derive(Debug)]
pub struct LargeMoveHistory(Vec<(Move, CastleRights, MoveCounter)>);

impl Default for LargeMoveHistory {
    #[cold]
    fn default() -> LargeMoveHistory {
        LargeMoveHistory(Vec::with_capacity(128))
    }
}

impl MoveHistory for LargeMoveHistory {
    #[inline(always)]
    fn last_move(&self) -> Move {
        match self.0.last() {
            Some((mv, _, _)) => *mv,
            _ => Move::None,
        }
    }

    #[inline(always)]
    fn push(&mut self, mv: Move, castle_rights: CastleRights, move_counter: MoveCounter) {
        self.0.push((mv, castle_rights, move_counter));
    }

    #[inline(always)]
    fn pop(&mut self) -> (Move, CastleRights, MoveCounter) {
        self.0.pop().unwrap()
    }
}

// A type to hold a small move history, for example one
// fit for exploring the game tree. MAX is the maximum
// number of elements that fit inside the history
#[derive(Debug)]
pub struct SmallMoveHistory<const MAX: usize> {
    cursor: usize,
    moves: [(Move, CastleRights, MoveCounter); MAX],
}

impl<const MAX: usize> Default for SmallMoveHistory<MAX> {
    #[inline(always)]
    fn default() -> SmallMoveHistory<MAX> {
        SmallMoveHistory {
            cursor: 0,
            moves: [(Move::None, CastleRights::NONE, MoveCounter::default()); MAX],
        }
    }
}

impl<const MAX: usize> MoveHistory for SmallMoveHistory<MAX> {
    fn last_move(&self) -> Move {
        if self.cursor == 0 {
            Move::None
        } else {
            self.moves[self.cursor as usize - 1].0
        }
    }

    fn push(&mut self, mv: Move, castle_rights: CastleRights, move_counter: MoveCounter) {
        self.moves[self.cursor as usize] = (mv, castle_rights, move_counter);
        self.cursor += 1;
    }

    fn pop(&mut self) -> (Move, CastleRights, MoveCounter) {
        self.cursor -= 1;
        self.moves[self.cursor as usize]
    }
}