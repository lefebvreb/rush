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
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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

// ================================ pub(crate) impl

impl Color {
    #[inline(always)]
    pub(crate) const fn idx(self) -> usize {
        self as usize
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
    // To FEN color notation
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            Color::White => "w",
            Color::Black => "b",
        })
    }
}

impl From<u8> for Color {
    #[inline(always)]
    fn from(i: u8) -> Color {
        Color::COLORS[i as usize]
    }
}

impl FromStr for Color {
    type Err = ParseFenError;

    // From FEN color notation
    fn from_str(s: &str) -> Result<Color, ParseFenError> {
        match s {
            "w" => Ok(Color::White),
            "b" => Ok(Color::Black),
            _ => Err(ParseFenError::new(format!("Invalid color litteral: {:?}", s))),
        }
    }
}
