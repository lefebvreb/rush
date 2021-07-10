use std::fmt;
use std::str::FromStr;

use crate::errors::ParseFenError;
use crate::square::Square;

//#################################################################################################
//
//                                  enum EnPassantSquare
//
//#################################################################################################

// Keeps track off the en passant target square.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EnPassantSquare {
    Some(Square),
    None,
}

// ================================ pub(crate) impl

impl EnPassantSquare {
    // Returns true if the square is some.
    #[inline]
    pub(crate) fn is_some(self) -> bool {
        matches!(self, EnPassantSquare::Some(_))
    }

    // Unwraps the en passant square, panics if there is none.
    #[inline]
    pub(crate) fn unwrap(self) -> Square {
        match self {
            EnPassantSquare::Some(sq) => sq,
            _ => unreachable!(),
        }
    }
}

// ================================ traits impl

impl Default for EnPassantSquare {
    // Returns EnPassantSquare::None.
    fn default() -> EnPassantSquare {
        EnPassantSquare::None
    }
}

impl fmt::Display for EnPassantSquare {
    // To fen en passant square notation.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EnPassantSquare::Some(sq) => write!(f, "{}", sq),
            _ => write!(f, "-"),
        }
    }
}

impl<'a> FromStr for EnPassantSquare {
    type Err = ParseFenError;

    // From fen en passant square notation.
    fn from_str(s: &str) -> Result<EnPassantSquare, ParseFenError> {
        Ok(match s {
            "-" => EnPassantSquare::None,
            s => EnPassantSquare::Some(Square::from_str(s)?),
        })
    }
}