use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

use crate::board::Board;
use crate::castle_rights::CastleRights;
use crate::color::Color;
use crate::en_passant::EnPassantSquare;
use crate::errors::ParseFenError;
use crate::clock::{Clock, ThreefoldCounter};
use crate::move_gen::MoveGenerator;
use crate::moves::Move;
use crate::piece::Piece;
use crate::square::Square;
use crate::zobrist::{Position, ZOBRIST_KEYS};

//#################################################################################################
//
//                                   enum GameStatus
//
//#################################################################################################

/// An enum representing the status of a game
pub enum GameStatus {
    Playing {
        playing: Color,
    },
    Drawn,
    Won {
        winner: Color,
    },
}

// ================================ traits impl

impl Default for GameStatus {
    fn default() -> GameStatus {
        GameStatus::Playing {playing: Color::White}
    }
}

//#################################################################################################
//
//                                        struct Game
//
//#################################################################################################

/// A struct that holds every information defining a complete game of chess
#[derive(Debug)]
pub struct Game {
    board: Board,
    color: Color,
    castle_rights: CastleRights,
    ep_rights: EnPassantSquare,
    clock: Clock,
    zobrist: u64,
}

// ================================ pub impl

impl Game {
    /// Perform a new move and modifiy the game accordingly
    #[inline]
    pub fn do_move(&self, mv: Move) -> Game {
        let mut board = self.board.clone();
        board.do_move(self.color, mv);
        let mut zobrist = board.get_zobrist();

        let color = self.color.invert();
        zobrist ^= ZOBRIST_KEYS.get_color(color);
        
        let ep_rights = self.ep_rights.update(mv, &mut zobrist);
        let castle_rights = self.castle_rights.update(self.color, mv, &mut zobrist);

        let clock = self.clock.increment(self.color, mv, &self.board);

        Game {
            board,
            color,
            castle_rights,
            ep_rights,
            clock,
            zobrist,
        }
    }

    /// Perform a new move ad modify the game accordingly, checking for mate or draw.
    /// A bit slow, so not suitable for tree search
    pub fn do_move_status(&self, counter: &mut ThreefoldCounter, mv: Move) -> (GameStatus, Game, HashMap<String, Move>) {
        let new_game = self.do_move(mv);
        let legals = new_game.legals().to_map();

        if 
            // 50 moves rule
            new_game.clock.get_halfmoves() == 50 ||
            // Threefold repetition rule
            counter.is_draw(mv, &self.board, &new_game)
        {
            return (GameStatus::Drawn, new_game, HashMap::new());
        }

        // No legal moves
        if legals.is_empty() {
            return (if new_game.board.is_king_in_check(new_game.color) {
                GameStatus::Won {winner: self.color}
            } else {
                GameStatus::Drawn
            }, new_game, HashMap::new());
        }

        (GameStatus::Playing {playing: new_game.color}, new_game, legals)
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

    /// Return the zobrist key of that position
    #[inline(always)]
    pub fn get_key(&self) -> u64 {
        self.zobrist
    }

    /// Try to parse a move from current position with given coordinates,
    /// in pure algebraic notation, of course.
    /// Does not verify the validity of the move.
    pub fn parse_move(&self, s: &str) -> Result<Move, ParseFenError> {
        match s.len() {
            4 => {
                let from = Square::from_str(&s[0..2])?;
                let to = Square::from_str(&s[2..4])?;

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
                let from = Square::from_str(&s[0..2])?;
                let to = Square::from_str(&s[2..4])?;

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

// ================================ pub(crate) impl

impl Game {
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
}

// ================================ traits impl

impl Default for Game {
    fn default() -> Game {
        Game::from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
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
            return Err(ParseFenError::new("missing informations in FEN notation"));
        }

        let mut game = Game {
            board: Board::from_str(strings[0])?,
            color: Color::from_str(strings[1])?,
            castle_rights: CastleRights::from_str(strings[2])?,
            ep_rights: EnPassantSquare::from_str(strings[3])?,
            clock: Clock::from_strs(strings[4], strings[5])?,
            zobrist: 0,
        };
        game.zobrist = Position::from(&game).get_key();

        Ok(game)
    }
}