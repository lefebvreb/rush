use std::ops::Range;

use chess::bitboard::BitBoard;
use chess::board::Board;
use chess::movegen;
use chess::moves::Move;
use chess::piece::Piece;

//#################################################################################################
//
//                                           struct MovePicker
//
//#################################################################################################

// Represents the state of a movepicker.
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

// A struct allowing semi-lazily move generation.
#[derive(Debug)]
pub(crate) struct MovePicker {
    state: GenState,
    zero_index: usize,
    check_mask: BitBoard,
}

// ================================ pub(crate) impl

impl MovePicker {
    // Creates a new MovePicker, for the current board and given buffer.
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

    // A state machine generating moves lazily.
    // If there is nothing left to generate, returns None.
    // If some moves where generated, returns some range,
    // giving the indexes of the generated moves in the buffer.
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
                        if self.check_mask.contains(to) {
                            buffer.push(Move::capture(from, to, capture));
                        }
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
                        if self.check_mask.contains(to) {
                            buffer.push(Move::quiet(from, to));
                        }
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
    // The list of under promotions, from best to worst.
    const UNDER_PROMOTES: [Piece; 3] = [Piece::Rook, Piece::Bishop, Piece::Knight];
}