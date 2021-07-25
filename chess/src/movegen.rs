use crate::attacks;
use crate::bitboard::BitBoard;
use crate::board::Board;
use crate::castle_rights::CastleMask;
use crate::color::Color;
use crate::en_passant::EnPassantSquare;
use crate::moves::Move;
use crate::piece::Piece;
use crate::square::Square;

//#################################################################################################
//
//                                      Generation Primitives
//
//#################################################################################################

// ================================ pawns pseudo-legals

/// Gives all pseudo-legal promote captures for pawns, promoting to the given pieces.
/// The provided closure is called for all generated moves.
#[inline]
pub fn gen_promote_captures(board: &Board, promotes: &[Piece], mut gen: impl FnMut(Move)) {
    let us = board.get_side_to_move();
    let them = board.get_other_side();

    for from in (board.get_bitboard(us, Piece::Pawn) & BitBoard::promote_rank(us)).iter_squares() {
        for to in (attacks::pawn(us, from) & board.get_occupancy().colored(them)).iter_squares() {
            for &promote in promotes {
                gen(Move::promote_capture(from, to, board.get_piece_unchecked(to), promote));
            }
        }
    }
}

/// Gives all pseudo-legal promotions for pawns, promoting to the given pieces.
/// The provided closure is called for all generated moves.
#[inline]
pub fn gen_promotes(board: &Board, promotes: &[Piece], mut gen: impl FnMut(Move)) {
    let us = board.get_side_to_move();

    for from in (board.get_bitboard(us, Piece::Pawn) & BitBoard::promote_rank(us)).iter_squares() {
        let to = attacks::pawn_push(us, from).unwrap();
        if board.get_piece(to).is_none() {
            for &promote in promotes {
                gen(Move::promote(from, to, promote));
            }
        }
    }
}

/// Gives all pseudo-legal en passant moves.
/// The provided closure is called for all generated moves.
#[inline]
pub fn gen_en_passant(board: &Board, mut gen: impl FnMut(Move)) {
    let us = board.get_side_to_move();
    let them = board.get_other_side();

    if let EnPassantSquare::Some(sq) = board.get_ep_square() {
        let to = attacks::pawn_push(us, sq).unwrap();
        for from in (attacks::pawn(them, to) & board.get_bitboard(us, Piece::Pawn)).iter_squares() {
            gen(Move::en_passant(from, to));
        }
    }
}

/// Gives all pseudo-legal captures from pawns.
/// The provided closure is called for all generated moves.
#[inline]
pub fn gen_pawn_captures(board: &Board, mut gen: impl FnMut(Move)) {
    let us = board.get_side_to_move();
    let them = board.get_other_side();

    let them_occ = board.get_occupancy().colored(them);
    
    for from in (board.get_bitboard(us, Piece::Pawn) & !BitBoard::promote_rank(us)).iter_squares() {
        for to in (attacks::pawn(us, from) & them_occ).iter_squares() {
            gen(Move::capture(from, to, board.get_piece_unchecked(to)));
        }
    }
}

/// Gives all pseudo-legals pushes and double pushes from pawns.
/// The provided closure is called for all generated moves.
#[inline]
pub fn gen_pushes(board: &Board, mut gen: impl FnMut(Move)) {
    let us = board.get_side_to_move();

    for from in (board.get_bitboard(us, Piece::Pawn) & !BitBoard::promote_rank(us)).iter_squares() {
        if let Some(to1) = attacks::pawn_push(us, from) {
            if board.get_piece(to1).is_none() {
                gen(Move::quiet(from, to1));
                if let Some(to2) = attacks::pawn_double_push(us, from) {
                    if board.get_piece(to2).is_none() {
                        gen(Move::double_push(from, to2));
                    }
                }
            }
        }
    }
}

// ================================ king pseudo-legals

