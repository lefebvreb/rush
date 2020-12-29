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

// A trait defining a move history aka a Vec of Move
pub trait MoveHistory: Default + 'static {
    // Return the last move in history
    fn last(&self) -> Move;

    // Push another move to history
    fn push(&mut self, mv: Move);

    // Pop out the last move in history
    fn pop(&mut self) -> Move;
}

// A type to hold a large move history, for example one
// fit for an entire game
#[derive(Debug)]
pub struct LargeMoveHistory(Vec<Move>);

impl Default for LargeMoveHistory {
    #[cold]
    fn default() -> LargeMoveHistory {
        LargeMoveHistory(Vec::with_capacity(128))
    }
}

impl MoveHistory for LargeMoveHistory {
    #[inline(always)]
    fn last(&self) -> Move {
        match self.0.last() {
            Some(mv) => *mv,
            _ => Move::None,
        }
    }

    #[inline(always)]
    fn push(&mut self, mv: Move) {
        self.0.push(mv);
    }

    #[inline(always)]
    fn pop(&mut self) -> Move {
        self.0.pop().unwrap()
    }
}

// A type to hold a small move history, for example one
// fit for exploring the game tree
#[derive(Debug)]
pub struct SmallMoveHistory<const MAX: usize> {
    cursor: u8,
    moves: [Move; MAX],
}

impl<const MAX: usize> Default for SmallMoveHistory<MAX> {
    #[inline(always)]
    fn default() -> SmallMoveHistory<MAX> {
        SmallMoveHistory {
            cursor: 0,
            moves: [Move::None; MAX],
        }
    }
}

impl<const MAX: usize> MoveHistory for SmallMoveHistory<MAX> {
    fn last(&self) -> Move {
        if self.cursor == 0 {
            Move::None
        } else {
            self.moves[self.cursor as usize - 1]
        }
    }

    fn push(&mut self, mv: Move) {
        self.moves[self.cursor as usize] = mv;
        self.cursor += 1;
    }

    fn pop(&mut self) -> Move {
        self.cursor -= 1;
        self.moves[self.cursor as usize]
    }
}