use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

use crate::board::Board;
use crate::color::Color;
use crate::errors::ParseFenError;
use crate::game::Game;
use crate::moves::Move;
use crate::zobrist::Position;

//#################################################################################################
//
//                                      struct Clock
//
//#################################################################################################

// Represent a ply (half-turn) counter
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct Clock {
    halfmoves: u8,
    fullmoves: u32,
}

// ================================ pub(crate) impl

impl Clock {
    // Increment the counter
    #[inline(always)]
    pub(crate) fn increment(self, color: Color, mv: Move, board: &Board) -> Clock {
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

    // Return the number of reversibles halfmoves so far
    pub(crate) fn get_halfmoves(self) -> u8 {
        self.halfmoves
    }

    // Parse a Clock from two strings
    pub(crate) fn from_strs(s1: &str, s2: &str) -> Result<Clock, ParseFenError> {
        Ok(Clock {
            halfmoves: u8::from_str(s1)?,
            fullmoves: u32::from_str(s2)?,
        })
    }
}

// ================================ traits impl

impl fmt::Display for Clock {
    // Display the counter in FEN notation
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.halfmoves, self.fullmoves)
    }
}

//#################################################################################################
//
//                                  struct ThreefoldCounter
//
//#################################################################################################

/// Count and compare the positions to determine if the game is drawn based
/// off the three-fold repetitions rule
#[derive(Debug)]
pub struct ThreefoldCounter {
    map: HashMap<Position, u8>,
}

// ================================ pub(crate) impl

impl ThreefoldCounter {
    // Register a new move, given the old board and the new game
    pub(crate) fn is_draw(&mut self, mv: Move, old_board: &Board, new_game: &Game) -> bool {
        let position = Position::from(new_game);

        if mv.is_truly_reversible(old_board) {
            if let Some(counter) = self.map.get_mut(&position) {
                *counter += 1;
                return *counter == 3;
            }
        } else {
            self.map.clear();
        }

        self.map.insert(position, 1);

        return false;
    }
}

// ================================ traits impl

impl Default for ThreefoldCounter {
    fn default() -> ThreefoldCounter {
        ThreefoldCounter {
            map: HashMap::with_capacity(50),
        }
    }
}