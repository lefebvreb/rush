use chess::{Game, Move, MoveGenerator, Zobrist};

use crate::eval::eval;
use crate::params;
use crate::shared::{self, Entry, NodeFlag, table_insert};

//#################################################################################################
//
//                                     struct Search
//
//#################################################################################################

pub(crate) struct Search {
    depth: u8,
    last_irresversible: u8,
    keys: [Zobrist; params::MAX_DEPTH as usize],
    best_move: Option<Move>,
}

// ================================ pub(crate) impl

impl Search {
    pub(crate) fn quiescence(&mut self, game: Game, mut alpha: f32, beta: f32) -> f32 {
        if self.is_pseudodraw(&game) {
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

            self.depth += 1;
            let score = -self.quiescence(game.do_move(mv), -beta, -alpha);
            self.depth -= 1;

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

    pub(crate) fn alpha_beta(&mut self, game: Game, mut alpha: f32, beta: f32, do_null: bool, mut depth: u8, search_depth: u8) -> f32 {
        if depth == 0 {
            return self.quiescence(game, alpha, beta);
        }

        if self.is_pseudodraw(&game) {
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
            self.depth += 1;
            let score = -self.alpha_beta(game.do_move(mv), -beta, -alpha, do_null, depth-1, search_depth);
            self.depth -= 1;

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

    pub(crate) fn search_position(&mut self) {
        todo!()
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
            (self.last_irresversible..self.depth)
            .any(|i| self.keys[i as usize] == game.get_zobrist())
    }
}

// ================================ traits impl

impl Default for Search {
    fn default() -> Search {
        todo!()
    }
}