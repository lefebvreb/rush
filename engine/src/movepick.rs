use std::cmp::Ordering;

use chess::bitboard::BitBoard;
use chess::board::Board;
use chess::movegen;
use chess::moves::Move;
use chess::piece::Piece;

use crate::heuristics::Heuristics;

const UNDER_PROMOTES: &[Piece] = &[Piece::Rook, Piece::Bishop, Piece::Knight];

//#################################################################################################
//
//                                           struct RatedMove
//
//#################################################################################################

/// A struct representing a move along with it's heuristic value.
#[derive(Copy, Clone, Debug)]
pub(crate) struct RatedMove {
    pub(crate) mv: Move,
    pub(crate) score: f32,
}

// ================================ impl

impl RatedMove {
    /// Rates a castling move.
    #[inline]
    fn castle(mv: Move) -> RatedMove {
        RatedMove {mv, score: 0.0}
    }

    /// Rates a move assuming it is a capture promotion.
    #[inline]
    fn promote_capture(mv: Move) -> RatedMove {
        RatedMove {
            mv,
            score: f32::from(16 * mv.get_promote() as u8 + mv.get_capture() as u8)
        }
    }

    /// Rates a move assuming it is a normal promotion.
    #[inline]
    fn promote(mv: Move) -> RatedMove {
        RatedMove {
            mv,
            score: f32::from(mv.get_promote() as u8)
        }
    }

    /// Rates a move, assuming it is a capture, with the techinque of most valuabe capture for
    /// least valuable attacker.
    #[inline]
    fn capture(piece: Piece, mv: Move) -> RatedMove {
        RatedMove {
            mv,
            score: f32::from(16 * mv.get_capture() as u8 + 5 - piece as u8)
        }
    }

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

/// A trait to keep the code DRY.
pub(crate) trait MovePickerState {
    /// Creates a new state from a given board.
    fn new(board: &Board) -> Self;

    /// Must generate the next batch of moves and change self appropriately.
    /// Returns None if no moves were generated (meaning there is nothing left to generate),
    /// else returns Some(end) where end is the new length of the buffer.
    fn gen_next_batch(&mut self, board: &Board, heuristics: &Heuristics, depth: u8, buffer: &mut Vec<RatedMove>) -> Option<u16>;
}

//#################################################################################################
//
//                                    struct MovePicker
//
//#################################################################################################

/// A struct used to provide a layer of abstraction over move generation and picking.
/// Uses u16s instead of usizes to save space, since we won't go as far as 65536 moves anyway.
#[derive(Debug)]
pub(crate) struct MovePicker<T: MovePickerState> {
    state: T,
    start: u16,
    end: u16,
}

// ================================ pub(crate) impl

impl<T: MovePickerState> MovePicker<T> {
    /// Constructs a new move picker.
    #[inline]
    pub(crate) fn new(board: &Board, buffer: &Vec<RatedMove>) -> MovePicker<T> {
        let len = buffer.len() as u16;

        MovePicker {
            state: T::new(board),
            start: len,
            end: len,
        }
    }

    /// Returns the next pseudo-legal move to try, or None if there is no more moves for this position.
    #[inline]
    pub(crate) fn next(&mut self, board: &Board, heuristics: &Heuristics, depth: u8, buffer: &mut Vec<RatedMove>) -> Option<Move> {
        // If there were any leftovers move from a deeper node's MovePicker: forget them.
        // SAFE: we know the buffer has at least end_index elements already.
        unsafe {buffer.set_len(self.end as usize)};

        // There are no more moves in the buffer.
        if self.start == self.end {
            if let Some(end) = self.state.gen_next_batch(board, heuristics, depth, buffer) {
                // A new batch was generated, sort the new moves.
                self.end = end;
                buffer[(self.start as usize)..].sort_by(RatedMove::pseudo_cmp);
            } else {
                // The new batch was empty, return None.
                return None;
            }
        }

        // Return the last element of the buffer.
        self.end -= 1;
        buffer.pop().map(|rated| rated.mv)
    }
}

//#################################################################################################
//
//                                         enum Standard
//
//#################################################################################################

/// The MovePickerState used for standard search, generates all pseudo-legals for a given position.
#[derive(Debug)]
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

