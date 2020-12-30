use crate::bitboard::BitBoard;
use crate::color::Color;
use crate::moves::Move;
use crate::history::Ply;
use crate::square::Square;

#[derive(Clone, Default, Debug)]
struct MiniVec {
    cursor: u8,
    plies: [Ply; 6],
}

impl MiniVec {
    fn push(&mut self, ply: Ply) {
        self.plies[self.cursor as usize] = ply;
        self.cursor += 1;
    }

    fn remove_last(&mut self) {
        self.cursor -= 1;
    }

    fn last(&self) -> Option<Ply> {
        if self.cursor == 0 {
            None
        } else {
            Some(self.plies[self.cursor as usize - 1])
        }
    }
}

// Convenient struct to carry the availability of castling moves
#[repr(u8)]
#[derive(PartialEq, Debug)]
pub enum CastleAvailability {
    None,
    KingSide,
    QueenSide,
    Both,
}

// Used to remember and handle the castling privileges associated
// to each color
#[derive(Clone, Debug)]
pub struct CastleRights {
    kings: [bool; 2],
    king_rooks: [bool; 2],  
    queen_rooks: [bool; 2],  
    history: MiniVec,
}

//#################################################################################################
//
//                                       Implementation
//
//#################################################################################################

impl CastleRights {
    // Return the castling abilities of a certain player, based on the monochrome
    // occupancy bitboard and the attack bitboard of the opponent
    // GIVEN THE KING IS NOT IN CHECK
    #[inline]
    pub fn get_availability(&self, color: Color, occ: BitBoard, danger: BitBoard) -> CastleAvailability {
        const BITBOARDS: [(BitBoard, BitBoard); 2] = [
            (
                BitBoard(0x60),
                BitBoard(0xE),
            ),
            (
                BitBoard(0x6000000000000000),
                BitBoard(0xE00000000000000),
            ),
        ];

        if self.kings[color as usize] {
            let block = occ | danger;

            let king_side = self.king_rooks[color as usize] && (
                block & BITBOARDS[color as usize].0
            ).is_empty();
            let queen_side = self.queen_rooks[color as usize] && (
                block & BITBOARDS[color as usize].1
            ).is_empty();

            if king_side {
                if queen_side {
                    CastleAvailability::Both
                } else {
                    CastleAvailability::KingSide
                }
            } else {
                if queen_side {
                    CastleAvailability::QueenSide
                } else {
                    CastleAvailability::None
                }
            }
        } else {
            CastleAvailability::None
        }
    }

    // Take into account the last move and modify the castling rights accordingly
    #[inline]
    pub fn do_move(&mut self, color: Color, mv: Move, ply: Ply) {
        /*let status = match mv {
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
        }*/
        todo!()
    }

    // Undo the last move and modify the castling rights accordingly
    #[inline]
    pub fn undo_move(&mut self, color: Color, mv: Move, ply: Ply) {
        /*if self.history.last().map_or(false, |p| *p == ply) {
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
        }*/
        todo!()
    }
}

impl Default for CastleRights {
    #[cold]
    fn default() -> CastleRights {
        CastleRights {
            queen_rooks: [true; 2],
            kings: [true; 2],
            king_rooks: [true; 2],    
            history: MiniVec::default(),
        }
    }
}