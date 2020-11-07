use crate::board::Board;
use crate::castle_rights::CastleRights;
use crate::color::Color;
use crate::moves::Move;

#[derive(Clone, Debug, Default)]
pub struct Game {
    board: Board,
    castle_rights: CastleRights,
    current_color: Color,
    move_history: Vec<Move>,
}
