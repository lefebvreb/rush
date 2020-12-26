use crate::game::Game;
use crate::moves::Move;

enum State {
    // if no check
    PawnCapture,
    RookCapture,
    KnightCapture,
    BishopCapture,
    QueenCapture,
    KingCapture,
    EnPassant,
    Castle,
    PawnPush,
    RookQuiet,
    KnightQuiet,
    BishopQuiet,
    QueenQuiet,
    KingQuiet,
    // goto Stop

    // if single check
    PawnCaptureSingleCheck,
    RookCaptureSingleCheck,
    KnightCaptureSingleCheck,
    BishopCaptureSingleCheck,
    QueenCaptureSingleCheck,
    KingCaptureSingleCheck,
    EnPassantSingleCheck,
    PawnBlock,
    RookBlock,
    KnightBlock,
    BishopBlock,
    QueenBlock,
    // goto KingEvade

    // if double check
    KingCaptureDoubleCheck,
    KingEvade,
    // goto Stop
    
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