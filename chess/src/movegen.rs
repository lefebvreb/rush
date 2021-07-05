use crate::attacks;
use crate::bitboard::BitBoard;
use crate::board::Board;
use crate::castle_rights::CastleMask;
use crate::color::Color;
use crate::en_passant::EnPassantSquare;
use crate::list::List;
use crate::moves::Move;
use crate::piece::Piece;
use crate::square::Square;

/// A type alias representing a movelist, of a fixed maximum capacity.
pub type MoveList = List<Move, 256>;

//#################################################################################################
//
//                                      Generation Primitives
//
//#################################################################################################

// ================================ pawns pseudo-legals

/// Gives all pseudo-legals promote captures for pawns.
/// The provided closure takes three arguments: from square, to square
/// and captured piece.
/// It is called for each pseudo-legal promote-capture.
#[inline]
pub fn gen_promote_captures(board: &Board, mut gen: impl FnMut(Square, Square, Piece)) {
    let us = board.get_side_to_move();
    let them = board.get_other_side();

    for from in (board.get_bitboard(us, Piece::Pawn) & BitBoard::promote_rank(us)).iter_squares() {
        for to in (attacks::pawn(us, from) & board.get_occupancy().colored(them)).iter_squares() {
            gen(from, to, board.piece_unchecked(to));
        }
    }
}

/// Gives all pseudo-legals promotion for pawns.
/// The provided closure takes two arguments: from square and to square.
/// It is called for each pseudo-legal promotion.
#[inline]
pub fn gen_promotes(board: &Board, mut gen: impl FnMut(Square, Square)) {
    let us = board.get_side_to_move();
    let them = board.get_other_side();

    for from in (board.get_bitboard(us, Piece::Pawn) & BitBoard::promote_rank(us)).iter_squares() {
        let to = attacks::pawn_push(us, from).unwrap();
        if board.get_piece(to).is_none() {
            gen(from, to);
        }
    }
}

/// Gives all pseudo-legals en passant moves.
/// The provided closure takes two arguments: from square and to square.
/// It is called for each pseudo-legal en passant.
#[inline]
pub fn gen_en_passant(board: &Board, mut gen: impl FnMut(Square, Square)) {
    let us = board.get_side_to_move();
    let them = board.get_other_side();

    if let EnPassantSquare::Some(sq) = board.get_ep_square() {
        let to = attacks::pawn_push(us, sq).unwrap();
        for from in (attacks::pawn(them, to) & board.get_bitboard(us, Piece::Pawn)).iter_squares() {
            gen(from, to);
        }
    }
}

/// Gives all pseudo-legals pawn captures.
/// The provided closure takes three arguments: from square, to square
/// and the captured piece.
/// It is called for each pseudo-legal pawn captures.
#[inline]
pub fn gen_pawn_captures(board: &Board, mut gen: impl FnMut(Square, Square, Piece)) {
    let us = board.get_side_to_move();
    let them = board.get_other_side();

    let them_occ = board.get_occupancy().colored(them);
    
    for from in (board.get_bitboard(us, Piece::Pawn) & !BitBoard::promote_rank(us)).iter_squares() {
        for to in (attacks::pawn(us, from) & them_occ).iter_squares() {
            gen(from, to, board.piece_unchecked(to));
        }
    }
}

/// Gives all pseudo-legals pushes and double pushes moves.
/// The provided closure takes three arguments: from square, to square
/// and a bool which is true if the move is a double push.
/// It is called for each pseudo-legal pushes and double pushes.
#[inline]
pub fn gen_pushes(board: &Board, mut gen: impl FnMut(Square, Square, bool)) {
    let us = board.get_side_to_move();

    for from in (board.get_bitboard(us, Piece::Pawn) & !BitBoard::promote_rank(us)).iter_squares() {
        if let Some(to1) = attacks::pawn_push(us, from) {
            if board.get_piece(to1).is_none() {
                gen(from, to1, false);
                if let Some(to2) = attacks::pawn_double_push(us, from) {
                    if board.get_piece(to2).is_none() {
                        gen(from, to2, true);
                    }
                }
            }
        }
    }
}

// ================================ king pseudo-legals

