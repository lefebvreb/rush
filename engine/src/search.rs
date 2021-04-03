use chess::{Game, Move, MoveGenerator, Piece, Zobrist};

use crate::eval::eval;
use crate::params::{self, PawnValue};
use crate::shared::{self, Entry, NodeFlag};

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
        const MAX_IDX: usize = params::ASPIRATION_WINDOW.len() - 1;

        let game = shared::game();

        let best_score = self.quiescence(game, f32::NEG_INFINITY, f32::INFINITY);

        'search: loop {
            let search_depth = shared::next_search_depth();

            let mut alpha = best_score - params::ASPIRATION_WINDOW[0];
            let mut beta  = best_score + params::ASPIRATION_WINDOW[0];

            let (mut alpha_idx, mut beta_idx) = (0, 0);

            loop {
                let best_score = self.alpha_beta(game, alpha, beta, true, search_depth, search_depth);
            
                if shared::should_stop() {
                    break 'search;
                }

                if shared::search_depth() >= search_depth {
                    break;
                }

                if best_score <= alpha {
                    alpha_idx = MAX_IDX.min(alpha_idx + 1);
                    alpha = best_score - params::ASPIRATION_WINDOW[alpha_idx];
                } else if best_score >= beta {
                    beta_idx = MAX_IDX.min(beta_idx + 1);
                    beta = best_score + params::ASPIRATION_WINDOW[beta_idx];
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
    fn next(&mut self, game: &Game) {
        self.keys[self.depth as usize] = game.get_zobrist();
        self.depth += 1;
    }

    // Remove the last key pushed
    #[inline(always)]
    fn prev(&mut self) {
        self.depth -= 1;
    } 

    // Return the value of the position, computed with a quiescent search (only considering captures)
    fn quiescence(&mut self, game: &Game, mut alpha: PawnValue, beta: PawnValue) -> PawnValue {
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

        // Big delta pruning
        let mut big_delta = params::value_of(Piece::Queen);
        if game.may_promote() {
            big_delta += params::value_of(Piece::Queen) - params::value_of(Piece::Pawn);
        }

        if stand_pat < alpha - big_delta {
            return alpha;
        }

        alpha = alpha.max(stand_pat);

        let mut legals = game.legals();

        while let Some(mv) = legals.next() {
            if let Some(capture) = mv.get_capture() {
                // Delta pruning
                if params::value_of(capture) + params::DELTA < alpha {
                    continue;
                }
            } else {
                break;
            }

            self.next(&game);
            let score = -self.quiescence(&game.do_move(mv), -beta, -alpha);
            self.prev();

            if shared::should_stop() {
                return 0.0;
            }

            if score > alpha {
                if score >= beta {
                    return beta;
                }
                alpha = score;
            }
        }
        
        alpha
    }

    // The alpha-beta negamax algorithm, with a few more heuristics in it
    pub(crate) fn alpha_beta(&mut self, game: &Game, mut alpha: PawnValue, beta: PawnValue, do_null: bool, mut depth: u8, search_depth: u8) -> PawnValue {        
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
            // Null-move pruning
            if !(game.in_check() || game.is_endgame()) {
                self.next(&game);
                let null_score = -self.alpha_beta(&game.do_null_move(), -beta, -beta + params::value_of(Piece::Pawn), false, depth - 4, search_depth);
                self.prev();

                if null_score >= beta {
                    return beta;
                }
            }
        }

        if let Some(entry) = shared::table_get(game.get_zobrist()) {
            // TODO: if entry.move is valid, do it first
        }

        let mut best_score = f32::NEG_INFINITY;
        let mut legals = game.legals();
        let mut best_move = None;
        let mut moves = 0u8;

        while let Some(mv) = legals.next() {
            self.next(&game);
            let score = -self.alpha_beta(&game.do_move(mv), -beta, -alpha, do_null, depth-1, search_depth);
            self.prev();

            if shared::should_stop() || shared::search_depth() >= search_depth {
                return 0.0;
            }

            if score > best_score {
                best_score = score;
                best_move = Some(mv);

                if score > alpha {
                    if score >= beta {
                        if mv.get_capture().is_none() {
                            // TODO: killer heuristic
                        }

                        shared::table_insert(game.get_zobrist(), Entry {
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
                -params::value_of(Piece::King) + self.depth as PawnValue
            } else {
                0.0
            };
        }

        if alpha != old_alpha {
            shared::table_insert(game.get_zobrist(), Entry {
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
            shared::table_insert(game.get_zobrist(), Entry {
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