use crate::bitboard::BitBoard;
use crate::color::Color;
use crate::moves::Move;
use crate::history::Ply;
use crate::square::Square;

// A stack-only mini-vector used to store information about plies
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
        const BITBOARDS: [(BitBoard, BitBoard); 2] = [(
                BitBoard(0x60),
                BitBoard(0xE),
            ), (
                BitBoard(0x6000000000000000),
                BitBoard(0xE00000000000000),
        )];

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
    pub fn do_move(&mut self, mv: Move, ply: Ply) {
        // The expression of the flag
        macro_rules! flag {
            ($flag: ident, $i: expr) => {
                self.$flag[$i]
            }
        }

        // Set the flag to false
        macro_rules! modify {
            ($flag: ident, $i: expr) => {
                flag!($flag, $i) = false;
            };
        }

        // Perform a match on that square and mofifies the flags
        macro_rules! on_square {
            ($sq: expr, $catch_all: expr) => {
                match $sq {
                    Square::A1 if flag!(queen_rooks, 0) => modify!(queen_rooks, 0),
                    Square::E1 if flag!(kings,       0) => modify!(kings, 0),
                    Square::H1 if flag!(king_rooks,  0) => modify!(king_rooks, 0),
                    Square::A8 if flag!(queen_rooks, 1) => modify!(queen_rooks, 1),
                    Square::E8 if flag!(kings,       1) => modify!(kings, 1),
                    Square::H8 if flag!(king_rooks,  1) => modify!(king_rooks, 1),
                    _ => $catch_all,
                }
            }
        }

        on_square!(mv.from(), on_square!(mv.to(), return));
        self.history.push(ply);
    }

    // Undo the last move and modify the castling rights accordingly
    #[inline]
    pub fn undo_move(&mut self, mv: Move, ply: Ply) {
        // Set the flag to true
        macro_rules! modify {
            ($flag: ident, $i: expr) => {
                self.$flag[$i] = true;
            };
        }

        // Perform a match on that square and mofifies the flags
        macro_rules! on_square {
            ($sq: expr, $catch_all: expr) => {
                match $sq {
                    Square::A1 => modify!(queen_rooks, 0),
                    Square::E1 => modify!(kings, 0),
                    Square::H1 => modify!(king_rooks, 0),
                    Square::A8 => modify!(queen_rooks, 1),
                    Square::E8 => modify!(kings, 1),
                    Square::H8 => modify!(king_rooks, 1),
                    _ => $catch_all,
                }
            }
        }

        if let Some(last_ply) = self.history.last() {
            if ply == last_ply {
                on_square!(mv.from(), on_square!(mv.to(), unreachable!()));
                self.history.remove_last();
            }    
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
            history: MiniVec::default(),
        }
    }
}