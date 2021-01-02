use std::fmt;
use std::str::FromStr;

use crate::bitboard::BitBoard;
use crate::color::Color;
use crate::errors::ParseFenError;
use crate::moves::Move;
use crate::square::Square;

// Convenient struct to carry the availability of castling moves
#[repr(u8)]
#[derive(PartialEq, Debug)]
pub enum CastleAvailability {
    None,
    KingSide,
    QueenSide,
    Both,
}

// Used to represent castle availability:
// bit 0: White king side rights
// bit 1: White queen side rights
// bit 2: Black king side rights
// bit 3: Black queen side rights
#[derive(Copy, Clone, Debug)]
pub struct CastleRights(u8);

//#################################################################################################
//
//                                       Implementation
//
//#################################################################################################

impl CastleRights {
    // Empty castle rights
    pub(crate) const NONE: CastleRights = CastleRights(0);

    // Return the castling abilities of a certain player, based on the monochrome
    // occupancy bitboard and the attack bitboard of the opponent
    // GIVEN THE KING IS NOT IN CHECK
    #[inline(always)]
    pub fn get_availability(self, color: Color, occ: BitBoard, danger: BitBoard) -> CastleAvailability {
        let (king_side, queen_side) = match color {
            Color::White => (
                self.0 & 0b0001 != 0 && ((occ | danger) & BitBoard(0x60)).is_empty(),
                self.0 & 0b0010 != 0 && (occ & BitBoard(0xE) | danger & BitBoard(0xC)).is_empty()
            ),
            Color::Black => (
                self.0 & 0b0100 != 0 && ((occ | danger) & BitBoard(0x6000000000000000)).is_empty(),
                self.0 & 0b1000 != 0 && (occ & BitBoard(0xE00000000000000) | danger & BitBoard(0xC00000000000000)).is_empty(),
            ),
        };

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
    }

    // Update castling rights
    #[inline(always)]
    pub(crate) fn update(self, color: Color, mv: Move) -> CastleRights {
        macro_rules! modify {
            ($mask: literal) => {
                CastleRights(self.0 & !$mask)
            };
        }

        match color {
            Color::White => match mv.from() {
                Square::H1 => modify!(0b0001),
                Square::E1 => modify!(0b0011),
                Square::A1 => modify!(0b0010),
                _ => match mv.to() {
                    Square::H8 => modify!(0b0100),
                    Square::A8 => modify!(0b1000),
                    _ => self,
                }
            }
            Color::Black => match mv.from() {
                Square::H8 => modify!(0b0100),
                Square::E8 => modify!(0b1100),
                Square::A8 => modify!(0b1000),
                _ => match mv.to() {
                    Square::H1 => modify!(0b0001),
                    Square::A1 => modify!(0b0010),
                    _ => self,
                }
            },
        }
    }
}

impl Default for CastleRights {
    // The default castle rights: all of them
    fn default() -> CastleRights {
        CastleRights(0b1111)
    }
}

impl fmt::Display for CastleRights {
    // To FEN castle rights notation
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self.0 {
            0b0000 => "-",
            0b0001 => "K",
            0b0010 => "Q",
            0b0011 => "KQ",
            0b0100 => "k",
            0b0101 => "Kk",
            0b0111 => "KQk",
            0b1000 => "q",
            0b1001 => "Kq",
            0b1010 => "Qq",
            0b1011 => "KQq",
            0b1100 => "kq",
            0b1101 => "Kkq",
            0b1111 => "KQkq",
            _ => unreachable!(),
        })
    }
}

impl FromStr for CastleRights {
    type Err = ParseFenError;

    // From FEN castle rights notation
    fn from_str(s: &str) -> Result<CastleRights, ParseFenError> {
        Ok(CastleRights(match s {
            "-"    => 0b0000,
            "K"    => 0b0001,
            "Q"    => 0b0010,
            "KQ"   => 0b0011,
            "k"    => 0b0100,
            "Kk"   => 0b0101,
            "KQk"  => 0b0111,
            "q"    => 0b1000,
            "Kq"   => 0b1001,
            "Qq"   => 0b1010,
            "KQq"  => 0b1011,
            "kq"   => 0b1100,
            "Kkq"  => 0b1101,
            "KQkq" => 0b1111,
            _ => return Err(ParseFenError::new("Invalid castle rights format".to_owned())),
        }))
    }
}