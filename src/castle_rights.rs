use crate::squares;
use crate::bitboard::BitBoard;
use crate::color::Color;
use crate::moves::MoveType;
use crate::square::Square;

/* ======== MEMO ===========

1. The castling must be kingside or queenside.
2. Neither the king nor the chosen rook has previously moved.
3. There are no pieces between the king and the chosen rook.
4. The king is not currently in check.
5. The king does not pass through a square that is attacked by an enemy piece.
6. The king does not end up in check. (True of any legal move.)

========================= */



/* ======== NOTE ===========

Dumb rules make ugly code sry

========================= */

#[derive(Clone, PartialEq, Debug)]
enum Status {
    InitialPosition,
    Captured,
    Moved(u32),
    Castled,
}

impl Default for Status {
    fn default() -> Self {
        Status::InitialPosition
    }
}

#[derive(Clone, Debug, Default)]
struct History {
    len: u32,
    plies: [u32; 6]
}

impl History {
    #[inline]
    pub fn push(&mut self, ply: u32) {
        self.plies[self.len as usize] = ply;
        self.len += 1;
    }

    #[inline]
    pub fn is_last(&self, ply: u32) -> bool {
        if self.len == 0 {
            false
        } else {
            self.plies[(self.len - 1) as usize] == ply
        }        
    }

    #[inline]
    pub fn pop(&mut self) {
        self.len -= 1;
    }
}

#[derive(Clone, Debug, Default)]
pub struct CastleRights {
    ply: u32,
    queen_rooks: [Status; 2],
    kings: [Status; 2],
    king_rooks: [Status; 2],    
    history: History,
}

impl CastleRights {
    #[inline]
    pub fn can_kingcastle(&self, color: Color, occ: BitBoard, atk: BitBoard) -> bool {
        match color {
            Color::White => {
                self.kings[0] == Status::InitialPosition &&
                self.king_rooks[0] == Status::InitialPosition &&                
                (occ & squares!(Square::F1, Square::G1)).is_empty() &&
                (atk & squares!(Square::E1, Square::F1, Square::G1)).is_empty()
            }
            Color::Black => {
                self.kings[1] == Status::InitialPosition &&
                self.king_rooks[1] == Status::InitialPosition &&                
                (occ & squares!(Square::F8, Square::G8)).is_empty() &&
                (atk & squares!(Square::E8, Square::F8, Square::G8)).is_empty()
            }
        }
    }

    #[inline]
    pub fn can_queencastle(&self, color: Color, occ: BitBoard, atk: BitBoard) -> bool {
        match color {
            Color::White => {
                self.queen_rooks[0] == Status::InitialPosition &&
                self.kings[0] == Status::InitialPosition &&
                (occ & squares!(Square::B1, Square::C1, Square::D1)).is_empty() &&
                (atk & squares!(Square::B1, Square::C1, Square::D1, Square::E1)).is_empty()
            }
            Color::Black => {
                self.queen_rooks[1] == Status::InitialPosition &&
                self.kings[1] == Status::InitialPosition &&
                (occ & squares!(Square::B8, Square::C8, Square::D8)).is_empty() &&
                (atk & squares!(Square::B8, Square::C8, Square::D8, Square::E8)).is_empty()
            }
        }
    }

    #[inline]
    pub fn do_move(&mut self, color: Color, mv: &MoveType) {
        macro_rules! status {
            (moved $set: expr) => {
                if $set[color as usize] == Status::InitialPosition {
                    $set[color as usize] = Status::Moved(self.ply);
                    self.history.push(self.ply);
                }
            };
            (captured $set: expr) => {
                if $set[color as usize] == Status::InitialPosition {
                    $set[color as usize] = Status::Captured;
                    self.history.push(self.ply);
                }
            };
        }

        match mv {
            MoveType::Capture {from: Square::A1, ..} |
            MoveType::Capture {from: Square::A8, ..} |
            MoveType::Quiet {from: Square::A1, ..} |
            MoveType::Quiet {from: Square::A8, ..} => 
                status!(moved self.queen_rooks),
            MoveType::Capture {from: Square::E1, ..} |
            MoveType::Capture {from: Square::E8, ..} |
            MoveType::Quiet {from: Square::E1, ..} |
            MoveType::Quiet {from: Square::E8, ..} => 
                status!(moved self.kings),
            MoveType::Capture {from: Square::H1, ..} |
            MoveType::Capture {from: Square::H8, ..} |
            MoveType::Quiet {from: Square::H1, ..} |
            MoveType::Quiet {from: Square::H8, ..} => 
                status!(moved self.king_rooks),
            MoveType::PromoteCapture {to: Square::A1, ..} | 
            MoveType::PromoteCapture {to: Square::A8, ..} |
            MoveType::Capture {to: Square::A1, ..} |
            MoveType::Capture {to: Square::A8, ..} => 
                status!(captured self.queen_rooks),
            MoveType::PromoteCapture {to: Square::E1, ..} | 
            MoveType::PromoteCapture {to: Square::E8, ..} |
            MoveType::Capture {to: Square::E1, ..} |
            MoveType::Capture {to: Square::E8, ..} => 
                status!(captured self.kings),
            MoveType::PromoteCapture {to: Square::H1, ..} | 
            MoveType::PromoteCapture {to: Square::H8, ..} |
            MoveType::Capture {to: Square::H1, ..} |
            MoveType::Capture {to: Square::H8, ..} => 
                status!(captured self.king_rooks),
            MoveType::QueenCastle |
            MoveType::KingCastle =>
                self.kings[color as usize] = Status::Castled,
            _ => (),
        }

        self.ply += 1;
    }

    #[inline]
    pub fn undo_move(&mut self, color: Color, mv: &MoveType) {
        self.ply -= 1;

        if self.history.is_last(self.ply) {
            match mv {
                MoveType::PromoteCapture {to: Square::A1, ..} | 
                MoveType::PromoteCapture {to: Square::A8, ..} |
                MoveType::Capture {from: Square::A1, ..} |
                MoveType::Capture {from: Square::A8, ..} |
                MoveType::Capture {to: Square::A1, ..} |
                MoveType::Capture {to: Square::A8, ..} |
                MoveType::Quiet {from: Square::A1, ..} |
                MoveType::Quiet {from: Square::A8, ..} => 
                    self.queen_rooks[color as usize] = Status::InitialPosition,
                MoveType::PromoteCapture {to: Square::E1, ..} |
                MoveType::PromoteCapture {to: Square::E8, ..} |
                MoveType::Capture {from: Square::E1, ..} |
                MoveType::Capture {from: Square::E8, ..} |
                MoveType::Capture {to: Square::E1, ..} |
                MoveType::Capture {to: Square::E8, ..} |
                MoveType::Quiet {from: Square::E1, ..} |
                MoveType::Quiet {from: Square::E8, ..} |
                MoveType::QueenCastle |
                MoveType::KingCastle => 
                    self.kings[color as usize] = Status::InitialPosition,
                MoveType::PromoteCapture {to: Square::H1, ..} | 
                MoveType::PromoteCapture {to: Square::H8, ..} |
                MoveType::Capture {from: Square::H1, ..} |
                MoveType::Capture {from: Square::H8, ..} |
                MoveType::Capture {to: Square::H1, ..} |
                MoveType::Capture {to: Square::H8, ..} |
                MoveType::Quiet {from: Square::H1, ..} |
                MoveType::Quiet {from: Square::H8, ..} => 
                    self.king_rooks[color as usize] = Status::InitialPosition,
                _ => unreachable!(),
            }
        }
    }
}