use std::fmt;

use crate::color::Color;

/// Represent a piece
#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Piece {
    Pawn   = 0,
    Rook   = 1,
    Knight = 2,
    Bishop = 3,
    Queen  = 4,
    King   = 5,
}

impl Piece {
    // Try to parse a piece from a single char
    pub(crate) fn from_char(c: char) -> Result<(Color, Piece), String> {
        match c {
            'P' => Ok((Color::White, Piece::Pawn)),
            'R' => Ok((Color::White, Piece::Rook)),
            'N' => Ok((Color::White, Piece::Knight)),
            'B' => Ok((Color::White, Piece::Bishop)),
            'Q' => Ok((Color::White, Piece::Queen)),
            'K' => Ok((Color::White, Piece::King)),
            'p' => Ok((Color::Black, Piece::Pawn)),
            'r' => Ok((Color::Black, Piece::Rook)),
            'n' => Ok((Color::Black, Piece::Knight)),
            'b' => Ok((Color::Black, Piece::Bishop)),
            'q' => Ok((Color::Black, Piece::Queen)),
            'k' => Ok((Color::Black, Piece::King)),
            _ => Err(format!("Unrecognized piece literal: \"{}\"", c).to_owned()),
        }
    }
}

impl fmt::Display for Piece {
    // Display the piece
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            Piece::Pawn => 'p',
            Piece::Rook => 'r',
            Piece::Knight => 'n',
            Piece::Bishop => 'b',
            Piece::Queen => 'q',
            Piece::King => 'k',
        })
    }
}