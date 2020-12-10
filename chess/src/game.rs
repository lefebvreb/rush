use crate::board::Board;
use crate::castle_rights::CastleRights;
use crate::color::Color;
use crate::moves::Move;
use crate::ply::Ply;

#[derive(Debug)]
pub struct Game {
    pub(crate) board: Board,
    pub(crate) castle_rights: CastleRights,
    pub(crate) color: Color,
    pub(crate) history: Vec<Move>,
    pub(crate) ply: Ply,
}

impl Game {
    /// Perform a new move and modifiy the game accordingly
    #[inline]
    pub fn do_move(&mut self, mv: Move) {
        self.board.do_move(self.color, mv);
        self.castle_rights.do_move(self.color, mv, self.ply);

        self.history.push(mv);
        self.color = self.color.invert();

        self.ply.incr();
    }

    /// Revert the last move. Panic if there is no move in history
    #[inline]
    pub(crate) fn undo_move(&mut self) {
        self.ply.decr();

        self.color = self.color.invert();
        let mv = self.history.pop().expect("Trying to revert nothing");
        
        self.castle_rights.undo_move(self.color, mv, self.ply);
        self.board.undo_move(self.color, mv);
    }

    #[cold]
    pub fn get_board(&self) -> &Board {
        &self.board
    }

    #[cold]
    pub fn get_color(&self) -> Color {
        self.color
    }
}

impl Default for Game {
    #[cold]
    fn default() -> Game {
        Game {
            board: Board::default(),
            castle_rights: CastleRights::default(),
            color: Color::default(),
            history: Vec::with_capacity(128),
            ply: Ply::default(),
        }
    }
}

impl Clone for Game {
    #[inline]
    fn clone(&self) -> Game {
        Game {
            board: self.board.clone(),
            castle_rights: self.castle_rights.clone(),
            color: self.color.clone(),
            history: Vec::with_capacity(24),
            ply: Ply::default(),
        }
    }
}