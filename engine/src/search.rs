use std::sync::Arc;

use chess::board::Board;
use chess::moves::Move;

use crate::engine::GlobalInfo;
use crate::params;

// A struct holding all the necessary information for a search thread.
pub(crate) struct Search {
    info: Arc<GlobalInfo>,
    best_move: Option<Move>,
    depth: u8,
    board: Board,
}

// ================================ pub(crate) impl

impl Search {
    // Creates a new search struct, ready to bes used for searching the game tree.
    pub(crate) fn new(info: Arc<GlobalInfo>) -> Search {
        Search {
            info,
            best_move: None,
            depth: 0,
            board: Board::default(),
        }
    }

    // The loop run by threads
    pub(crate) fn thread_main(&mut self) {
        loop {
            // The start barrier.
            self.info.wait();
    
            // The stop flag was set: we must return from this function. The thread will be joined.
            if self.info.should_stop() {
                return;
            }
    
            // Search the position while the flag is on.
            self.search_position();
    
            // The end search barrier.
            self.info.wait();
        }
    }

}

// ================================ impl

impl Search {
    // Search the position until told to stop.
    fn search_position(&mut self) {
        // Clone global board and get search depth.
        const MAX_IDX: usize = params::ASPIRATION_WINDOW.len() - 1;
        
        self.board = self.info.board();
        
        let best_score = self.quiescence(f32::NEG_INFINITY, f32::INFINITY);
        
        'search: loop {
            let search_depth = self.info.thread_search_depth();
            
            let mut alpha = best_score - params::ASPIRATION_WINDOW[0];
            let mut beta  = best_score + params::ASPIRATION_WINDOW[0];
            
            let (mut alpha_idx, mut beta_idx) = (0, 0);
            
            loop {
                let best_score = self.alpha_beta(alpha, beta, true, search_depth, search_depth);
                
                if !self.info.is_searching() {
                    break 'search;
                }
                
                if self.info.search_depth() >= search_depth {
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
                self.info.report_move(mv, search_depth);
            }
        }
    }
    
    // The alpha-beta negamax algorithm, with a few more heuristics in it.
    pub(crate) fn alpha_beta(&mut self, alpha: f32, beta: f32, do_null: bool, depth: u8, search_depth: u8) -> f32 {        
        /*if depth == 0 {
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
        
        alpha*/
    
        todo!()
    }

    // Return the value of the position, computed with a quiescent search (only considering captures).
    fn quiescence(&mut self, alpha: f32, beta: f32) -> f32 {
        /*if self.is_pseudodraw(game) {
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
        
        alpha*/

        todo!()
    }
}