/// Gives all pseudo-legal captures from the king.
/// The provided closure is called for all generated moves.
#[inline]
pub fn gen_king_captures(board: &Board, mut gen: impl FnMut(Move)) {
    let them_occ = board.get_occupancy().colored(board.get_other_side());

    let from = board.king_sq();
    for to in (attacks::king(from) & them_occ).iter_squares() {
        gen(Move::capture(from, to, board.get_piece_unchecked(to)));
    }
}

/// Gives all pseudo-legal quiets from the king.
/// The provided closure is called for all generated moves.
#[inline]
pub fn gen_king_quiets(board: &Board, mut gen: impl FnMut(Move)) {
    let free = board.get_occupancy().free();

    let from = board.king_sq();
    for to in (attacks::king(from) & free).iter_squares() {
        gen(Move::quiet(from, to));
    }
}

/// Gives all pseudo-legal castling moves.
/// The provided closure is called for all generated moves.
#[inline]
pub fn gen_castles(board: &Board, mut gen: impl FnMut(Move)) {
    let us = board.get_side_to_move();
    let castle_rights = board.get_castle_rights();

    match us {
        Color::White => {
            if castle_rights.has(CastleMask::WhiteOO) & board.is_path_clear(Square::E1, Square::H1) {
                gen(Move::castle(Square::E1, Square::G1));
            }
            if castle_rights.has(CastleMask::WhiteOOO) & board.is_path_clear(Square::E1, Square::A1) {
                gen(Move::castle(Square::E1, Square::C1));
            }
        },
        Color::Black => {
            if castle_rights.has(CastleMask::BlackOO) & board.is_path_clear(Square::E8, Square::H8) {
                gen(Move::castle(Square::E8, Square::G8));
            }
            if castle_rights.has(CastleMask::BlackOOO) & board.is_path_clear(Square::E8, Square::A8) {
                gen(Move::castle(Square::E8, Square::C8));
            }
        },
    }
}

// ================================ other moves

/// Gives all pseudo-legal captures from knights, bishops, rooks and queens.
/// The provided closure is called for all generated moves, and given the piece that moved.
#[inline]
pub fn gen_captures(board: &Board, mut gen: impl FnMut(Piece, Move)) {
    let us = board.get_side_to_move();
    let them = board.get_other_side();

    let occ = board.get_occupancy().all();
    let them_occ = board.get_occupancy().colored(them);

    for from in board.get_bitboard(us, Piece::Knight).iter_squares() {
        for to in (attacks::knight(from) & them_occ).iter_squares() {
            let mv = Move::capture(from, to, board.get_piece_unchecked(to));
            gen(Piece::Knight, mv);
        }
    }
    for from in board.get_bitboard(us, Piece::Bishop).iter_squares() {
        for to in (attacks::bishop(from, occ) & them_occ).iter_squares() {
            let mv = Move::capture(from, to, board.get_piece_unchecked(to));
            gen(Piece::Bishop, mv);
        }
    }
    for from in board.get_bitboard(us, Piece::Rook).iter_squares() {
        for to in (attacks::rook(from, occ) & them_occ).iter_squares() {
            let mv = Move::capture(from, to, board.get_piece_unchecked(to));
            gen(Piece::Rook, mv);
        }
    }
    for from in board.get_bitboard(us, Piece::Queen).iter_squares() {
        for to in (attacks::queen(from, occ) & them_occ).iter_squares() {
            let mv = Move::capture(from, to, board.get_piece_unchecked(to));
            gen(Piece::Queen, mv);
        }
    }
}

