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

/// Gives all pseudo-legals promote captures for pawns.
/// The provided closure takes three arguments: from square, to square
/// and captured piece.
/// It is called for each pseudo-legal promote-capture.
#[inline]
pub fn gen_promote_captures(board: &Board, promotes: &[Piece], mut gen: impl FnMut(Piece, Move)) {
    let us = board.get_side_to_move();
    let them = board.get_other_side();

    for from in (board.get_bitboard(us, Piece::Pawn) & BitBoard::promote_rank(us)).iter_squares() {
        for to in (attacks::pawn(us, from) & board.get_occupancy().colored(them)).iter_squares() {
            for &promote in promotes {
                let mv = Move::promote_capture(from, to, board.get_piece_unchecked(to), promote);
                gen(Piece::Pawn, mv);
            }
        }
    }
}

/// Gives all pseudo-legals promotion for pawns.
/// The provided closure takes two arguments: from square and to square.
/// It is called for each pseudo-legal promotion.
#[inline]
pub fn gen_promotes(board: &Board, promotes: &[Piece], mut gen: impl FnMut(Piece, Move)) {
    let us = board.get_side_to_move();

    for from in (board.get_bitboard(us, Piece::Pawn) & BitBoard::promote_rank(us)).iter_squares() {
        let to = attacks::pawn_push(us, from).unwrap();
        if board.get_piece(to).is_none() {
            for &promote in promotes {
                let mv = Move::promote(from, to, promote);
                gen(Piece::Pawn, mv);
            }
        }
    }
}

/// Gives all pseudo-legals en passant moves.
/// The provided closure takes two arguments: from square and to square.
/// It is called for each pseudo-legal en passant.
#[inline]
pub fn gen_en_passant(board: &Board, mut gen: impl FnMut(Piece, Move)) {
    let us = board.get_side_to_move();
    let them = board.get_other_side();

    if let EnPassantSquare::Some(sq) = board.get_ep_square() {
        let to = attacks::pawn_push(us, sq).unwrap();
        for from in (attacks::pawn(them, to) & board.get_bitboard(us, Piece::Pawn)).iter_squares() {
            let mv = Move::en_passant(from, to);
            gen(Piece::Pawn, mv);
        }
    }
}

/// Gives all pseudo-legals pawn captures.
/// The provided closure takes three arguments: from square, to square
/// and the captured piece.
/// It is called for each pseudo-legal pawn captures.
#[inline]
pub fn gen_pawn_captures(board: &Board, mut gen: impl FnMut(Piece, Move)) {
    let us = board.get_side_to_move();
    let them = board.get_other_side();

    let them_occ = board.get_occupancy().colored(them);
    
    for from in (board.get_bitboard(us, Piece::Pawn) & !BitBoard::promote_rank(us)).iter_squares() {
        for to in (attacks::pawn(us, from) & them_occ).iter_squares() {
            let mv = Move::capture(from, to, board.get_piece_unchecked(to));
            gen(Piece::Pawn, mv);
        }
    }
}

