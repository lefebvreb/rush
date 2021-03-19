use std::fmt;
use std::str::FromStr;

use crate::bitboard::BitBoard;
use crate::color::Color;
use crate::errors::ParseFenError;
use crate::moves::Move;
use crate::square::Square;

//#################################################################################################
//
//                                      enum CastleAvailability
//
//#################################################################################################

// Convenient struct to carry the availability of castling moves
#[repr(u8)]
#[derive(Debug)]
pub(crate) enum CastleAvailability {
    None,
    KingSide,
    QueenSide,
    Both,
}

//#################################################################################################
//
//                                      struct CastleRights
//
//#################################################################################################

// Used to represent castle availability:
// bit 0: White king side rights
// bit 1: White queen side rights
// bit 2: Black king side rights
// bit 3: Black queen side rights
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub(crate) struct CastleRights(u8);

// ================================ pub(crate) impl

impl CastleRights {
    // Return the castling abilities of a certain player, based on the monochrome
    // occupancy bitboard and the attack bitboard of the opponent
    // GIVEN THE KING IS NOT IN CHECK
    #[inline(always)]
    pub(crate) fn get_availability(self, color: Color, occ: BitBoard, danger: BitBoard) -> CastleAvailability {
        let (king_side, queen_side) = match color {
            Color::White => (
                self.0 & Self::WHITE_OO  != 0 && (
                    (occ | danger) & squares!(Square::F1, Square::G1)
                ).empty(),
                self.0 & Self::WHITE_OOO != 0 && (
                    occ & squares!(Square::B1, Square::C1, Square::D1) | 
                    danger & squares!(Square::C1, Square::D1)
                ).empty(),
            ),
            Color::Black => (
                self.0 & Self::BLACK_OO  != 0 && (
                    (occ | danger) & squares!(Square::F8, Square::G8)
                ).empty(),
                self.0 & Self::BLACK_OOO != 0 && (
                    occ & squares!(Square::B8, Square::C8, Square::D8) | 
                    danger & squares!(Square::C8, Square::D8)
                ).empty(),
            ),
        };

        match (king_side, queen_side) {
            (false, false) => CastleAvailability::None,
            (true, false)  => CastleAvailability::KingSide,
            (false, true)  => CastleAvailability::QueenSide,
            (true, true)   => CastleAvailability::Both,
        }
    }

    // Update castling rights and zobrist key
    #[inline(always)]
    pub(crate) fn update(self, color: Color, mv: Move) -> CastleRights {
        macro_rules! remove {
            ($mask: expr) => {
                CastleRights(self.0 & !$mask)
            };
        }

        match color {
            Color::White => match mv.from() {
                Square::H1 => remove!(Self::WHITE_OO),
                Square::E1 => remove!(Self::WHITE_OO | Self::WHITE_OOO),
                Square::A1 => remove!(Self::WHITE_OOO),
                _ => match mv.to() {
                    Square::H8 => remove!(Self::BLACK_OO),
                    Square::A8 => remove!(Self::BLACK_OOO),
                    _ => self,
                }
            }
            Color::Black => match mv.from() {
                Square::H8 => remove!(Self::BLACK_OO),
                Square::E8 => remove!(Self::BLACK_OO | Self::BLACK_OOO),
                Square::A8 => remove!(Self::BLACK_OOO),
                _ => match mv.to() {
                    Square::H1 => remove!(Self::WHITE_OO),
                    Square::A1 => remove!(Self::WHITE_OOO),
                    _ => self,
                }
            },
        }
    }

    // Get the raw integer corresponding to those rights
    #[inline(always)]
    pub(crate) fn get_raw(self) -> u8 {
        self.0
    }
}

// ================================ impl

impl CastleRights {
    // The masks corresponding to the different castling rights
    const WHITE_OO:  u8 = 0b0001;
    const WHITE_OOO: u8 = 0b0010;
    const BLACK_OO:  u8 = 0b0100;
    const BLACK_OOO: u8 = 0b1000;
}

// ================================ traits impl

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
            0b0110 => "Qk",
            0b0111 => "KQk",
            0b1000 => "q",
            0b1001 => "Kq",
            0b1010 => "Qq",
            0b1011 => "KQq",
            0b1100 => "kq",
            0b1101 => "Kkq",
            0b1110 => "Qkq",
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
            "Qk"   => 0b0110,
            "KQk"  => 0b0111,
            "q"    => 0b1000,
            "Kq"   => 0b1001,
            "Qq"   => 0b1010,
            "KQq"  => 0b1011,
            "kq"   => 0b1100,
            "Kkq"  => 0b1101,
            "Qkq"  => 0b1110,
            "KQkq" => 0b1111,
            _ => return Err(ParseFenError::new("Invalid castle rights format".to_owned())),
        }))
    }
}