/// Gives all pseudo-legal quiets from knights, bishops, rooks and queens.
/// The provided closure is called for all generated moves, and given the piece that moved.
#[inline]
pub fn gen_quiets(board: &Board, mut gen: impl FnMut(Piece, Move)) {
    let us = board.get_side_to_move();

    let occ = board.get_occupancy().all();
    let free = board.get_occupancy().free();

    for from in board.get_bitboard(us, Piece::Knight).iter_squares() {
        for to in (attacks::knight(from) & free).iter_squares() {
            let mv = Move::quiet(from, to);
            gen(Piece::Knight, mv);
        }
    }
    for from in board.get_bitboard(us, Piece::Bishop).iter_squares() {
        for to in (attacks::bishop(from, occ) & free).iter_squares() {
            let mv = Move::quiet(from, to);
            gen(Piece::Bishop, mv);
        }
    }
    for from in board.get_bitboard(us, Piece::Rook).iter_squares() {
        for to in (attacks::rook(from, occ) & free).iter_squares() {
            let mv = Move::quiet(from, to);
            gen(Piece::Rook, mv);
        }
    }
    for from in board.get_bitboard(us, Piece::Queen).iter_squares() {
        for to in (attacks::queen(from, occ) & free).iter_squares() {
            let mv = Move::quiet(from, to);
            gen(Piece::Queen, mv);
        }
    }
}

//#################################################################################################
//
//                                         fn legals()
//
//#################################################################################################

/// Generates all legal moves for the current position, and pushes them at the end of the buffer, 
/// in no particular order.
pub fn legals(board: &Board, buffer: &mut Vec<Move>) {
    // Generates all non-king moves with the given consumer.
    pub fn gen_non_king(board: &Board, mut gen: impl FnMut(Move)) {
        gen_promote_captures(board, &Piece::PROMOTES, |mv| gen(mv));
        gen_en_passant(board, |mv| gen(mv));
        gen_pawn_captures(board, |mv| gen(mv));
        gen_promotes(board, &Piece::PROMOTES, |mv| gen(mv));
        gen_pushes(board, |mv| gen(mv));
        gen_captures(board, |_, mv| gen(mv));
        gen_quiets(board, |_, mv| gen(mv));
    }

    let checkers = board.get_checkers();

    let mut gen = |mv| if board.is_legal(mv) {buffer.push(mv)};

    if checkers.empty() {
        // No checkers.

        // Generate all castling and king moves. 
        gen_castles(board, |mv| gen(mv));
        gen_king_captures(board, |mv| gen(mv));
        gen_king_quiets(board, |mv| gen(mv));

        // Generates all other moves.
        gen_non_king(board, gen);
    } else if checkers.is_one() {
        // One checker.

        // Generate all king moves.
        gen_king_captures(board, |mv| gen(mv));
        gen_king_quiets(board, |mv| gen(mv));

        // Check that the move is either capturing the checker or blocking it.
        // SAFE: there is always a king on the board.
        let checker = unsafe {checkers.as_square_unchecked()};
        let mask = BitBoard::between(board.king_sq(), checker) | checkers;
        let gen = |mv: Move| if mask.contains(mv.to()) && board.is_legal(mv) {buffer.push(mv)};

        // Generate.
        gen_non_king(board, gen);
    } else {
        // Two checkers.

        // Only generate king moves.
        gen_king_captures(board, |mv| gen(mv));
        gen_king_quiets(board, |mv| gen(mv));
    }
}

//#################################################################################################
//
//                                         fn perft()
//
//#################################################################################################

/// Counts the number of leaf nodes of a given position and a given game tree depth.
pub fn perft(board: &mut Board, depth: usize) -> u64 {
    // The real perft function, optimized by bulk counting.
    pub fn internal_perft(board: &mut Board, buffer: &mut Vec<Move>, depth: usize) -> u64 {
        let start_index = buffer.len();
        legals(board, buffer);

        let total = if depth == 1 {
            (buffer.len() - start_index) as u64
        } else {
            let mut count = 0;

            for i in start_index..buffer.len() {
                let mv = buffer[i];

                board.do_move(mv);
                count += internal_perft(board, buffer, depth - 1);
                board.undo_move(mv);
            }

            count
        };

        // SAFE: we had at least start_index moves prior to calling this function
        unsafe {buffer.set_len(start_index)};

        total
    }

    // The internal function will panic if depth is 0.
    if depth == 0 {
        1
    } else {
        internal_perft(board, &mut Vec::new(), depth)
    }
}