/// Gives all pseudo-legals captures from the king.
/// The provided closure takes three arguments: from square, to square
/// and the captured piece.
/// It is called for each pseudo-legal capture from the king.
#[inline]
pub fn gen_king_captures(board: &Board, mut gen: impl FnMut(Square, Square, Piece)) {
    let them_occ = board.get_occupancy().colored(board.get_other_side());

    let from = board.king_sq();
    for to in (attacks::king(from) & them_occ).iter_squares() {
        gen(from, to, board.piece_unchecked(to));
    }
}

/// Gives all pseudo-legals quiets from the king.
/// The provided closure takes two arguments: from square and to square.
/// It is called for each pseudo-legal quiets from the king.
#[inline]
pub fn gen_king_quiets(board: &Board, mut gen: impl FnMut(Square, Square)) {
    let free = board.get_occupancy().free();

    let from = board.king_sq();
    for to in (attacks::king(from) & free).iter_squares() {
        gen(from, to);
    }
}

// Generates all pseudo-legals castling moves for the position.
/// The provided closure takes two arguments: from square and to square.
/// It is called for each pseudo-legal castling.
#[inline]
pub fn gen_castles(board: &Board, mut gen: impl FnMut(Square, Square)) {
    let us = board.get_side_to_move();
    let castle_rights = board.get_castle_rights();

    match us {
        Color::White => {
            if castle_rights.has(CastleMask::WhiteOO) & board.is_path_clear(Square::E1, Square::H1) {
                gen(Square::E1, Square::G1);
            }
            if castle_rights.has(CastleMask::WhiteOOO) & board.is_path_clear(Square::E1, Square::A1) {
                gen(Square::E1, Square::C1);
            }
        },
        Color::Black => {
            if castle_rights.has(CastleMask::BlackOO) & board.is_path_clear(Square::E8, Square::H8) {
                gen(Square::E8, Square::G8);
            }
            if castle_rights.has(CastleMask::BlackOOO) & board.is_path_clear(Square::E8, Square::A8) {
                gen(Square::E8, Square::C8);
            }
        },
    }
}

// ================================ other moves

/// Gives all pseudo-legals captures from knights, bishops, rooks and queens.
/// The provided closure takes three arguments: from square, to square
/// and captured piece.
/// It is called for each pseudo-legal capture.
#[inline]
pub fn gen_captures(board: &Board, mut gen: impl FnMut(Square, Square, Piece)) {
    let us = board.get_side_to_move();
    let them = board.get_other_side();

    let occ = board.get_occupancy().all();
    let them_occ = board.get_occupancy().colored(them);

    for from in board.get_bitboard(us, Piece::Knight).iter_squares() {
        for to in (attacks::knight(from) & them_occ).iter_squares() {
            gen(from, to, board.piece_unchecked(to));
        }
    }
    for from in board.get_bitboard(us, Piece::Bishop).iter_squares() {
        for to in (attacks::bishop(from, occ) & them_occ).iter_squares() {
            gen(from, to, board.piece_unchecked(to));
        }
    }
    for from in board.get_bitboard(us, Piece::Rook).iter_squares() {
        for to in (attacks::rook(from, occ) & them_occ).iter_squares() {
            gen(from, to, board.piece_unchecked(to));
        }
    }
    for from in board.get_bitboard(us, Piece::Queen).iter_squares() {
        for to in (attacks::queen(from, occ) & them_occ).iter_squares() {
            gen(from, to, board.piece_unchecked(to));
        }
    }
}

/// Gives all pseudo-legals quiets from knights, bishops, rooks and queens.
/// The provided closure takes two arguments: from square and to square.
/// It is called for each pseudo-legal quiets.
#[inline]
pub fn gen_quiets(board: &Board, mut gen: impl FnMut(Square, Square)) {
    let us = board.get_side_to_move();
    let them = board.get_other_side();

    let occ = board.get_occupancy().all();
    let free = board.get_occupancy().free();

    for from in board.get_bitboard(us, Piece::Knight).iter_squares() {
        for to in (attacks::knight(from) & free).iter_squares() {
            gen(from, to);
        }
    }
    for from in board.get_bitboard(us, Piece::Bishop).iter_squares() {
        for to in (attacks::bishop(from, occ) & free).iter_squares() {
            gen(from, to);
        }
    }
    for from in board.get_bitboard(us, Piece::Rook).iter_squares() {
        for to in (attacks::rook(from, occ) & free).iter_squares() {
            gen(from, to);
        }
    }
    for from in board.get_bitboard(us, Piece::Queen).iter_squares() {
        for to in (attacks::queen(from, occ) & free).iter_squares() {
            gen(from, to);
        }
    }
}

