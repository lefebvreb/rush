use crate::bitboard::BitBoard;
use crate::color::Color;
use crate::moves::MoveType;

/* ======== MEMO ===========

1. The castling must be kingside or queenside.
2. Neither the king nor the chosen rook has previously moved.
3. There are no pieces between the king and the chosen rook.
4. The king is not currently in check.
5. The king does not pass through a square that is attacked by an enemy piece.
6. The king does not end up in check. (True of any legal move.)

========================= */

#[derive(Clone, Debug)]
pub struct CastleRights {
    
}

impl CastleRights {
    #[inline]
    pub fn can_kingcastle(&self, color: Color, attacks: BitBoard) -> bool {
        match color {
            Color::White => {
                todo!()
            },
            Color::Black => {
                todo!()
            }
        }
    }

    #[inline]
    pub fn can_queencastle(&self, color: Color, attacks: BitBoard) -> bool {
        todo!()
    }

    #[inline]
    pub fn do_move(&mut self, color: Color, mv: &MoveType) {
        todo!()
    }

    #[inline]
    pub fn undo_move(&mut self, color: Color, mv: &MoveType) {
        todo!()
    }
}

impl Default for CastleRights {
    fn default() -> Self {
        todo!()
    }
}
