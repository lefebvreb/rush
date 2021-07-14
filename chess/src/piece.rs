use std::fmt;

use crate::color::Color;
use crate::errors::ParseFenError;

//#################################################################################################
//
//                                        enum Piece
//
//#################################################################################################

/// Represents a piece.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Piece {
    Pawn = 0,
    Rook = 1,
    Knight = 2,
    Bishop = 3,
    Queen = 4,
    King = 5,
}

// ================================ pub impl

impl Piece {
    /// The list of all pieces, in order.
    pub const PIECES: [Piece; 6] = [
        Piece::Pawn, Piece::Rook, Piece::Knight, 
        Piece::Bishop, Piece::Queen, Piece::King,
    ];

    /// The pieces a pawn promotes to, in order from most to least interesting.
    pub const PROMOTES: [Piece; 4] = [
        Piece::Queen, Piece::Rook, Piece::Bishop, Piece::Knight,
    ];
}

// ================================ pub(crate) impl

impl Piece {
    // Returns the piece corresponding to that number, assumes 0 <= i < 6
    pub(crate) unsafe fn from_unchecked(i: u8) -> Piece {
        *Piece::PIECES.get_unchecked(i as usize)
    }
    
    // Tries to parse a piece from a single char.
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
            _ => Err(ParseFenError::new("unrecognized piece literal")),
        }
    }
}

// ================================ traits impl

impl fmt::Display for Piece {
    /// Gives the character representing the piece.
    /// Use the # modifier to print it in uppercase.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", if f.alternate() {
            match self {
                Piece::Pawn   => 'P',
                Piece::Rook   => 'R',
                Piece::Knight => 'N',
                Piece::Bishop => 'B',
                Piece::Queen  => 'Q',
                Piece::King   => 'K',
            }
        } else {
            match self {
                Piece::Pawn   => 'p',
                Piece::Rook   => 'r',
                Piece::Knight => 'n',
                Piece::Bishop => 'b',
                Piece::Queen  => 'q',
                Piece::King   => 'k',
            }
        })
    }
}

impl From<u8> for Piece {
    /// Creates a piece from a number. See codes in number definition.
    #[inline]
    fn from(i: u8) -> Piece {
        Piece::PIECES[i as usize]
    }
}

impl From<Piece> for usize {
    /// Use the piece as an index.
    #[inline]
    fn from(piece: Piece) -> usize {
        piece as usize
    }
}