//#################################################################################################
//
//                                         Legals Generation
//
//#################################################################################################

/// Generates all legal moves for the current position.
/// This function is rather slow. Use the other movegen functions
/// for more control over generation and better performance.
pub fn legals(board: &Board, list: &mut MoveList) {
    // The pieces a pawn promotes to, in order from most to least interesting.
    const PROMOTES: [Piece; 4] = [
        Piece::Queen, Piece::Rook, Piece::Bishop, Piece::Knight,
    ];

    // Bitboard of the checkers.
    let checkers = board.get_checkers();
    
    // A closure to check that the move is legal before pushing it
    // to the list.
    let mut push = |mv| if board.is_legal(mv) {list.push(mv)};

    match checkers.count() {
        0 => {
            // King moves.
            gen_castles(board, |from, to| push(Move::castle(from, to)));
            gen_king_captures(board, |from, to, capture| push(Move::capture(from, to, capture)));
            gen_king_quiets(board, |from, to| push(Move::quiet(from, to)));

            // Pawn moves.
            gen_promote_captures(board, |from, to, capture| {
                for promote in PROMOTES {
                    push(Move::promote_capture(from, to, capture, promote));
                }
            });
            gen_en_passant(board, |from, to| push(Move::en_passant(from, to)));
            gen_pawn_captures(board, |from, to, capture| push(Move::capture(from, to, capture)));
            gen_promotes(board, |from, to| {
                for promote in PROMOTES {
                    push(Move::promote(from, to, promote));
                }
            });
            gen_pushes(board, |from, to, is_double| {
                if is_double {
                    push(Move::double_push(from, to));
                } else {
                    push(Move::quiet(from, to));
                }
            });

            // Other moves.
            gen_captures(board, |from, to, capture| push(Move::capture(from, to, capture)));
            gen_quiets(board, |from, to| push(Move::quiet(from, to)));
        },
        1 => {
            // King moves.
            gen_king_captures(board, |from, to, capture| push(Move::capture(from, to, capture)));
            gen_king_quiets(board, |from, to| push(Move::quiet(from, to)));

            // If there is a single checker, we must also check that the move is either blocking
            // (in between the king and the checker) or that it is capturing the checker.
            // Or that the king itself is moving.
            let mask = BitBoard::between(board.king_sq(), checkers.as_square_unchecked()) | checkers;
            let mut push = |mv: Move| if mask.contains(mv.to()) && board.is_legal(mv) {list.push(mv)};

            // Pawn moves.
            gen_promote_captures(board, |from, to, capture| {
                for promote in PROMOTES {
                    push(Move::promote_capture(from, to, capture, promote));
                }
            });
            gen_en_passant(board, |from, to| push(Move::en_passant(from, to)));
            gen_pawn_captures(board, |from, to, capture| push(Move::capture(from, to, capture)));
            gen_promotes(board, |from, to| {
                for promote in PROMOTES {
                    push(Move::promote(from, to, promote));
                }
            });
            gen_pushes(board, |from, to, is_double| {
                if is_double {
                    push(Move::double_push(from, to));
                } else {
                    push(Move::quiet(from, to));
                }
            });

            // Other moves.
            gen_captures(board, |from, to, capture| push(Move::capture(from, to, capture)));
            gen_quiets(board, |from, to| push(Move::quiet(from, to)));
        },
        2 => {
            // King captures.
            gen_king_captures(board, |from, to, capture| push(Move::capture(from, to, capture)));
            // King quiets.
            gen_king_quiets(board, |from, to| push(Move::quiet(from, to)));
        },
        _ => unreachable!(),
    }
}