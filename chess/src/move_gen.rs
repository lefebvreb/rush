use crate::attacks::{double_push, get_pinned, pawn_push, pin_mask, squares_between};
use crate::bitboard::BitBoard;
use crate::castle_rights::CastleAvailability;
use crate::color::Color;
use crate::game::Game;
use crate::moves::Move;
use crate::piece::Piece;
use crate::square::Square;

#[repr(u8)]
enum State {
    // if no check =>
    PawnCapture,
    RookCapture,
    KnightCapture,
    BishopCapture,
    QueenCapture,
    KingCapture,
    EnPassant,
    KingCastle,
    QueenCastle,
    PawnSinglePush,
    PawnDoublePush,
    RookQuiet,
    KnightQuiet,
    BishopQuiet,
    QueenQuiet,
    KingQuiet,
    // goto Stop

    // if double check =>
    KingCaptureDoubleCheck,
    // goto KingQuiet
    
    Stop,
}

#[repr(u8)]
enum PawnPromote {
    Queen,
    Rook,
    Bishop,
    Knight,
}

impl PawnPromote {
    #[inline(always)]
    pub fn next(&mut self) {
        *self = match self {
            PawnPromote::Queen => PawnPromote::Rook,
            PawnPromote::Rook => PawnPromote::Bishop,
            PawnPromote::Bishop => PawnPromote::Knight,
            PawnPromote::Knight => PawnPromote::Queen,
        };
    }
}

pub struct MoveGenerator<'game> {
    state: State,
    game: &'game Game,
    color_inv: Color,
    king_square: Square,
    check_mask: BitBoard,
    castle_availability: CastleAvailability,
    safe: BitBoard,
    pinned: BitBoard,
    from: Square,
    pawn_promote: PawnPromote,
    moves_buffer: BitBoard,
    piece_buffer: BitBoard,
}

