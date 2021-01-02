use std::fmt;

use crate::color::Color;
use crate::errors::ParseFenError;

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
    // List of all pieces
    pub const PIECES: [Piece; 6] = [
        Piece::Pawn, Piece::Rook, Piece::Knight, 
        Piece::Bishop, Piece::Queen, Piece::King,
    ];

    // Return true if piece is a slider
    pub(crate) fn is_slider(self) -> bool {
        match self {
            Piece::Rook | Piece::Bishop | Piece::Queen => true,
            _ => false,
        }
    }

    // Try to parse a piece from a single char
    pub(crate) fn from_char(c: char) -> Result<(Color, Piece), ParseFenError> {
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
            _ => Err(ParseFenError::new(format!("unrecognized piece literal: \"{}\"", c))),
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