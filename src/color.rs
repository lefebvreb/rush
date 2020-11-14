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
    #[inline]
    pub const fn invert(self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

impl Default for Color {
    #[cold]
    fn default() -> Self {
        Color::White
    }
}
