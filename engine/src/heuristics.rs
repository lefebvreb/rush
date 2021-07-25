use chess::moves::Move;

use crate::movepick::RatedMove;
use crate::params::MAX_DEPTH;

/// A struct keeping track of the various moves ordering heuristics.
#[derive(Debug)]
pub(crate) struct Heuristics {
    // Two killer moves.
    killers: [[Option<Move>; 2]; MAX_DEPTH],
    // History heuristic table.
    history: [[f32; 64]; 64],
}

// ================================ pub(crate) impl

impl Heuristics {
    /// Constructs a new Heuristics object.
    pub(crate) fn new() -> Heuristics {
        Heuristics {
            killers: [[None; 2]; MAX_DEPTH],
            history: [[0.0; 64]; 64],
        }
    }

    #[inline]
    /// Store a new killer move, replacing the oldest one.
    pub(crate) fn store_killer(&mut self, mv: Move, depth: u8) {
        let depth = usize::from(depth);
        self.killers[depth][0] = self.killers[depth][1];
        self.killers[depth][1] = Some(mv);
    }

    #[inline]
    /// Updates the history for a move that is played by the given color
    pub(crate) fn update_history(&mut self, mv: Move, depth: u8) {
        let depth = f32::from(depth);
        self.history[usize::from(mv.from())][usize::from(mv.to())] += depth * depth;
    }

    #[inline]
    /// Rates a given quiet move.
    pub(crate) fn rate(&self, mv: Move, depth: u8) -> RatedMove {
        let score = if self.killers[usize::from(depth)][0].map_or(false, |killer| killer == mv) {
            9000000.0
        } else if self.killers[usize::from(depth)][1].map_or(false, |killer| killer == mv) {
            8000000.0
        } else {
            self.history[usize::from(mv.from())][usize::from(mv.to())]
        };

        RatedMove {mv, score}
    }
}