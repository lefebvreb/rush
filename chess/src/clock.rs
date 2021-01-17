use std::fmt;
use std::str::FromStr;

use crate::board::Board;
use crate::color::Color;
use crate::errors::ParseFenError;
use crate::moves::Move;

// Represent a ply (half-turn) counter
#[derive(Copy, Clone, PartialEq, Default, Debug)]
pub struct Clock {
    halfmoves: u8,
    fullmoves: u32,
}

impl Clock {
    // Increment the counter
    #[inline(always)]
    pub fn increment(self, color: Color, mv: Move, board: &Board) -> Clock {
        Clock {
            halfmoves: if mv.is_reversible(board) {
                self.halfmoves + 1
            } else {
                0
            },
            fullmoves: match color {
                Color::White => self.fullmoves,
                Color::Black => self.fullmoves + 1,
            },
        }
    }

    // Parse a Clock from two strings
    pub fn from_strs(s1: &str, s2: &str) -> Result<Clock, ParseFenError> {
        Ok(Clock {
            halfmoves: u8::from_str(s1)?,
            fullmoves: u32::from_str(s2)?,
        })
    }
}

impl fmt::Display for Clock {
    // Display the counter in FEN notation
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.halfmoves, self.fullmoves)
    }
}