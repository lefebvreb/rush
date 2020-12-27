use crate::squares;
use crate::bitboard::BitBoard;
use crate::color::Color;
use crate::moves::Move;
use crate::ply::Ply;
use crate::square::Square;

/* ======== MEMO ===========

1. The castling must be kingside or queenside.
2. Neither the king nor the chosen rook has previously moved.
3. There are no pieces between the king and the chosen rook.
4. The king is not currently in check.
5. The king does not pass through a square that is attacked by an enemy piece.
6. The king does not end up in check. (True of any legal move.)

========================= */

// Dumb rules make ugly code sry

/// Convenient struct to carry the availability of castling moves
#[repr(u8)]
#[derive(PartialEq, Debug)]
pub enum CastleAvailability {
    None,
    KingSide,
    QueenSide,
    Both,
}

/// Used to remember and handle the castling privileges associated
/// to each color
#[derive(Clone, Debug)]
pub struct CastleRights {
    kings: [bool; 2],
    king_rooks: [bool; 2],  
    queen_rooks: [bool; 2],  
    history: Vec<Ply>,
}

//#################################################################################################
//
//                                    Implementation
//
//#################################################################################################

impl CastleRights {
    /// Return the castling abilities of a certain player, based on the monochrome
    /// occupancy bitboard and the attack bitboard of the opponent.
    #[inline]
    pub fn get_availability(&self, color: Color, occ: BitBoard, danger: BitBoard) -> CastleAvailability {
        match match color {
            Color::White if self.kings[0] => {(
                    self.king_rooks[0] &&                
                    (occ & squares!(Square::F1, Square::G1)).is_empty() &&
                    (danger & squares!(Square::E1, Square::F1, Square::G1)).is_empty(),
                    self.queen_rooks[0] &&
                    (occ & squares!(Square::B1, Square::C1, Square::D1)).is_empty() &&
                    (danger & squares!(Square::B1, Square::C1, Square::D1, Square::E1)).is_empty(),
            )}
            Color::Black if self.kings[1] => {(
                    self.king_rooks[1] &&                
                    (occ & squares!(Square::F8, Square::G8)).is_empty() &&
                    (danger & squares!(Square::E8, Square::F8, Square::G8)).is_empty(),
                    self.queen_rooks[1] &&
                    (occ & squares!(Square::B8, Square::C8, Square::D8)).is_empty() &&
                    (danger & squares!(Square::B8, Square::C8, Square::D8, Square::E8)).is_empty(),
            )}
            _ => return CastleAvailability::None,
        } {
            (false, false) => CastleAvailability::None,
            (true, false) => CastleAvailability::KingSide,
            (false, true) => CastleAvailability::QueenSide,
            (true, true) => CastleAvailability::Both,
        }
    }

    /// Take into account the last move and modify the castling rights accordingly
    #[inline]
    pub fn do_move(&mut self, color: Color, mv: Move, ply: Ply) {
        let status = match mv {
            Move::PromoteCapture {to: Square::A1, ..} | 
            Move::PromoteCapture {to: Square::A8, ..} |
            Move::Capture {from: Square::A1, ..} |
            Move::Capture {from: Square::A8, ..} |
            Move::Capture {to: Square::A1, ..} |
            Move::Capture {to: Square::A8, ..} |
            Move::Quiet {from: Square::A1, ..} |
            Move::Quiet {from: Square::A8, ..} => 
                &mut self.queen_rooks,
            Move::PromoteCapture {to: Square::E1, ..} |
            Move::PromoteCapture {to: Square::E8, ..} |
            Move::Capture {from: Square::E1, ..} |
            Move::Capture {from: Square::E8, ..} |
            Move::Capture {to: Square::E1, ..} |
            Move::Capture {to: Square::E8, ..} |
            Move::Quiet {from: Square::E1, ..} |
            Move::Quiet {from: Square::E8, ..} |       
            Move::QueenCastle |
            Move::KingCastle =>
                &mut self.kings,
            Move::PromoteCapture {to: Square::H1, ..} | 
            Move::PromoteCapture {to: Square::H8, ..} |
            Move::Capture {from: Square::H1, ..} |
            Move::Capture {from: Square::H8, ..} |
            Move::Capture {to: Square::H1, ..} |
            Move::Capture {to: Square::H8, ..} |
            Move::Quiet {from: Square::H1, ..} |
            Move::Quiet {from: Square::H8, ..} => 
                &mut self.king_rooks,            
            _ => return,
        };

        if (*status)[color as usize] {
            (*status)[color as usize] = false;
            self.history.push(ply);
        }
    }

    /// Undo the last move and modify the castling rights accordingly
    #[inline]
    pub fn undo_move(&mut self, color: Color, mv: Move, ply: Ply) {
        if self.history.last().map_or(false, |p| *p == ply) {
            *match mv {
                Move::PromoteCapture {to: Square::A1, ..} | 
                Move::PromoteCapture {to: Square::A8, ..} |
                Move::Capture {from: Square::A1, ..} |
                Move::Capture {from: Square::A8, ..} |
                Move::Capture {to: Square::A1, ..} |
                Move::Capture {to: Square::A8, ..} |
                Move::Quiet {from: Square::A1, ..} |
                Move::Quiet {from: Square::A8, ..} => 
                    &mut self.queen_rooks[color as usize],
                Move::PromoteCapture {to: Square::E1, ..} |
                Move::PromoteCapture {to: Square::E8, ..} |
                Move::Capture {from: Square::E1, ..} |
                Move::Capture {from: Square::E8, ..} |
                Move::Capture {to: Square::E1, ..} |
                Move::Capture {to: Square::E8, ..} |
                Move::Quiet {from: Square::E1, ..} |
                Move::Quiet {from: Square::E8, ..} |
                Move::QueenCastle |
                Move::KingCastle => 
                    &mut self.kings[color as usize],
                Move::PromoteCapture {to: Square::H1, ..} | 
                Move::PromoteCapture {to: Square::H8, ..} |
                Move::Capture {from: Square::H1, ..} |
                Move::Capture {from: Square::H8, ..} |
                Move::Capture {to: Square::H1, ..} |
                Move::Capture {to: Square::H8, ..} |
                Move::Quiet {from: Square::H1, ..} |
                Move::Quiet {from: Square::H8, ..} => 
                    &mut self.king_rooks[color as usize],
                _ => unreachable!()
            } = true;
        }
    }
}

impl Default for CastleRights {
    #[cold]
    fn default() -> CastleRights {
        CastleRights {
            queen_rooks: [true; 2],
            kings: [true; 2],
            king_rooks: [true; 2],    
            history: Vec::with_capacity(6),
        }
    }
}