use crate::board::Board;
use crate::list::List;
use crate::moves::Move;

type MoveList = List<Move, 256>;

impl Board {
    /// Generates all pseudo-legal captures and queen promotion.
    /// Combined with gen_quiets(), this function gives all pseudo-legal moves
    /// of a not-check position.
    /// Precondition: there are no checkers.
    pub fn gen_captures(&self, list: &mut MoveList) {
        todo!()
    }

    /// Generates all pseudo-legal non-captures moves and underpiece promotions.
    /// Combined with gen_captures(), this function gives all pseudo-legal moves
    /// of a not-check position.
    /// Precondition: there are no checkers.
    pub fn gen_quiets(&self, list: &mut MoveList) {
        todo!()
    }

    /// Generates all pseudo-legal non-king moves of a position
    /// if there is one checker.
    /// Combined with gen_evasions(), this function gives all pseudo-legal moves
    /// of a one checker position.
    /// Precondition: there is one checker, and exactly one.
    pub fn gen_blocks(&self, list: &mut MoveList) {
        todo!()
    }

    /// Generates all pseudo-legal king moves of a position with
    /// potentially multiple checkers.
    /// Combined with gen_blocks(), this function gives all pseudo-legal moves
    /// of a one checker position.
    /// Alone, this functions gives all moves of a two-checkers position.
    /// Precondition: there is one checker, and exactly one.
    pub fn gen_evasions(&self, list: &mut MoveList) {
        todo!()
    }
}