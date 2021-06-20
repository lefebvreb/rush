use std::fmt;
use std::str::FromStr;

use crate::errors::ParseFenError;
use crate::square::Square;

//#################################################################################################
//
//                                  enum EnPassantSquare
//
//#################################################################################################

// Keep track off the en passant target square
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EnPassantSquare {
    Some(Square),
    None,
}

// ================================ traits impl

impl Default for EnPassantSquare {
    fn default() -> EnPassantSquare {
        EnPassantSquare::None
    }
}

impl fmt::Display for EnPassantSquare {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EnPassantSquare::Some(sq) => write!(f, "{}", sq),
            _ => write!(f, "-"),
        }
    }
}

impl FromStr for EnPassantSquare {
    type Err = ParseFenError;

    fn from_str(s: &str) -> Result<EnPassantSquare, ParseFenError> {
        Ok(match s {
            "-" => EnPassantSquare::None,
            s => EnPassantSquare::Some(Square::from_str(s)?),
        })
    }
}