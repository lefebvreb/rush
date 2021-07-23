use std::cmp::Ordering;

use chess::bitboard::BitBoard;
use chess::board::Board;
use chess::moves::Move;

//#################################################################################################
//
//                                           struct RatedMove
//
//#################################################################################################

/// A struct representing a move along with it's heuristic value.
#[derive(Copy, Clone, Debug)]
pub(crate) struct RatedMove {
    mv: Move,
    score: f32,
}

// ================================ impl

impl RatedMove {
    /// Compares the two moves scores, we simply assume that no floats here are infinite.
    #[inline]
    fn pseudo_cmp(&self, rhs: &RatedMove) -> Ordering {
        if self.score < rhs.score {
            Ordering::Less
        } else if self.score > rhs.score {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

//#################################################################################################
//
//                                       trait MovePickerState
//
//#################################################################################################

trait MovePickerState {
    /// Creates a new state from a given board.
    fn new(board: &Board) -> Self;

    /// Must generate the next batch of moves and change self appropriately.
    /// Returns None if no moves were generated (meaning there is nothing left to generate),
    /// else returns Some(end) where end is the new length of the buffer.
    fn gen_next_batch(&mut self, board: &Board, buffer: &mut Vec<RatedMove>) -> Option<u16>;
}

//#################################################################################################
//
//                                    struct MovePicker
//
//#################################################################################################

/// A struct used to provide a layer of abstraction over move generation and picking.
/// Uses u16s instead of usizes to save space, since we won't go as far as 65536 moves anyway.
pub(crate) struct MovePicker<T: MovePickerState> {
    state: T,
    start: u16,
    end: u16,
}

// ================================ pub(crate) impl

impl<T: MovePickerState> MovePicker<T> {
    /// Constructs a new move picker.
    #[inline]
    fn new(board: &Board, buffer: &mut Vec<RatedMove>) -> MovePicker<T> {
        let len = buffer.len() as u16;

        MovePicker {
            state: T::new(board),
            start: len,
            end: len,
        }
    }

    /// Returns the next pseudo-legal move to try, or None if there is no more moves for this position.
    fn next(&mut self, board: &Board, buffer: &mut Vec<RatedMove>) -> Option<Move> {
        // If there were any leftovers move from a deeper node's MovePicker: forget them.
        // SAFE: we know the buffer has at least end_index elements already.
        unsafe {buffer.set_len(self.end as usize)};

        // There are no more moves in the buffer.
        if self.start == self.end {
            if let Some(end) = self.state.gen_next_batch(board, buffer) {
                // A new batch was generated, sort the new moves.
                self.end = end;
                &buffer[(self.start as usize)..].sort_by(RatedMove::pseudo_cmp);
            } else {
                // The new batch was empty, return None.
                return None;
            }
        }

        // Return the last element of the buffer.
        buffer.pop().map(|rated| rated.mv)
    }
}

//#################################################################################################
//
//                                   struct StandardGen
//
//#################################################################################################

/// The MovePickerState used for standard search, generates all pseudo-legals for a given position.
pub(crate) enum Standard {
    // No checkers.
    QueenPromotes,
    Captures,
    Castles,
    UnderPromotes,
    Quiets,

    // One checker: store the mask in which pieces must move.
    CheckQueenPromotes {mask: BitBoard},
    CheckCaptures {mask: BitBoard},
    CheckOthers {mask: BitBoard},

    // Two checkers.
    DoubleCheck,
}

impl MovePickerState for Standard {
    #[inline]
    fn new(board: &Board) -> Standard {
        let checkers = board.get_checkers();

        if checkers.empty() {
            // No checkers, start by queen promotions.
            Standard::QueenPromotes
        } else if checkers.more_than_one() {
            // Two checkers, no choice.
            Standard::DoubleCheck
        } else {
            // One checker: compute the check mask: the checker's square or any square between them and the king.
            let checker = unsafe {checkers.as_square_unchecked()};
            let mask = BitBoard::between(board.king_sq(), checker) | checkers;

            Standard::CheckQueenPromotes {mask}
        }
    }

    #[inline]
    fn gen_next_batch(&mut self, board: &Board, buffer: &mut Vec<RatedMove>) -> Option<u16> {
        loop {
            match self {
                Standard::QueenPromotes => todo!(),
                Standard::Captures => todo!(),
                Standard::Castles => todo!(),
                Standard::UnderPromotes => todo!(),
                Standard::Quiets => todo!(),

                Standard::CheckQueenPromotes {mask} => todo!(),
                Standard::CheckCaptures {mask} => todo!(),
                Standard::CheckOthers {mask} => todo!(),

                Standard::DoubleCheck => todo!(),
            }
        }
    }
}

// ================================ pub(crate) impl

/*
pub(crate) trait MovePicker {
    /// Creates a new MovePicker, that will generate moves
    fn new(board: &Board, buffer: &Vec<Move>) -> Self;

    fn start_index(&self) -> usize;

    fn next_batch(&mut self, board: &Board, buffer: &mut Vec<Move>) -> Option<Range<usize>>;

    fn next(&mut self, board: &Board, buffer: &mut Vec<Move>) -> Option<Move> {
        if buffer.len() == self.start_index() {

        }

        todo!()
    }
}
*/
/*
//#################################################################################################
//
//                                           struct MovePicker
//
//#################################################################################################

/// Represents the state of a movepicker.
#[derive(Debug)]
enum GenState {
    // No checkers.
    QueenPromotes,
    Captures,
    Castles,
    UnderPromotes,
    Quiets,

    // One checker.
    CheckQueenPromotes,
    CheckCaptures,
    CheckOthers,

    // Two checkers.
    DoubleCheck,

    // When there is nothing left to generate.
    End,
}

/// A struct allowing semi-lazily move generation.
#[derive(Debug)]
pub(crate) struct MovePicker {
    state: GenState,
    zero_index: usize,
    check_mask: BitBoard,
}

// ================================ pub(crate) impl

impl MovePicker {
    /// Creates a new MovePicker, for the current board and given buffer.
    #[inline]
    pub(crate) fn new(board: &Board, buffer: &[Move]) -> MovePicker {
        let zero_index = buffer.len();
        
        let checkers = board.get_checkers();

        if checkers.empty() {
            MovePicker {
                state: GenState::QueenPromotes, 
                zero_index, 
                check_mask: BitBoard::default(),
            }            
        } else if checkers.more_than_one() {
            MovePicker {
                state: GenState::DoubleCheck, 
                zero_index, 
                check_mask: BitBoard::default(),
            }     
        } else {
            let checker = unsafe {checkers.as_square_unchecked()};
            let check_mask = BitBoard::between(board.king_sq(), checker) | checkers;
            MovePicker {
                state: GenState::CheckQueenPromotes, 
                zero_index, 
                check_mask,
            }
        }
    }

    /// A state machine generating moves lazily.
    /// If there is nothing left to generate, returns None.
    /// If some moves where generated, returns some range,
    /// giving the indexes of the generated moves in the buffer.
    pub(crate) fn next(&mut self, board: &Board, buffer: &mut Vec<Move>) -> Option<Range<usize>> {
        // Remove the last batch from the list.
        buffer.truncate(self.zero_index);

        loop {
            // Compute that stage and go to the next one.
            self.state = match self.state {
                // --- No checkers ---
                GenState::QueenPromotes => {
                    // All queen promotions, captures and non-captures.
                    movegen::gen_promote_captures(board, |from, to, capture| {
                        buffer.push(Move::promote_capture(from, to, capture, Piece::Queen));
                    });
                    movegen::gen_promotes(board, |from, to| {
                        buffer.push(Move::promote(from, to, Piece::Queen));
                    });
                    GenState::Captures
                },
                GenState::Captures => {
                    // All captures, incuding en passant captures.
                    movegen::gen_en_passant(board, |from, to| {
                        buffer.push(Move::en_passant(from, to));
                    });
                    movegen::gen_pawn_captures(board, |from, to, capture| {
                        buffer.push(Move::capture(from, to, capture));
                    });
                    movegen::gen_captures(board, |from, to, capture| {
                        buffer.push(Move::capture(from, to, capture));
                    });
                    movegen::gen_king_captures(board, |from, to, capture| {
                        buffer.push(Move::capture(from, to, capture));
                    });
                    GenState::Castles
                },
                GenState::Castles => {
                    // All castling moves.
                    movegen::gen_castles(board, |from, to| {
                        buffer.push(Move::castle(from, to));
                    });
                    GenState::UnderPromotes
                },
                GenState::UnderPromotes => {
                    // --- All under promotions ---
                    movegen::gen_promote_captures(board, |from, to, capture| {
                        for promote in MovePicker::UNDER_PROMOTES {
                            buffer.push(Move::promote_capture(from, to, capture, promote));
                        }
                    });
                    movegen::gen_promotes(board, |from, to| {
                        for promote in MovePicker::UNDER_PROMOTES {
                            buffer.push(Move::promote(from, to, promote));
                        }
                    });
                    GenState::Quiets
                },
                GenState::Quiets => {
                    // All quiet moves.
                    movegen::gen_pushes(board, |from, to, is_double| {
                        if is_double {
                            buffer.push(Move::double_push(from, to));
                        } else {
                            buffer.push(Move::quiet(from, to));
                        }
                    });
                    movegen::gen_quiets(board, |from, to| {
                        buffer.push(Move::quiet(from, to));
                    });
                    movegen::gen_king_quiets(board, |from, to| {
                        buffer.push(Move::quiet(from, to));
                    });
                    GenState::End
                },
    
                // --- One checker ---
                GenState::CheckQueenPromotes => {
                    // All queen promotions, captures and non-captures.
                    movegen::gen_promote_captures(board, |from, to, capture| {
                        if self.check_mask.contains(to) {
                            buffer.push(Move::promote_capture(from, to, capture, Piece::Queen))
                        };
                    });
                    movegen::gen_promotes(board, |from, to| {
                        if self.check_mask.contains(to) {
                            buffer.push(Move::promote(from, to, Piece::Queen));
                        };
                    });
                    GenState::CheckCaptures
                },
                GenState::CheckCaptures => {
                    // All captures.
                    movegen::gen_en_passant(board, |from, to| {
                        if self.check_mask.contains(to) {
                            buffer.push(Move::en_passant(from, to));
                        }
                    });
                    movegen::gen_pawn_captures(board, |from, to, capture| {
                        if self.check_mask.contains(to) {
                            buffer.push(Move::capture(from, to, capture));
                        }
                    });
                    movegen::gen_captures(board, |from, to, capture| {
                        if self.check_mask.contains(to) {
                            buffer.push(Move::capture(from, to, capture));
                        }
                    });
                    movegen::gen_king_captures(board, |from, to, capture| {
                        buffer.push(Move::capture(from, to, capture));
                    });
                    GenState::CheckOthers
                },
                GenState::CheckOthers => {
                    // Under promotions.
                    movegen::gen_promote_captures(board, |from, to, capture| {
                        for promote in MovePicker::UNDER_PROMOTES {
                            if self.check_mask.contains(to) {
                                buffer.push(Move::promote_capture(from, to, capture, promote))
                            };
                        }
                    });
                    movegen::gen_promotes(board, |from, to| {
                        for promote in MovePicker::UNDER_PROMOTES {
                            if self.check_mask.contains(to) {
                                buffer.push(Move::promote(from, to, promote));
                            };
                        }
                    });
                    // All quiet moves.
                    movegen::gen_pushes(board, |from, to, is_double| {
                        if self.check_mask.contains(to) {
                            if is_double {
                                buffer.push(Move::double_push(from, to));
                            } else {
                                buffer.push(Move::quiet(from, to));
                            }
                        }
                    });
                    movegen::gen_quiets(board, |from, to| {
                        if self.check_mask.contains(to) {
                            buffer.push(Move::quiet(from, to));
                        }
                    });
                    movegen::gen_king_quiets(board, |from, to| {
                        buffer.push(Move::quiet(from, to));
                    });
                    GenState::End
                },
    
                // --- Two checkers ---
                GenState::DoubleCheck => {
                    // Only the king may move: captures then quiets, in one batch.
                    movegen::gen_king_captures(board, |from, to, capture| {
                        buffer.push(Move::capture(from, to, capture));
                    });
                    movegen::gen_king_quiets(board, |from, to| {
                        buffer.push(Move::quiet(from, to));
                    });
                    GenState::End
                },
    
                // Nothing left to yield: return None.
                GenState::End => return None,
            };

            // If anything was generated, return.
            // Else, the loop goes for another round.
            if buffer.len() != self.zero_index {
                return Some(self.zero_index..buffer.len());
            }
        }        
    }
}

// ================================ impl

impl MovePicker {
    /// The list of under promotions, from best to worst.
    const UNDER_PROMOTES: [Piece; 3] = [Piece::Rook, Piece::Bishop, Piece::Knight];
}*/