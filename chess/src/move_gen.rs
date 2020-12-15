use crate::game::Game;
use crate::moves::Move;

enum State {

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