    // Nothing left.
    Stop,
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

    fn gen_next_batch(&mut self, board: &Board, heuristics: &Heuristics, depth: u8, buffer: &mut Vec<RatedMove>) -> Option<u16> {
        let start = buffer.len() as u16;

        loop {
            *self = match self {
                // Only queen promotions and capture promotions.
                Standard::QueenPromotes => {
                    movegen::gen_promote_captures(board, &[Piece::Queen], |mv| buffer.push(RatedMove::promote_capture(mv)));
                    movegen::gen_promotes(board, &[Piece::Queen], |mv| buffer.push(RatedMove::promote(mv)));
                    Standard::Captures
                },
                // All captures, including en passant ones.
                Standard::Captures => {
                    movegen::gen_pawn_captures(board, |mv| buffer.push(RatedMove::capture(Piece::Pawn, mv)));
                    movegen::gen_en_passant(board, |mv| buffer.push(RatedMove::capture(Piece::Pawn, mv)));
                    movegen::gen_captures(board, |piece, mv| buffer.push(RatedMove::capture(piece, mv)));
                    movegen::gen_king_captures(board, |mv| buffer.push(RatedMove::capture(Piece::King, mv)));
                    Standard::Castles
                },
                // All castling.
                Standard::Castles => {
                    movegen::gen_castles(board, |mv| buffer.push(RatedMove::castle(mv)));
                    Standard::UnderPromotes
                },
                // All under promotions.
                Standard::UnderPromotes => {
                    movegen::gen_promote_captures(board, UNDER_PROMOTES, |mv| buffer.push(RatedMove::promote_capture(mv)));
                    movegen::gen_promotes(board, UNDER_PROMOTES, |mv| buffer.push(RatedMove::promote(mv)));
                    Standard::Quiets
                },
                // All quiets, including pushes and king ones.
                Standard::Quiets => {
                    movegen::gen_pushes(board, |mv| buffer.push(heuristics.rate(mv, depth)));
                    movegen::gen_quiets(board, |_, mv| buffer.push(heuristics.rate(mv, depth)));
                    movegen::gen_king_quiets(board, |mv| buffer.push(heuristics.rate(mv, depth)));
                    Standard::Stop
                },

                // All queen promotions under single check.
                Standard::CheckQueenPromotes {mask} => {
                    movegen::gen_promote_captures(board, &[Piece::Queen], |mv| if mask.contains(mv.to()) {buffer.push(RatedMove::promote_capture(mv))});
                    movegen::gen_promotes(board, &[Piece::Queen], |mv| if mask.contains(mv.to()) {buffer.push(RatedMove::promote(mv))});
                    Standard::CheckCaptures {mask: *mask}
                },
                // All captures under single check.
                Standard::CheckCaptures {mask} => {
                    movegen::gen_pawn_captures(board, |mv| if mask.contains(mv.to()) {buffer.push(RatedMove::capture(Piece::Pawn, mv))});
                    movegen::gen_en_passant(board, |mv| if mask.contains(mv.to()) {buffer.push(RatedMove::capture(Piece::Pawn, mv))});
                    movegen::gen_captures(board, |piece, mv| if mask.contains(mv.to()) {buffer.push(RatedMove::capture(piece, mv))});
                    movegen::gen_king_captures(board, |mv| buffer.push(RatedMove::capture(Piece::King, mv)));
                    Standard::CheckOthers {mask: *mask}
                },
                // All other moves under single check. 
                Standard::CheckOthers {mask} => {
                    // Under promotions.
                    movegen::gen_promote_captures(board, UNDER_PROMOTES, |mv| if mask.contains(mv.to()) {buffer.push(RatedMove::promote_capture(mv))});
                    movegen::gen_promotes(board, UNDER_PROMOTES, |mv| if mask.contains(mv.to()) {buffer.push(RatedMove::promote(mv))});
                    
                    // Quiet moves.
                    movegen::gen_pushes(board, |mv| if mask.contains(mv.to()) {buffer.push(heuristics.rate(mv, depth))});
                    movegen::gen_quiets(board, |_, mv| if mask.contains(mv.to()) {buffer.push(heuristics.rate(mv, depth))});
                    movegen::gen_king_quiets(board, |mv| buffer.push(heuristics.rate(mv, depth)));

                    Standard::Stop
                },

                // All moves under double check (only the king may move).
                Standard::DoubleCheck => {
                    movegen::gen_king_captures(board, |mv| buffer.push(RatedMove::capture(Piece::King, mv)));
                    movegen::gen_king_quiets(board, |mv| buffer.push(heuristics.rate(mv, depth)));
                    Standard::Stop
                },

                Standard::Stop => return None,
            };

            let end = buffer.len() as u16;
            if start != end {
                return Some(end);
            }
        }
    }
}

