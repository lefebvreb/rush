use std::fmt;

use crate::color::Color;
use crate::errors::ParseFenError;

//#################################################################################################
//
//                                        enum Piece
//
//#################################################################################################

/// Represent a piece
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Piece {
    Pawn   = 0,
    Rook   = 1,
    Knight = 2,
    Bishop = 3,
    Queen  = 4,
    King   = 5,
}

// ================================ pub impl

impl Piece {
    // List of all pieces
    pub const PIECES: [Piece; 6] = [
        Piece::Pawn, Piece::Rook, Piece::Knight, 
        Piece::Bishop, Piece::Queen, Piece::King,
    ];
}

// ================================ pub(crate) impl

impl Piece {
    // Return true if self is a slider
    #[inline]
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

// ================================ traits impl

impl fmt::Display for Piece {
    // Display the piece
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            Piece::Pawn   => 'p',
            Piece::Rook   => 'r',
            Piece::Knight => 'n',
            Piece::Bishop => 'b',
            Piece::Queen  => 'q',
            Piece::King   => 'k',
        })
    }
}

impl From<u8> for Piece {
    #[inline(always)]
    fn from(i: u8) -> Piece {
        Piece::PIECES[i as usize]
    }
}