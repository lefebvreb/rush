use std::cmp::Ordering;

use chess::bitboard::BitBoard;
use chess::board::Board;
use chess::movegen;
use chess::moves::Move;
use chess::piece::Piece;

use crate::heuristics::Heuristics;

/// All under-prmotions.
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
    /// Rates a move assuming it is a castling move.
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
//                                    struct MovePicker
//
//#################################################################################################

/// A struct used to provide a layer of abstraction over move generation and picking.
/// Uses u16s instead of usizes to save space, since we won't go as far as 65536 moves anyway.
#[derive(Debug)]
pub(crate) struct MovePicker {
    state: MovePickerState,
    start: u16,
    end: u16,
}

// ================================ pub(crate) impl

impl MovePicker {
    /// Constructs a new move picker.
    #[inline]
    pub(crate) fn new(board: &Board, buffer: &Vec<RatedMove>) -> MovePicker {
        let len = buffer.len() as u16;

        MovePicker {
            state: MovePickerState::new(board),
            start: len,
            end: len,
        }
    }

    /// Returns the next pseudo-legal move to try, or None if there is no more moves for this position.
    #[inline]
    pub(crate) fn next(&mut self, board: &Board, heuristics: &Heuristics, depth: u8, buffer: &mut Vec<RatedMove>) -> Option<Move> {
        // If there were any leftovers move from a deeper node's MovePicker: forget them.
        // SAFE: we know the buffer has at least self.end elements already.
        unsafe {buffer.set_len(self.end as usize)};

        // There are no more moves in the buffer.
        if self.start == self.end {
            if self.gen_next_batch(board, heuristics, depth, buffer) {
                // A new batch was generated, sort the new moves.
                buffer[usize::from(self.start)..].sort_unstable_by(RatedMove::pseudo_cmp);
            } else {
                // The new batch was empty, return None.
                return None;
            }
        }

        // Return the last element of the buffer.
        self.end -= 1;
        buffer.pop().map(|rated| rated.mv)
    }

    /// Needs to be called after all moves have been consumed from the movepicker.
    #[inline]
    pub(crate) fn truncate(&self, buffer: &mut Vec<RatedMove>) {
        // SAFE: we know the buffer had at least self.start elements already.
        unsafe {buffer.set_len(self.start as usize)};
    }
}

impl MovePicker {
    fn gen_next_batch(&mut self, board: &Board, heuristics: &Heuristics, depth: u8, buffer: &mut Vec<RatedMove>) -> bool {
        loop {
            self.state = match self.state {
                // Only queen promotions and capture promotions.
                MovePickerState::QueenPromotes => {
                    movegen::gen_promote_captures(board, &[Piece::Queen], |mv| buffer.push(RatedMove::promote_capture(mv)));
                    movegen::gen_promotes(board, &[Piece::Queen], |mv| buffer.push(RatedMove::promote(mv)));
                    MovePickerState::Captures
                },
                // All captures, including en passant ones.
                MovePickerState::Captures => {
                    movegen::gen_pawn_captures(board, |mv| buffer.push(RatedMove::capture(Piece::Pawn, mv)));
                    movegen::gen_en_passant(board, |mv| buffer.push(RatedMove::capture(Piece::Pawn, mv)));
                    movegen::gen_captures(board, |piece, mv| buffer.push(RatedMove::capture(piece, mv)));
                    movegen::gen_king_captures(board, |mv| buffer.push(RatedMove::capture(Piece::King, mv)));
                    MovePickerState::Castles
                },
                // All castling.
                MovePickerState::Castles => {
                    movegen::gen_castles(board, |mv| buffer.push(RatedMove::castle(mv)));
                    MovePickerState::UnderPromotes
                },
                // All under promotions.
                MovePickerState::UnderPromotes => {
                    movegen::gen_promote_captures(board, UNDER_PROMOTES, |mv| buffer.push(RatedMove::promote_capture(mv)));
                    movegen::gen_promotes(board, UNDER_PROMOTES, |mv| buffer.push(RatedMove::promote(mv)));
                    MovePickerState::Quiets
                },
                // All quiets, including pushes and king ones.
                MovePickerState::Quiets => {
                    movegen::gen_pushes(board, |mv| buffer.push(heuristics.rate(mv, depth)));
                    movegen::gen_quiets(board, |_, mv| buffer.push(heuristics.rate(mv, depth)));
                    movegen::gen_king_quiets(board, |mv| buffer.push(heuristics.rate(mv, depth)));
                    MovePickerState::Stop
                },

                // All queen promotions under single check.
                MovePickerState::CheckQueenPromotes {mask} => {
                    movegen::gen_promote_captures(board, &[Piece::Queen], |mv| if mask.contains(mv.to()) {buffer.push(RatedMove::promote_capture(mv))});
                    movegen::gen_promotes(board, &[Piece::Queen], |mv| if mask.contains(mv.to()) {buffer.push(RatedMove::promote(mv))});
                    MovePickerState::CheckCaptures {mask}
                },
                // All captures under single check.
                MovePickerState::CheckCaptures {mask} => {
                    movegen::gen_pawn_captures(board, |mv| if mask.contains(mv.to()) {buffer.push(RatedMove::capture(Piece::Pawn, mv))});
                    movegen::gen_en_passant(board, |mv| if mask.contains(mv.to()) {buffer.push(RatedMove::capture(Piece::Pawn, mv))});
                    movegen::gen_captures(board, |piece, mv| if mask.contains(mv.to()) {buffer.push(RatedMove::capture(piece, mv))});
                    movegen::gen_king_captures(board, |mv| buffer.push(RatedMove::capture(Piece::King, mv)));
                    MovePickerState::CheckOthers {mask}
                },
                // All other moves under single check. 
                MovePickerState::CheckOthers {mask} => {
                    // Under promotions.
                    movegen::gen_promote_captures(board, UNDER_PROMOTES, |mv| if mask.contains(mv.to()) {buffer.push(RatedMove::promote_capture(mv))});
                    movegen::gen_promotes(board, UNDER_PROMOTES, |mv| if mask.contains(mv.to()) {buffer.push(RatedMove::promote(mv))});
                    
                    // Quiet moves.
                    movegen::gen_pushes(board, |mv| if mask.contains(mv.to()) {buffer.push(heuristics.rate(mv, depth))});
                    movegen::gen_quiets(board, |_, mv| if mask.contains(mv.to()) {buffer.push(heuristics.rate(mv, depth))});
                    movegen::gen_king_quiets(board, |mv| buffer.push(heuristics.rate(mv, depth)));

                    MovePickerState::Stop
                },

                // All moves under double check (only the king may move).
                MovePickerState::DoubleCheck => {
                    movegen::gen_king_captures(board, |mv| buffer.push(RatedMove::capture(Piece::King, mv)));
                    movegen::gen_king_quiets(board, |mv| buffer.push(heuristics.rate(mv, depth)));
                    MovePickerState::Stop
                },

                MovePickerState::Stop => return false,
            };

            let end = buffer.len() as u16;
            if self.start != end {
                self.end = end;
                return true;
            }
        }
    }
}

