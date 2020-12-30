use crate::board::Board;
use crate::castle_rights::CastleRights;
use crate::color::Color;
use crate::moves::Move;
use crate::history::{LargeMoveHistory, MoveHistory, Ply, SmallMoveHistory};

/// A struct that holds every information defining a complete game of chess
#[derive(Debug)]
pub struct Game<H: MoveHistory> {
    board: Board,
    castle_rights: CastleRights,
    color: Color,
    history: H,
    ply: Ply,
}

impl<H: MoveHistory> Game<H> {
    /// Perform a new move and modifiy the game accordingly
    #[inline]
    pub fn do_move(&mut self, mv: Move) {
        self.board.do_move(self.color, mv);
        self.castle_rights.do_move(mv, self.ply);

        self.history.push(mv);
        self.color = self.color.invert();

        self.ply.incr();
    }

    /// Revert the last move. Panic if there is no move in history
    #[inline]
    pub fn undo_move(&mut self) {
        self.ply.decr();

        self.color = self.color.invert();
        let mv = self.history.pop();
        
        self.castle_rights.undo_move(mv, self.ply);
        self.board.undo_move(self.color, mv);
    }

    /// Return the game's board
    #[inline(always)]
    pub fn get_board(&self) -> &Board {
        &self.board
    }

    /// Return the color of the current player
    #[inline(always)]
    pub fn get_color(&self) -> Color {
        self.color
    }

    // Return the castling rights
    #[inline(always)]
    pub(crate) fn get_castle_rights(&self) -> &CastleRights {
        &self.castle_rights
    }

    // Return the last move played
    #[inline(always)]
    pub(crate) fn get_last_move(&self) -> Move {
        self.history.last()
    }
}

impl Game<LargeMoveHistory> {
    /// Return a new SearchGame, able to be used
    #[inline(always)]
    pub fn search_game<const MAX: usize>(&self) -> SearchGame<MAX> {
        Game {
            board: self.board.clone(),
            castle_rights: self.castle_rights.clone(),
            color: self.color.clone(),
            history: SmallMoveHistory::default(),
            ply: self.ply.clone(),
        }
    }
}

impl Default for FullGame {
    #[cold]
    fn default() -> FullGame {
        Game {
            board: Board::default(),
            castle_rights: CastleRights::default(),
            color: Color::default(),
            history: LargeMoveHistory::default(),
            ply: Ply::default(),
        }
    }
}

/// A type used to record a full game from beginning to end
pub type FullGame = Game<LargeMoveHistory>;

/// A type used to explore the game tree, in which only MAX moves
/// may be played
pub type SearchGame<const MAX: usize> = Game<SmallMoveHistory<MAX>>;