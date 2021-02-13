use std::fmt;
use std::str::FromStr;

use crate::errors::ParseFenError;

//#################################################################################################
//
//                                       struct Color
//
//#################################################################################################

/// Represent the color of a player
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Color {
    White = 0,
    Black = 1,
}

// ================================ pub impl

impl Color {
    /// List of colors, ordered by their values
    pub const COLORS: [Color; 2] = [
        Color::White, Color::Black,
    ];

    /// Give the opposite color of `self`
    #[inline(always)]
    pub const fn invert(self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

// ================================ traits impl

impl Default for Color {
    /// Return Color::White
    fn default() -> Self {
        Color::White
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            Color::White => "w",
            Color::Black => "b",
        })
    }
}

impl FromStr for Color {
    type Err = ParseFenError;

    fn from_str(s: &str) -> Result<Color, ParseFenError> {
        match s {
            "w" => Ok(Color::White),
            "b" => Ok(Color::Black),
            _ => Err(ParseFenError::new(format!("Invalid color litteral: {:?}", s))),
        }
    }
}
