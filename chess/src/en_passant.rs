use std::fmt;
use std::str::FromStr;

use anyhow::{Error, Result};

use crate::square::Square;

//#################################################################################################
//
//                                  enum EnPassantSquare
//
//#################################################################################################

/// Keeps track off the en passant target square.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EnPassantSquare {
    Some(Square),
    None,
}

// ================================ pub impl

impl EnPassantSquare {
    /// Returns true if the square is some.
    #[inline]
    pub fn is_some(self) -> bool {
        matches!(self, EnPassantSquare::Some(_))
    }

    /// Unwraps the en passant square, panics if there is none.
    #[inline]
    pub fn unwrap(self) -> Square {
        match self {
            EnPassantSquare::Some(sq) => sq,
            _ => unreachable!(),
        }
    }
}

// ================================ traits impl

impl Default for EnPassantSquare {
    /// Returns EnPassantSquare::None.
    fn default() -> EnPassantSquare {
        EnPassantSquare::None
    }
}

impl fmt::Display for EnPassantSquare {
    /// To fen en passant square notation.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EnPassantSquare::Some(sq) => write!(f, "{}", sq),
            _ => write!(f, "-"),
        }
    }
}

impl<'a> FromStr for EnPassantSquare {
    type Err = Error;

    /// From fen en passant square notation.
    fn from_str(s: &str) -> Result<EnPassantSquare, Error> {
        Ok(match s {
            "-" => EnPassantSquare::None,
            s => EnPassantSquare::Some(Square::from_str(s)?),
        })
    }
}