use crate::castle_rights::CastleRights;
use crate::moves::Move;

// Represent a ply (half-turn) counter
#[derive(Copy, Clone, PartialEq, Default, Debug)]
pub struct Ply(u16);

impl Ply {
    // Increment the counter
    #[inline(always)]
    pub fn incr(&mut self) {
        self.0 += 1;
    }

    // Decrement the counter
    #[inline(always)]
    pub fn decr(&mut self) {
        self.0 -= 1;
    }
}

// A trait defining a move history aka a Vec of Move and CastleRights
pub trait MoveHistory: Default + 'static {
    // Return the last move in history
    fn last_move(&self) -> Move;

    // Push another move and castle_history to history
    fn push(&mut self, mv: Move, castle_rights: CastleRights);

    // Pop out the last element of the history
    fn pop(&mut self) -> (Move, CastleRights);
}

// A type to hold a large move history, for example one
// fit for an entire game
#[derive(Debug)]
pub struct LargeMoveHistory(Vec<(Move, CastleRights)>);

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
            Some((mv, _)) => *mv,
            _ => Move::None,
        }
    }

    #[inline(always)]
    fn push(&mut self, mv: Move, castle_rights: CastleRights) {
        self.0.push((mv, castle_rights));
    }

    #[inline(always)]
    fn pop(&mut self) -> (Move, CastleRights) {
        self.0.pop().unwrap()
    }
}

// A type to hold a small move history, for example one
// fit for exploring the game tree. MAX is the maximum
// number of elements that fit inside the history
#[derive(Debug)]
pub struct SmallMoveHistory<const MAX: usize> {
    cursor: usize,
    moves: [(Move, CastleRights); MAX],
}

impl<const MAX: usize> Default for SmallMoveHistory<MAX> {
    #[inline(always)]
    fn default() -> SmallMoveHistory<MAX> {
        SmallMoveHistory {
            cursor: 0,
            moves: [(Move::None, CastleRights::NONE); MAX],
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

    fn push(&mut self, mv: Move, castle_rights: CastleRights) {
        self.moves[self.cursor as usize] = (mv, castle_rights);
        self.cursor += 1;
    }

    fn pop(&mut self) -> (Move, CastleRights) {
        self.cursor -= 1;
        self.moves[self.cursor as usize]
    }
}