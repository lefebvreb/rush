use std::fmt;
use std::str::FromStr;

/// Represent the color of a player
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Color {
    White = 0,
    Black = 1,
}

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
    type Err = String;

    fn from_str(s: &str) -> Result<Color, String> {
        match s {
            "w" => Ok(Color::White),
            "b" => Ok(Color::Black),
            _ => Err(format!("Invalid color litteral: {:?}", s)),
        }
    }
}