impl MoveGenerator<'_> {
    #[inline(always)]
    pub fn legals(game: &Game) -> MoveGenerator {
        let board = game.get_board();
        let color = game.get_color();

        let color_inv = color.invert();

        let king_square = board.get_bitboard(color, Piece::King).first_square();
        let king_attacks = board.get_attacks(king_square) & board.get_color_occupancy(color_inv);

        let (state, moves_buffer, piece_buffer, check_mask) = match king_attacks.card() {
            0 => (
                State::PawnCapture, 
                board.get_bitboard(color, Piece::Pawn),
                BitBoard(0),
                BitBoard(0),
            ),
            1 => (
                State::PawnCapture, 
                board.get_bitboard(color, Piece::Pawn),
                BitBoard(0),
                squares_between(king_square, king_attacks.first_square()) | king_attacks,
            ),
            2 => (
                State::KingCaptureDoubleCheck, 
                board.get_bitboard(color, Piece::King),
                BitBoard(0),
                BitBoard(0),
            ),
            _ => unreachable!(),
        };

        let danger = {
            let mut res = BitBoard(0);
            for from in board.get_color_occupancy(color_inv).iter_squares() {
                res |= board.get_defend_unchecked(from);
            }
            !res
        };

        MoveGenerator {
            state,
            game,
            color_inv,
            king_square,
            check_mask,
            castle_availability: game.castle_rights.get_availability(color, board.get_occupancy(), danger),
            safe: !danger,
            pinned: get_pinned(color, board),
            from: Square::None,
            pawn_promote: PawnPromote::Queen,
            moves_buffer,
            piece_buffer,
        }
    }

    // Behold ! The behemoth
    #[inline(always)]
    pub fn next(&mut self) -> Move {
        let color = self.game.get_color();
        let board = self.game.get_board();

        macro_rules! next_move {
            ($mask: expr, $next_piece: expr, $next_state: expr, $encode_move: expr) => {
                if let Some(to_bitboard) = self.moves_buffer.pop_first_bitboard() {
                    return $encode_move(to_bitboard.first_square(), to_bitboard);
                } else {
                    if let Some(from_bitboard) = self.piece_buffer.pop_first_bitboard() {
                        self.from = from_bitboard.first_square();
                        self.moves_buffer = board.get_defend_unchecked(self.from) & $mask & self.check_mask;
                        if (from_bitboard & self.pinned).is_not_empty() {
                            self.moves_buffer &= pin_mask(self.king_square, self.from);
                        }
                    } else {
                        self.piece_buffer = board.get_bitboard(color, $next_piece);
                        self.moves_buffer = BitBoard(0);
                        self.state = $next_state;
                    }
                }
            };
        }

        macro_rules! capture {
            ($next_piece: expr, $next_state: expr) => {
                next_move!(
                    board.get_color_occupancy(self.color_inv), $next_piece, $next_state, |to: Square, _| 
                        Move::Capture {
                            from: self.from, 
                            to, 
                            capture: board.get_piece_unchecked(to),
                        }
                )
            };
        }

        macro_rules! quiet {
            ($next_piece: expr, $next_state: expr) => {
                next_move!(
                    board.get_free(), $next_piece, $next_state, |to: Square, _| 
                        Move::Quiet {
                            from: self.from, 
                            to,
                        }
                )
            };
        }

        loop {
            match self.state {
                State::PawnCapture => next_move!(
                    board.get_color_occupancy(self.color_inv), Piece::Rook, State::RookCapture, |to: Square, mask: BitBoard| {
                        if to.is_last_rank(color) {
                            let promote = match self.pawn_promote {
                                PawnPromote::Queen => {
                                    self.moves_buffer |= mask;
                                    Piece::Queen
                                },
                                PawnPromote::Rook => {
                                    self.moves_buffer |= mask;
                                    Piece::Rook
                                },
                                PawnPromote::Bishop => {
                                    self.moves_buffer |= mask;
                                    Piece::Bishop
                                },
                                PawnPromote::Knight => Piece::Knight
                            };
                            self.pawn_promote.next();
                            Move::PromoteCapture {
                                from: self.from, 
                                to, 
                                capture: board.get_piece_unchecked(to),
                                promote,
                            }
                        } else {
                            Move::Capture {
                                from: self.from, 
                                to, 
                                capture: board.get_piece_unchecked(to),
                            }
                        }
                }),
                State::RookCapture => capture!(Piece::Knight, State::KnightCapture),
                State::KnightCapture => capture!(Piece::Bishop, State::BishopCapture),
                State::BishopCapture => capture!(Piece::Queen, State::QueenCapture),
                State::QueenCapture => capture!(Piece::King, State::KingCapture),
                State::KingCapture => next_move!(
                    board.get_color_occupancy(self.color_inv) & self.safe, Piece::Pawn, State::EnPassant, |to: Square, _| 
                        Move::Capture {
                            from: self.from, 
                            to, 
                            capture: board.get_piece_unchecked(to),
                        }
                ),
                State::EnPassant => {
                    // TODO
                    self.state = State::KingCastle;
                },
                State::KingCastle => match self.castle_availability {
                    CastleAvailability::KingSide | CastleAvailability::Both => {
                        self.state = State::QueenCastle;
                        return Move::KingCastle;
                    }
                    CastleAvailability::QueenSide => self.state = State::QueenCastle,
                    CastleAvailability::None => {
                        self.piece_buffer = board.get_bitboard(color, Piece::Pawn);
                        self.moves_buffer = BitBoard(0);
                        self.state = State::PawnSinglePush
                    },
                }
                State::QueenCastle => {
                    self.piece_buffer = board.get_bitboard(color, Piece::Pawn);
                    self.moves_buffer = BitBoard(0);
                    self.state = State::PawnSinglePush;
                    match self.castle_availability {
                        CastleAvailability::QueenSide | CastleAvailability::Both => return Move::QueenCastle,
                        _ => (),
                    }
                }
                State::PawnSinglePush => if let Some(from_bitboard) = self.piece_buffer.pop_first_bitboard() {
                    self.from = from_bitboard.first_square();

                    self.moves_buffer = if (from_bitboard & self.pinned).is_empty() {
                        BitBoard(0xFFFFFFFFFFFFFFFF)
                    } else {
                        pin_mask(self.king_square, self.from)
                    };

                    let push = self.moves_buffer & board.get_free() & self.check_mask & pawn_push(color, self.from).into();

                    if push.is_not_empty() {
                        self.state = State::PawnDoublePush;
                        return Move::Quiet {
                            from: self.from,
                            to: push.first_square(),
                        };
                    }
                } else {
                    self.piece_buffer = board.get_bitboard(color, Piece::Rook);
                    self.moves_buffer = BitBoard(0);
                    self.state = State::RookQuiet
                }
                State::PawnDoublePush => {
                    let push = self.moves_buffer & board.get_free() & self.check_mask & double_push(color, self.from).into();

                    self.state = State::PawnSinglePush;

                    if push.is_not_empty() {
                        return Move::DoublePush {
                            from: self.from,
                            to: push.first_square(),
                        };
                    }
                },
                State::RookQuiet => quiet!(Piece::Knight, State::KnightQuiet),
                State::KnightQuiet => quiet!(Piece::Bishop, State::BishopQuiet),
                State::BishopQuiet => quiet!(Piece::Queen, State::QueenQuiet),
                State::QueenQuiet => quiet!(Piece::King, State::KingQuiet),
                State::KingQuiet => next_move!(
                    board.get_free() & self.safe, /* useless fetch */ Piece::Pawn, State::Stop, |to: Square, _| 
                        Move::Quiet {
                            from: self.from, 
                            to,
                        }
                ),

                State::KingCaptureDoubleCheck => next_move!(
                    board.get_color_occupancy(self.color_inv) & self.safe, /* useless fetch */ Piece::King, State::KingQuiet, |to: Square, _| 
                        Move::Capture {
                            from: self.from, 
                            to, 
                            capture: board.get_piece_unchecked(to),
                        }
                ),

                State::Stop => {
                    return Move::None;
                }
            }
        }
    }

    #[cold]
    pub fn collect(&mut self) -> Vec<Move> {
        let mut res = Vec::new();

        loop {
            let mv = self.next();
            if mv.is_none() {
                break;
            } else {
                res.push(mv)
            }
        }

        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opening() {
        let game = Game::default();

        let mut move_gen = MoveGenerator::legals(&game);

        let moves = move_gen.collect();

        println!("{:?}", moves.len());
        println!("{:?}", moves);
    }
}