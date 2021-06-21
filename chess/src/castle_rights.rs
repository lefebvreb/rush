use std::fmt;
use std::str::FromStr;

use crate::errors::ParseFenError;

//#################################################################################################
//
//                                       enum CastleMask
//
//#################################################################################################

#[repr(u8)]
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

// Used to represent castle availability:
// bit 0: White king side rights
// bit 1: White queen side rights
// bit 2: Black king side rights
// bit 3: Black queen side rights
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub(crate) struct CastleRights(u8);

// ================================ pub impl

impl CastleRights {
    #[inline]
    pub fn has(self, mask: CastleMask) -> bool {
        (self.0 & mask as u8) != 0
    }

    #[inline]
    pub fn set(mut self, mask: CastleMask) -> CastleRights {
        self.0 |= mask as u8;
        self
    }

    #[inline]
    pub fn unset(mut self, mask: CastleMask) -> CastleRights {
        self.0 &= !(mask as u8);
        self
    }
}

// ================================ pub(crate) impl

impl CastleRights {
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