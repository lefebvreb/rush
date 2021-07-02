use std::fmt;
use std::str::FromStr;

use crate::errors::ParseFenError;
use crate::square::Square;

//#################################################################################################
//
//                                       enum CastleMask
//
//#################################################################################################

// Represents the masks used to manipulate castle rights.
#[repr(u8)]
#[derive(Debug)]
pub(crate) enum CastleMask {
    WhiteOO  = 0b0001,
    WhiteOOO = 0b0010,
    BlackOO  = 0b0100,
    BlackOOO = 0b1000,
}

//#################################################################################################
//
//                                      struct CastleRights
//
//#################################################################################################

// Used to represent castle availability for both players.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CastleRights(u8);

// ================================ pub(crate) impl

impl CastleRights {
    // Returns true if those rights contain that mask.
    #[inline]
    pub(crate) fn has(self, mask: CastleMask) -> bool {
        (self.0 & mask as u8) != 0
    }

    // Updates the rights with the given from and to squares of the move.
    #[inline]
    pub(crate) fn update(&mut self, from: Square, to: Square) {
        match from {
            Square::C1 => self.remove(CastleMask::WhiteOOO),
            Square::G1 => self.remove(CastleMask::WhiteOO),
            Square::C8 => self.remove(CastleMask::BlackOOO),
            Square::G8 => self.remove(CastleMask::BlackOO),
            _ => (),
        }

        match to {
            Square::A1 => self.remove(CastleMask::WhiteOOO),
            Square::H1 => self.remove(CastleMask::WhiteOO),
            Square::A8 => self.remove(CastleMask::BlackOOO),
            Square::H8 => self.remove(CastleMask::BlackOO),
            _ => (),
        }
    }
}

// ================================ impl

impl CastleRights {
    // Remove the mask from the castling rights.
    #[inline]
    fn remove(&mut self, mask: CastleMask) {
        self.0 &= !(mask as u8)
    }
}

// ================================ traits impl

impl Default for CastleRights {
    // The default castle rights: all of them.
    fn default() -> CastleRights {
        CastleRights(0b1111)
    }
}

impl fmt::Display for CastleRights {
    // To fen notation for castle rights.
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

impl<'a> FromStr for CastleRights {
    type Err = ParseFenError;

    // From fen notation for castle rights.
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
            _ => return Err(ParseFenError::new("Invalid castle rights format")),
        }))
    }
}