use crate::game::Game;
use crate::moves::Move;

enum State {
    PawnCapture,
    RookCapture,
    KnightCapture,
    BishopCapture,
    QueenCapture,
    KingCapture,
    EnPassant,
    Castle,
    PawnPushes,
    RookQuiet,
    KnightQuiet,
    BishopQuiet,
    QueenQuiet,
    KingQuiet,

    
    
    Stop,
}

pub struct MoveGenerator {
    state: State
}

impl MoveGenerator {
    #[inline(always)]
    pub fn legals(game: &Game) -> MoveGenerator {
        todo!()
    }

    #[inline(always)]
    pub fn next(&mut self) -> Move {
        todo!()
    }

    #[cold]
    pub fn collect(&mut self) -> Vec<Move> {
        let mut res = Vec::new();

        loop {
            let mv = self.next();
            if mv.is_none() {
                break;
            } else {
                res.push(mv)
            }
        }

        res
    }
}