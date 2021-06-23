use std::fmt;
use std::str::FromStr;

use crate::errors::ParseFenError;

//#################################################################################################
//
//                                       enum CastleMask
//
//#################################################################################################

// Represent the masks used to manipulate castle rights.
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

// Used to represent castle availability for both players
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CastleRights(u8);

// ================================ pub impl

impl CastleRights {
    /// Return true if those rights contain that mask
    #[inline(always)]
    pub fn has(self, mask: CastleMask) -> bool {
        (self.0 & mask as u8) != 0
    }

    /// Add that mask to the rights and return the new rights
    #[inline(always)]
    pub fn add(self, mask: CastleMask) -> CastleRights {
        CastleRights(self.0 | mask as u8)
    }

    /// Remove that mask from the rights and return the new rights
    #[inline(always)]
    pub fn rem(self, mask: CastleMask) -> CastleRights {
        CastleRights(self.0 & !(mask as u8))
    }
}

// ================================ pub(crate) impl

impl CastleRights {
    // Return the raw part of the rights, as an 8 bit integer
    #[inline(always)]
    pub(crate) fn get_raw(self) -> u8 {
        self.0
    }
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