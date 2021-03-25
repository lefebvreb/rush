use std::borrow::Borrow;

use chess::{Game, Move, MoveGenerator, Zobrist};

use crate::eval::eval;
use crate::params;
use crate::shared::{self, Entry, NodeFlag, table_insert};

//#################################################################################################
//
//                                     struct Search
//
//#################################################################################################

// A helper struct to keep all of a thread's information about a search
pub(crate) struct Search {
    depth: u8,
    keys: [Zobrist; params::MAX_DEPTH as usize],
    best_move: Option<Move>,
}

// ================================ pub(crate) impl

impl Search {
    // The main search loop of a single thread, lauching the alpha-beta search
    // with an increasingly wide aspiration window
    pub(crate) fn search_position(&mut self) {
        let game = shared::game();

        let best_score = self.quiescence(game.clone(), f32::NEG_INFINITY, f32::INFINITY);
        
        'search: loop {
            let search_depth = shared::search_depth();

            let mut alpha = best_score - params::ASPIRATION_WINDOW[0];
            let mut beta  = best_score + params::ASPIRATION_WINDOW[0];
        
            let mut alpha_index = 1;
            let mut beta_index  = 1;

            loop {
                let best_score = self.alpha_beta(game.clone(), alpha, beta, true, search_depth, search_depth);
            
                if shared::should_stop() || shared::search_depth() >= search_depth {
                    break 'search;
                }

                if best_score <= alpha {
                    alpha = best_score - params::ASPIRATION_WINDOW[alpha_index.min(3)];
                    alpha_index += 1;
                } else if best_score >= beta {
                    beta = best_score + params::ASPIRATION_WINDOW[beta_index.min(3)];
                    beta_index += 1;
                } else {
                    break;
                }
            }

            if let Some(mv) = self.best_move {
                shared::report_move(mv, search_depth);
            }
        }
    }
}

// ================================ impl

impl Search {
    // Return true if the game is pseudo-drawn, meaning likely a draw:
    // detect the 50 moves rules, but consider a game drawn if the zobrist
    // key of the game has already occured during that branch of the search.
    // Is used to reduce the quantity of nodes to explore
    #[inline(always)]
    fn is_pseudodraw(&self, game: &Game) -> bool {
        game.get_clock().halfmoves() == 100 || 
            (0..self.depth)
            .any(|i| self.keys[i as usize] == game.get_zobrist())
    }

    // Put another key in the list
    #[inline(always)]
    fn next(&mut self, game: &Game, mv: Move) {
        self.keys[self.depth as usize] = game.get_zobrist();
        self.depth += 1;
    }

    // Remove the last key pushed
    #[inline(always)]
    fn prev(&mut self) {
        self.depth -= 1;
    } 

    // Return the value of the position, computed with a quiescent search (only considering captures)
    fn quiescence<G: Borrow<Game>>(&mut self, game: G, mut alpha: f32, beta: f32) -> f32 {
        let game = game.borrow();

        if self.is_pseudodraw(game) {
            return 0.0;
        }
        
        let stand_pat = eval(&game);

        if self.depth >= params::MAX_DEPTH {
            return stand_pat;
        }

        if stand_pat >= beta {
            return beta;
        }
        alpha = alpha.min(stand_pat);

        let mut legals = game.legals();

        while let Some(mv) = legals.next() {
            if !mv.is_capture() {
                break;
            }

            self.next(&game, mv);
            let score = -self.quiescence(game.do_move(mv), -beta, -alpha);
            self.prev();

            if shared::should_stop() {
                return 0.0;
            }

            if score >= beta {
                return beta;
            }
            alpha = alpha.max(score);
        }
        
        alpha
    }

    // The alpha-beta negamax algorithm, with a few more heuristics in it
    pub(crate) fn alpha_beta<G: Borrow<Game>>(&mut self, game: G, mut alpha: f32, beta: f32, do_null: bool, mut depth: u8, search_depth: u8) -> f32 {
        let game = game.borrow();
        
        if depth == 0 {
            return self.quiescence(game, alpha, beta);
        }

        if self.is_pseudodraw(game) {
            return 0.0;
        }

        if self.depth >= params::MAX_DEPTH {
            return eval(&game);
        }

        if let Some((entry, score)) = shared::table_probe(game.get_zobrist(), alpha, beta, depth) {
            if score >= alpha && self.depth == 0 {
                self.best_move = Some(entry.mv);
            }
            return score;
        }

        let old_alpha = alpha;
        let in_check = game.in_check();

        if in_check {
            depth += 1;
        } else if do_null && self.depth > 0 && depth >= 4 {
            // TODO: avoid endgame position heuristic
            // and do null move heuristic
        }

        if let Some(entry) = shared::table_get(game.get_zobrist()) {
            // TODO: if entry.move is valid, do it first
        }

        let mut best_score = f32::NEG_INFINITY;
        let mut legals = game.legals();
        let mut best_move = None;
        let mut moves = 0u8;

        while let Some(mv) = legals.next() {
            self.next(&game, mv);
            let score = -self.alpha_beta(game.do_move(mv), -beta, -alpha, do_null, depth-1, search_depth);
            self.prev();

            if shared::should_stop() || shared::search_depth() >= search_depth {
                return 0.0;
            }

            if score > best_score {
                best_score = score;
                best_move = Some(mv);

                if score > alpha {
                    if score >= beta {
                        if !mv.is_capture() {
                            // TODO: killer heuristic
                        }

                        table_insert(game.get_zobrist(), Entry {
                            mv, 
                            score: beta, 
                            age: game.get_clock().ply(), 
                            depth, 
                            flag: NodeFlag::Beta, 
                        });

                        return beta;
                    }

                    alpha = score;
                }
            }

            moves += 1;
        }

        if moves == 0 {
            return if in_check {
                -params::MATE_SCORE + self.depth as f32
            } else {
                0.0
            }
        }

        if alpha != old_alpha {
            table_insert(game.get_zobrist(), Entry {
                mv: best_move.unwrap(), 
                score: best_score, 
                age: game.get_clock().ply(), 
                depth, 
                flag: NodeFlag::Exact, 
            });

            if self.depth == 0 {
                self.best_move = best_move;
            }
        } else {
            table_insert(game.get_zobrist(), Entry {
                mv: best_move.unwrap(), 
                score: alpha, 
                age: game.get_clock().ply(), 
                depth, 
                flag: NodeFlag::Alpha, 
            });
        }

        alpha
    }
}

// ================================ traits impl

impl Default for Search {
    fn default() -> Search {
        Search {
            depth: 0,
            keys: [Zobrist::default(); params::MAX_DEPTH as usize],
            best_move: None,
        }
    }
}