//#################################################################################################
//
//                                         enum Quiescient
//
//#################################################################################################

/// The MovePickerState used for quiescient search, generates all pseudo-legal captures for a given position.
#[derive(Debug)]
pub(crate) struct Quiescient(bool);

impl MovePickerState for Quiescient {
    #[inline]
    fn new(_: &Board) -> Quiescient {
        Quiescient(false)
    }

    #[inline]
    fn gen_next_batch(&mut self, board: &Board, _: &Heuristics, _: u8, buffer: &mut Vec<RatedMove>) -> Option<u16> {
        if self.0 {
            return None;
        }
        self.0 = true;

        let start = buffer.len() as u16;

        let checkers = board.get_checkers();

        if checkers.empty() {
            // No checkers, do all captures, including promotion, en passant, pawn and king ones.
            movegen::gen_promote_captures(board, &Piece::PROMOTES, |mv| buffer.push(RatedMove::promote_capture(mv)));
            movegen::gen_pawn_captures(board, |mv| buffer.push(RatedMove::capture(Piece::Pawn, mv)));
            movegen::gen_en_passant(board, |mv| buffer.push(RatedMove::capture(Piece::Pawn, mv)));
            movegen::gen_captures(board, |piece, mv| buffer.push(RatedMove::capture(piece, mv)));
            movegen::gen_king_captures(board, |mv| buffer.push(RatedMove::capture(Piece::King, mv)));
        } else if checkers.more_than_one() {
            // Two checkers, only the king may capture.
            movegen::gen_king_captures(board, |mv| buffer.push(RatedMove::capture(Piece::King, mv)));
        } else {
            // One checker, must check that the move is inside the computed mask.
            let checker = unsafe {checkers.as_square_unchecked()};
            let mask = BitBoard::between(board.king_sq(), checker) | checkers;

            movegen::gen_promote_captures(board, &Piece::PROMOTES, |mv| if mask.contains(mv.to()) {buffer.push(RatedMove::promote_capture(mv))});
            movegen::gen_pawn_captures(board, |mv| if mask.contains(mv.to()) {buffer.push(RatedMove::capture(Piece::Pawn, mv))});
            movegen::gen_en_passant(board, |mv| if mask.contains(mv.to()) {buffer.push(RatedMove::capture(Piece::Pawn, mv))});
            movegen::gen_captures(board, |piece, mv| if mask.contains(mv.to()) {buffer.push(RatedMove::capture(piece, mv))});
            movegen::gen_king_captures(board, |mv| buffer.push(RatedMove::capture(Piece::King, mv)));
        }        

        let end = buffer.len() as u16;
        if start != end {
            Some(end)
        } else {
            None
        }
    }
}