use std::ops::{Generator, GeneratorState};
use std::pin::Pin;

//use crate::attacks::{double_push, get_pinned, pawn_push, pin_mask, squares_between};
//use crate::bitboard::BitBoard;
//use crate::castle_rights::CastleAvailability;
//use crate::color::Color;
use crate::game::Game;
use crate::moves::Move;
//use crate::piece::Piece;
//use crate::square::Square;

#[repr(u8)]
#[derive(Debug)]
enum PawnPromote {
    Queen,
    Rook,
    Bishop,
    Knight,
}

impl PawnPromote {
    #[inline(always)]
    pub fn next(&mut self) {
        *self = match self {
            PawnPromote::Queen => PawnPromote::Rook,
            PawnPromote::Rook => PawnPromote::Bishop,
            PawnPromote::Bishop => PawnPromote::Knight,
            PawnPromote::Knight => PawnPromote::Queen,
        };
    }
}

pub trait MoveGenerator {
    fn next(&mut self) -> Move;

    #[cold]
    fn collect(&mut self) -> Vec<Move> {
        (0..)
            .map(|_| self.next())
            .take_while(|mv| match mv {
                Move::None => false,
                _ => true,
            })
            .collect()
    }

    fn test() {

    }
}

impl<G: Generator<(), Yield=Move, Return=()> + Unpin> MoveGenerator for G {
    #[inline(always)]
    fn next(&mut self) -> Move {
        match Pin::new(self).resume(()) {
            GeneratorState::Yielded(mv) => mv,
            GeneratorState::Complete(_) => Move::None,
        }
    }
}

impl Game {
    #[inline(always)]
    pub fn legals(&self) -> impl MoveGenerator {
        move || {
            yield Move::None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opening() {
        let game = Game::default();

        let moves = game.legals().collect();

        println!("{:?}", moves.len());
        println!("{:?}", moves);
    }
}