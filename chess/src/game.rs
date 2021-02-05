use std::fmt;
use std::str::FromStr;

use crate::board::Board;
use crate::castle_rights::CastleRights;
use crate::color::Color;
use crate::en_passant::EnPassantSquare;
use crate::errors::ParseFenError;
use crate::clock::Clock;
use crate::moves::Move;
use crate::piece::Piece;
use crate::square::Square;

/// A struct that holds every information defining a complete game of chess
#[derive(Debug)]
pub struct Game {
    board: Board,
    color: Color,
    castle_rights: CastleRights,
    ep_rights: EnPassantSquare,
    clock: Clock,
}

impl Game {
    /// Perform a new move and modifiy the game accordingly
    #[inline]
    pub fn do_move(&self, mv: Move) -> Game {
        let mut board = self.board.clone();
        board.do_move(self.color, mv);
        let color = self.color.invert();
        let en_passant = EnPassantSquare::get(mv);
        let castle_rights = self.castle_rights.update(self.color, mv);
        let clock = self.clock.increment(self.color, mv, &self.board);

        Game {
            board,
            color,
            castle_rights,
            ep_rights: en_passant,
            clock,
        }
    }

    /// Return the game's board
    #[inline(always)]
    pub fn get_board(&self) -> &Board {
        &self.board
    }

    /// Return the color of the current player
    #[inline(always)]
    pub fn get_color(&self) -> Color {
        self.color
    }

    // Return the castling rights
    #[inline(always)]
    pub(crate) fn get_castle_rights(&self) -> CastleRights {
        self.castle_rights
    }

    // Return the last move played
    #[inline(always)]
    pub(crate) fn get_ep_rights(&self) -> EnPassantSquare {
        self.ep_rights
    }

    /// Try to parse a move from current position with given coordinates,
    /// in pure algebraic notation, of course.
    /// Does not verify the validity of the move.
    pub fn parse_move(&self, s: &str) -> Result<Move, ParseFenError> {
        let from = Square::from_str(&s[0..2])?;
        let to = Square::from_str(&s[2..4])?;

        match s.len() {
            4 => {
                match self.board.get_piece_unchecked(from) {
                    Piece::Pawn => if from.x() != to.x() && self.board.is_empty(to) {
                        return Ok(Move::EnPassant {
                            from,
                            to,
                        })
                    } else {
                        let diff = from.y() as i8 - to.y() as i8;
                        if diff == 2 || diff == -2 {
                            return Ok(Move::DoublePush {
                                from,
                                to,
                            })
                        }
                    }
                    Piece::King => match (from, to) {
                        (Square::E1, Square::G1) | 
                        (Square::E8, Square::G8) => return Ok(Move::KingCastle {color: self.color}),
                        (Square::E1, Square::C1) | 
                        (Square::E8, Square::C8) => return Ok(Move::QueenCastle {color: self.color}),
                        _ => (),
                    }
                    _ => (),
                }

                if let Some((_, capture)) = self.board.get_piece(to) {
                    Ok(Move::Capture {
                        from,
                        to,
                        capture,
                    })
                } else {
                    Ok(Move::Quiet {
                        from, 
                        to,
                    })
                }
            }
            5 => {
                let promote = match s.chars().nth(4).unwrap() {
                    'r' => Piece::Rook,
                    'n' => Piece::Knight,
                    'b' => Piece::Bishop,
                    'q' => Piece::Queen,
                    c => return Err(ParseFenError::new(format!("unrecognized promotion: {:?}, valid promotions are: \"rnbq\"", c))),
                };
    
                if let Some((_, capture)) = self.board.get_piece(to) {
                    Ok(Move::PromoteCapture {
                        from, 
                        to, 
                        capture, 
                        promote,
                    })
                } else {
                    Ok(Move::Promote {
                        from, 
                        to, 
                        promote,
                    })
                }
            }
            _ => Err(ParseFenError::new("a move should be encoded in pure algebraic coordinate notation")),
        }
    }

    /// Try to parse a position from fen notation.
    pub fn from_fen(fen: &str) -> Result<Game, ParseFenError> {
        Game::from_str(fen)
    }
}

impl Default for Game {
    // Return a new full game, with the starting position of chess
    fn default() -> Game {
        Game {
            board: Board::default(),
            castle_rights: CastleRights::default(),
            color: Color::default(),
            ep_rights: EnPassantSquare::None,
            clock: Clock::default(),
        }
    }
}

impl fmt::Display for Game {
    // Give the FEN representation of the board
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {}",
            self.board,
            self.color,
            self.castle_rights,
            self.ep_rights,
            self.clock,
        )
    }
}

impl FromStr for Game {
    type Err = ParseFenError;

    // Try to construct a board from a given FEN notation
    fn from_str(s: &str) -> Result<Game, ParseFenError> {
        let strings = s.split(" ").into_iter().collect::<Vec<_>>();
        
        if strings.len() != 6 {
            return Err(ParseFenError::new("missing informations on FEN notation"));
        }

        Ok(Game {
            board: Board::from_str(strings[0])?,
            color: Color::from_str(strings[1])?,
            castle_rights: CastleRights::from_str(strings[2])?,
            ep_rights: EnPassantSquare::from_str(strings[3])?,
            clock: Clock::from_strs(strings[4], strings[5])?,
        })
    }
}