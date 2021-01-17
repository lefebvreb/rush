use std::fmt;
use std::str::FromStr;

use crate::board::Board;
use crate::castle_rights::CastleRights;
use crate::color::Color;
use crate::errors::ParseFenError;
use crate::history::{LargeMoveHistory, MoveHistory, MoveCounter, SmallMoveHistory};
use crate::moves::Move;
use crate::piece::Piece;
use crate::square::Square;

/// A struct that holds every information defining a complete game of chess
#[derive(Clone, Debug)]
pub struct Game<H: MoveHistory> {
    board: Board,
    color: Color,
    castle_rights: CastleRights,
    history: H,
    clock: MoveCounter,
}

impl<H: MoveHistory> Game<H> {
    /// Perform a new move and modifiy the game accordingly
    #[inline]
    pub fn do_move(&mut self, mv: Move) {
        self.history.push(mv, self.castle_rights, self.clock);
        self.clock = self.clock.increment(self.color, mv, &self.board);
        self.castle_rights = self.castle_rights.update(self.color, mv);

        self.board.do_move(self.color, mv);  

        self.color = self.color.invert();
    }

    /// Revert the last move. Panic if there is no move in history
    #[inline]
    pub fn undo_move(&mut self) {
        self.color = self.color.invert();

        let (mv, castle_rights, clock) = self.history.pop();    
        self.castle_rights = castle_rights;
        self.clock = clock;

        self.board.undo_move(self.color, mv);
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
    pub(crate) fn get_castle_rights(&self) -> &CastleRights {
        &self.castle_rights
    }

    // Return the last move played
    #[inline(always)]
    pub(crate) fn get_last_move(&self) -> Move {
        self.history.last_move()
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
                    'r' | 'R' => Piece::Rook,
                    'n' | 'N' => Piece::Knight,
                    'b' | 'B' => Piece::Bishop,
                    'q' | 'Q' => Piece::Queen,
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
    pub fn from_fen(fen: &str) -> Result<Game<H>, ParseFenError> {
        Game::from_str(fen)
    }
}

impl Game<LargeMoveHistory> {
    /// Return a new SearchGame, able to be used
    #[inline(always)]
    pub fn search_game<const MAX: usize>(&self) -> SearchGame<MAX> {
        Game {
            board: self.board.clone(),
            castle_rights: self.castle_rights.clone(),
            color: self.color.clone(),
            history: {
                let mut history = SmallMoveHistory::default();
                let last_move = self.history.last_move();
                if last_move.is_some() {
                    history.push(last_move, CastleRights::default(), MoveCounter::default());
                }
                history
            },
            clock: self.clock.clone(),
        }
    }
}

impl Default for FullGame {
    // Return a new full game, with the starting position of chess
    fn default() -> FullGame {
        Game {
            board: Board::default(),
            castle_rights: CastleRights::default(),
            color: Color::default(),
            history: LargeMoveHistory::default(),
            clock: MoveCounter::default(),
        }
    }
}

impl<H: MoveHistory> fmt::Display for Game<H> {
    // Give the FEN representation of the board
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {}",
            self.board,
            self.color,
            self.castle_rights,
            self.history.en_passant_square(),
            self.clock,
        )
    }
}

impl<H: MoveHistory> FromStr for Game<H> {
    type Err = ParseFenError;

    // Try to construct a board from a given FEN notation
    fn from_str(s: &str) -> Result<Game<H>, ParseFenError> {
        let strings = s.split(" ").into_iter().collect::<Vec<_>>();
        
        if strings.len() != 6 {
            return Err(ParseFenError::new("missing informations on FEN notation"));
        }

        let castle_rights = CastleRights::from_str(strings[2])?;
        let clock = MoveCounter::from_strs(strings[4], strings[5])?;

        Ok(Game {
            board: Board::from_str(strings[0])?,
            color: Color::from_str(strings[1])?,
            castle_rights,
            history: {
                let mut history = H::default();

                match strings[3] {
                    "-" => (),
                    s => {
                        let sq = Square::from_str(s)?;
                        let (x, y) = (sq.x(), sq.y());
                        let mv = match y {
                            2 => Move::DoublePush {
                                from: Square::from((x, 1)),
                                to: Square::from((x, 3)),
                            },
                            5 => Move::DoublePush {
                                from: Square::from((x, 6)),
                                to: Square::from((x, 4)),
                            },
                            _ => return Err(ParseFenError::new("invalid en passant square")),
                        };
                        history.push(mv, castle_rights, clock);
                    }
                }

                history
            },
            clock,
        })
    }
}

/// A type used to record a full game from beginning to end
pub type FullGame = Game<LargeMoveHistory>;

/// A type used to explore the game tree, in which only MAX moves
/// may be played. It is a little bit more efficient, since it
/// does not involves any heap allocations
pub type SearchGame<const MAX: usize> = Game<SmallMoveHistory<MAX>>;