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

pub type MoveList = List<Move, 256>;

// ================================ pub impl

impl Board {
    /// Generates all pseudo-legal captures and queen promotion.
    /// Combined with gen_quiets(), this function gives all pseudo-legal moves
    /// of a not-check position.
    /// Precondition: there are no checkers.
    pub fn gen_captures(&self, list: &mut MoveList) {
        // Castling.
        self.make_castles(list);

        // Queen promotes.
        self.make_promotes(|from, to, maybe_capture| {
            if let Some(capture) = maybe_capture {
                list.push(Move::promote_capture(from, to, capture, Piece::Queen));
            } else {
                list.push(Move::promote(from, to, Piece::Queen));
            }
        });

        // En passant.
        self.make_ep(|from, to| list.push(Move::en_passant(from, to)));

        // Captures.
        self.make_captures(|from, to, capture| list.push(Move::capture(from, to, capture)));
    }

    /// Generates all pseudo-legal non-captures moves and underpiece promotions.
    /// Combined with gen_captures(), this function gives all pseudo-legal moves
    /// of a not-check position.
    /// Precondition: there are no checkers.
    pub fn gen_quiets(&self, list: &mut MoveList) {
        // Push, double push
        // knight, bishops, rooks, queen, king quiets
        // rook promotion, bishop promotion, knight promotion.
        todo!()
    }

    /// Generates all pseudo-legal non-king moves of a position
    /// if there is one checker.
    /// Combined with gen_evasions(), this function gives all pseudo-legal moves
    /// of a one checker position.
    /// Precondition: there is one checker, and exactly one.
    pub fn gen_blocks(&self, list: &mut MoveList) {
        // pawns en passant, push, double push (blocking)
        // knights, bishops, rooks, queen (blocking)
        todo!()
    }

    /// Generates all pseudo-legal king moves of a position with
    /// potentially multiple checkers.
    /// Combined with gen_blocks(), this function gives all pseudo-legal moves
    /// of a one checker position.
    /// Alone, this functions gives all moves of a two-checkers position.
    /// Precondition: there is one checker, and exactly one.
    pub fn gen_evasions(&self, list: &mut MoveList) {
        // king captures
        // king quiets
        todo!()
    }
}

// ================================ impl

impl Board {
    // Generates all pseudo-legals castling moves for the position.
    // Assumes the king is not in check, there remaint to check that
    // the squares the king traverses are safe.
    #[inline(always)]
    pub fn make_castles(&self, list: &mut MoveList) {
        let us = self.get_side_to_move();
        let castle_rights = self.get_castle_rights();

        match us {
            Color::White => {
                if castle_rights.has(CastleMask::WhiteOO) & self.is_path_clear(Square::E1, Square::H1) {
                    list.push(Move::castle(Square::E1, Square::G1));
                }
                if castle_rights.has(CastleMask::WhiteOOO) & self.is_path_clear(Square::E1, Square::A1) {
                    list.push(Move::castle(Square::E1, Square::C1));
                }
            },
            Color::Black => {
                if castle_rights.has(CastleMask::BlackOO) & self.is_path_clear(Square::E8, Square::H8) {
                    list.push(Move::castle(Square::E8, Square::G8));
                }
                if castle_rights.has(CastleMask::BlackOOO) & self.is_path_clear(Square::E8, Square::A8) {
                    list.push(Move::castle(Square::E8, Square::C8));
                }
            },
        }
    }

    // Generates all pseudo-legals promotions.
    #[inline(always)]
    pub fn make_promotes<F>(&self, mut gen: F) where 
        F: FnMut(Square, Square, Option<Piece>),
    {
        let us = self.get_side_to_move();
        let them = self.get_other_side();

        let may_promote_rank = match us {
            Color::White => BitBoard::RANK_7,
            Color::Black => BitBoard::RANK_2,
        };

        for from in (self.get_bitboard(us, Piece::Pawn) & may_promote_rank).iter_squares() {
            if let Some(to) = attacks::pawn_push(us, from) {
                gen(from, to, None);
            }
            for to in (attacks::pawn(us, from) & self.get_occupancy().colored(them)).iter_squares() {
                gen(from, to, Some(self.piece_unchecked(to)));
            }
        }
    }

    // Make all en passants.
    #[inline(always)]
    fn make_ep<F>(&self, mut gen: F) where
        F: FnMut(Square, Square)
    {
        let us = self.get_side_to_move();
        let them = self.get_other_side();

        if let EnPassantSquare::Some(sq) = self.get_ep_square() {
            let to = attacks::pawn_push(us, sq).unwrap();
            for from in (attacks::pawn(them, to) & self.get_bitboard(us, Piece::Pawn)).iter_squares() {
                gen(from, to);
            }
        }
    }

    // Make all captures.
    #[inline(always)]
    fn make_captures<F>(&self, mut gen: F) where
        F: FnMut(Square, Square, Piece)
    {
        let us = self.get_side_to_move();
        let them = self.get_other_side();

        let occ = self.get_occupancy();
        let them_occ = occ.colored(them);

        for from in self.get_bitboard(us, Piece::Pawn).iter_squares() {
            for to in (attacks::pawn(us, from) & them_occ).iter_squares() {
                gen(from, to, self.piece_unchecked(to));
            }
        }
        for from in self.get_bitboard(us, Piece::Knight).iter_squares() {
            for to in (attacks::knight(from) & them_occ).iter_squares() {
                gen(from, to, self.piece_unchecked(to));
            }
        }
        for from in self.get_bitboard(us, Piece::Bishop).iter_squares() {
            for to in (attacks::bishop(from, occ) & them_occ).iter_squares() {
                gen(from, to, self.piece_unchecked(to));
            }
        }
        for from in self.get_bitboard(us, Piece::Rook).iter_squares() {
            for to in (attacks::rook(from, occ) & them_occ).iter_squares() {
                gen(from, to, self.piece_unchecked(to));
            }
        }
        for from in self.get_bitboard(us, Piece::Queen).iter_squares() {
            for to in (attacks::queen(from, occ) & them_occ).iter_squares() {
                gen(from, to, self.piece_unchecked(to));
            }
        }
        for from in self.get_bitboard(us, Piece::King).iter_squares() {
            for to in (attacks::king(from) & them_occ).iter_squares() {
                gen(from, to, self.piece_unchecked(to));
            }
        }
    }

    // Make all pawn pushes.
    #[inline(always)]
    fn make_pushes<F>(&self, mut gen: F) where
        F: FnMut(Square, Square)
    {
        todo!()
    }
}