use crate::game::Game;
use crate::history::MoveHistory;
use crate::moves::Move;

impl<H: MoveHistory> Game<H> {
    /// Try to parse a position from fen notation
    #[cold]
    pub fn from_fen(fen: &str) -> Result<Game<H>, ()> {
        todo!()
    }

    /// Produce a fen string corresponding to that position
    #[cold]
    pub fn to_fen(&self) -> String {
        todo!()
    }

    /// Try to parse a move from current position with given coordinates,
    /// in pure algebraic notation, of course
    #[cold]
    pub fn parse_move(&self, coords: &str) -> Result<Move, ()> {
        todo!()
    }
}