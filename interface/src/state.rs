use std::collections::HashMap;

use chess::{Game, Move, Square};

use crate::assets::Assets;
use crate::render::{render_buttons, render_last_move, render_moves, render_pieces, render_promotion, render_squares};

#[derive(PartialEq)]
pub enum State {
    SelectingPiece,
    SelectingDestination,
    SelectingPromotion,
    Waiting,
    Checkmate,
}

pub struct App {
    game: Game,
    moves: Vec<Move>,
    select: Option<Square>,
    available_moves: HashMap<Square, Move>,
    last_move: Option<(Square, Square)>,
    state: State,
    reversed: bool,
}

impl App {
    pub fn act(&mut self, assets: &Assets) {
        render_buttons(assets);
        if self.state == State::SelectingPromotion {
            render_promotion(assets, self.game.get_color());
        }
        render_squares(assets, self.reversed);
        if let Some((from, to)) = self.last_move {
            render_last_move(assets, from, to, self.reversed)
        }
        if let Some(select) = self.select {
            render_moves(assets, select, self.available_moves.keys().map(|sq| *sq), self.reversed);
        }
        render_pieces(assets, self.game.get_board(), self.reversed);

        /*match self.state {
            State::Waiting => {
                // Handle AI stuff here
            }
            _ => (),
        }

        if let Some(click) = get_click(self.reversed) {
            println!("{:?}", click);

            match self.state {
                State::SelectingPiece => if let Click::Square(square) = click {
                    if let Some((color, piece)) = self.game.get_board().get_piece(square) {
                        if color == self.game.get_color() {
                            self.select = Some(square);
                            self.available_moves.clear();
                            for mv in &self.moves {
                                if mv.from(color) == square {
                                    self.available_moves.insert(mv.to(color), *mv);
                                }
                            }
                            self.state = State::SelectingDestination;
                        }
                    }
                }
                State::SelectingDestination => {
                    
                }
                State::SelectingPromotion => {
                    
                }
                _ => ()
            }
        }*/
    }
}

impl Default for App {
    fn default() -> App {
        /*let game = Game::default();

        let (state, moves, reversed) = if rand() & 1 == 0 {
            (State::SelectingPiece, game.get_moves().collect(), false)
        } else {
            (State::Waiting, vec![], true)
        };

        App {
            game,
            moves,
            select: None,
            available_moves: HashMap::new(),
            last_move: None,
            state,
            reversed,
        }*/
        todo!()
    }
}