/// Gives all pseudo-legals pushes and double pushes moves.
/// The provided closure takes three arguments: from square, to square
/// and a bool which is true if the move is a double push.
/// It is called for each pseudo-legal pushes and double pushes.
#[inline]
pub fn gen_pushes(board: &Board, mut gen: impl FnMut(Piece, Move)) {
    let us = board.get_side_to_move();

    for from in (board.get_bitboard(us, Piece::Pawn) & !BitBoard::promote_rank(us)).iter_squares() {
        if let Some(to1) = attacks::pawn_push(us, from) {
            if board.get_piece(to1).is_none() {
                let mv = Move::quiet(from, to1);
                gen(Piece::Pawn, mv);
                if let Some(to2) = attacks::pawn_double_push(us, from) {
                    if board.get_piece(to2).is_none() {
                        let mv = Move::double_push(from, to2);
                        gen(Piece::Pawn, mv);
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
pub fn gen_king_captures(board: &Board, mut gen: impl FnMut(Piece, Move)) {
    let them_occ = board.get_occupancy().colored(board.get_other_side());

    let from = board.king_sq();
    for to in (attacks::king(from) & them_occ).iter_squares() {
        let mv = Move::capture(from, to, board.get_piece_unchecked(to));
        gen(Piece::King, mv);
    }
}

/// Gives all pseudo-legals quiets from the king.
/// The provided closure takes two arguments: from square and to square.
/// It is called for each pseudo-legal quiets from the king.
#[inline]
pub fn gen_king_quiets(board: &Board, mut gen: impl FnMut(Piece, Move)) {
    let free = board.get_occupancy().free();

    let from = board.king_sq();
    for to in (attacks::king(from) & free).iter_squares() {
        let mv = Move::quiet(from, to);
        gen(Piece::King, mv);
    }
}

/// Generates all pseudo-legals castling moves for the position.
/// The provided closure takes two arguments: from square and to square.
/// It is called for each pseudo-legal castling.
#[inline]
pub fn gen_castles(board: &Board, mut gen: impl FnMut(Piece, Move)) {
    let us = board.get_side_to_move();
    let castle_rights = board.get_castle_rights();

    match us {
        Color::White => {
            if castle_rights.has(CastleMask::WhiteOO) & board.is_path_clear(Square::E1, Square::H1) {
                let mv = Move::castle(Square::E1, Square::G1);
                gen(Piece::King, mv);
            }
            if castle_rights.has(CastleMask::WhiteOOO) & board.is_path_clear(Square::E1, Square::A1) {
                let mv = Move::castle(Square::E1, Square::C1);
                gen(Piece::King, mv);
            }
        },
        Color::Black => {
            if castle_rights.has(CastleMask::BlackOO) & board.is_path_clear(Square::E8, Square::H8) {
                let mv = Move::castle(Square::E8, Square::G8);
                gen(Piece::King, mv);
            }
            if castle_rights.has(CastleMask::BlackOOO) & board.is_path_clear(Square::E8, Square::A8) {
                let mv = Move::castle(Square::E8, Square::C8);
                gen(Piece::King, mv);
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

/// Gives all pseudo-legals quiets from knights, bishops, rooks and queens.
/// The provided closure takes two arguments: from square and to square.
/// It is called for each pseudo-legal quiets.
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

/// Generates all legal moves for the current position.
/// This function is rather slow. Use the other movegen functions
/// for more control over generation and better performance.
pub fn legals(board: &Board, buffer: &mut Vec<Move>) {
    // Generates all non-king moves with the given consumer.
    pub fn gen_non_king(board: &Board, mut gen: impl FnMut(Move)) {
        gen_promote_captures(board, &Piece::PROMOTES, |_, mv| gen(mv));
        gen_en_passant(board, |_, mv| gen(mv));
        gen_pawn_captures(board, |_, mv| gen(mv));
        gen_promotes(board, &Piece::PROMOTES, |_, mv| gen(mv));
        gen_pushes(board, |_, mv| gen(mv));
        gen_captures(board, |_, mv| gen(mv));
        gen_quiets(board, |_, mv| gen(mv));
    }

    let checkers = board.get_checkers();

    let mut gen = |mv| if board.is_legal(mv) {buffer.push(mv)};

    if checkers.empty() {
        // No checkers.

        // Generate all castling and king moves. 
        gen_castles(board, |_, mv| gen(mv));
        gen_king_captures(board, |_, mv| gen(mv));
        gen_king_quiets(board, |_, mv| gen(mv));

        // Generates all other moves.
        gen_non_king(board, gen);
    } else if checkers.is_one() {
        // One checker.

        // Generate all king moves.
        gen_king_captures(board, |_, mv| gen(mv));
        gen_king_quiets(board, |_, mv| gen(mv));

        // Check that the move is either capturing the checker or blocking it.
        let checker = unsafe {checkers.as_square_unchecked()};
        let mask = BitBoard::between(board.king_sq(), checker) | checkers;
        let gen = |mv: Move| if mask.contains(mv.to()) && board.is_legal(mv) {buffer.push(mv)};

        // Generate.
        gen_non_king(board, gen);
    } else {
        // Two checkers.

        // Only generate king moves.
        gen_king_captures(board, |_, mv| gen(mv));
        gen_king_quiets(board, |_, mv| gen(mv));
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

        buffer.truncate(start_index);
        total
    }

    // The internal function will panic if depth is 0.
    if depth == 0 {
        1
    } else {
        internal_perft(board, &mut Vec::new(), depth)
    }
}