//#################################################################################################
//
//                                         enum MovePickerState
//
//#################################################################################################

/// The MovePickerState used for standard search, generates all pseudo-legals for a given position.
#[derive(Debug)]
pub(crate) enum MovePickerState {
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

impl MovePickerState {
    #[inline]
    fn new(board: &Board) -> MovePickerState {
        let checkers = board.get_checkers();

        if checkers.empty() {
            // No checkers, start by queen promotions.
            MovePickerState::QueenPromotes
        } else if checkers.more_than_one() {
            // Two checkers, no choice.
            MovePickerState::DoubleCheck
        } else {
            // One checker: compute the check mask: the checker's square or any square between them and the king.
            // SAFE: there is always a king on the board.
            let checker = unsafe {checkers.as_square_unchecked()};
            let mask = BitBoard::between(board.king_sq(), checker) | checkers;

            MovePickerState::CheckQueenPromotes {mask}
        }
    }
}

//#################################################################################################
//
//                                         struct Captures
//
//#################################################################################################

pub(crate) struct Captures {
    start: u16,
    end: u16,
}

impl Captures {
    #[inline]
    pub(crate) fn new(board: &Board, buffer: &mut Vec<RatedMove>) -> Captures {
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
            // SAFE: there is always a king on the board.
            let checker = unsafe {checkers.as_square_unchecked()};
            let mask = BitBoard::between(board.king_sq(), checker) | checkers;

            movegen::gen_promote_captures(board, &Piece::PROMOTES, |mv| if mask.contains(mv.to()) {buffer.push(RatedMove::promote_capture(mv))});
            movegen::gen_pawn_captures(board, |mv| if mask.contains(mv.to()) {buffer.push(RatedMove::capture(Piece::Pawn, mv))});
            movegen::gen_en_passant(board, |mv| if mask.contains(mv.to()) {buffer.push(RatedMove::capture(Piece::Pawn, mv))});
            movegen::gen_captures(board, |piece, mv| if mask.contains(mv.to()) {buffer.push(RatedMove::capture(piece, mv))});
            movegen::gen_king_captures(board, |mv| buffer.push(RatedMove::capture(Piece::King, mv)));
        }

        buffer[usize::from(start)..].sort_unstable_by(RatedMove::pseudo_cmp);

        Captures {
            start,
            end: buffer.len() as u16,
        }
    }

    #[inline]
    pub(crate) fn next(&mut self, buffer: &mut Vec<RatedMove>) -> Option<Move> {
        if self.start == self.end {
            None
        } else {
            self.end -= 1;
            buffer.pop().map(|rated| rated.mv)
        }
    }

    /// Needs to be called after all moves have been consumed from the movepicker.
    #[inline]
    pub(crate) fn truncate(&self, buffer: &mut Vec<RatedMove>) {
        // SAFE: we know the buffer had at least self.start elements already.
        unsafe {buffer.set_len(self.start as usize)};
    }
}