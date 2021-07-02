use std::fmt;
use std::str::FromStr;

use crate::errors::ParseFenError;

//#################################################################################################
//
//                                       struct Color
//
//#################################################################################################

/// Represents the color of a player.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Color {
    White = 0,
    Black = 1,
}

// ================================ pub impl

impl Color {
    /// List of colors, ordered by their values.
    pub const COLORS: [Color; 2] = [
        Color::White, Color::Black,
    ];

    /// Gives the opposite color of self.
    #[inline]
    pub const fn invert(self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }

    /// Returns the color used as an index.
    #[inline]
    pub fn idx(self) -> usize {
        self as usize
    }
}

// ================================ traits impl

impl Default for Color {
    /// Returns Color::White.
    fn default() -> Self {
        Color::White
    }
}

impl fmt::Display for Color {
    // To fen color notation.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            Color::White => "w",
            Color::Black => "b",
        })
    }
}

impl<'a> FromStr for Color {
    type Err = ParseFenError;

    // From fen color notation.
    fn from_str(s: &str) -> Result<Color, ParseFenError> {
        match s {
            "w" => Ok(Color::White),
            "b" => Ok(Color::Black),
            _ => Err(ParseFenError::new("invalid color litteral")),
        }
    }
}