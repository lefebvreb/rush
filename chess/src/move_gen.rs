use crate::attacks::{get_pinned, pin_mask};
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

    // if single check =>
    PawnCaptureSingleCheck,
    RookCaptureSingleCheck,
    KnightCaptureSingleCheck,
    BishopCaptureSingleCheck,
    QueenCaptureSingleCheck,
    KingCaptureSingleCheck,
    EnPassantSingleCheck,
    PawnBlock,
    RookBlock,
    KnightBlock,
    BishopBlock,
    QueenBlock,
    // goto KingEvade

    // if double check =>
    KingCaptureDoubleCheck,
    KingEvade,
    // goto Stop
    
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
    pub fn legals(color: Color, game: &Game) -> MoveGenerator {
        let board = game.get_board();

        let color_inv = color.invert();

        let king_square = board.get_bitboard(color, Piece::King).first_square();
        let king_attacks = board.get_attacks(king_square);

        let (state, moves_buffer, piece_buffer) = match king_attacks.card() {
            0 => (
                State::PawnCapture, 
                board.get_bitboard(color, Piece::Pawn),
                BitBoard(0),
            ),
            1 => (
                State::PawnCaptureSingleCheck, 
                board.get_bitboard(color, Piece::Pawn),
                BitBoard(0),
            ),
            2 => (
                State::KingCaptureDoubleCheck, 
                board.get_bitboard(color, Piece::King),
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
            castle_availability: game.castle_rights.get_availability(color, board.get_occupancy(), danger),
            safe: !danger,
            pinned: get_pinned(color, board),
            from: Square::None,
            pawn_promote: PawnPromote::Queen,
            moves_buffer,
            piece_buffer,
        }
    }

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
                        self.moves_buffer = board.get_defend_unchecked(self.from) & $mask;
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
                State::EnPassant => todo!(),
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
                    
                }
                State::PawnDoublePush => todo!(),
                State::RookQuiet => todo!(),
                State::KnightQuiet => todo!(),
                State::BishopQuiet => todo!(),
                State::QueenQuiet => todo!(),
                State::KingQuiet => todo!(),

                State::PawnCaptureSingleCheck => todo!(),
                State::RookCaptureSingleCheck => todo!(),
                State::KnightCaptureSingleCheck => todo!(),
                State::BishopCaptureSingleCheck => todo!(),
                State::QueenCaptureSingleCheck => todo!(),
                State::KingCaptureSingleCheck => todo!(),
                State::EnPassantSingleCheck => todo!(),
                State::PawnBlock => todo!(),
                State::RookBlock => todo!(),
                State::KnightBlock => todo!(),
                State::BishopBlock => todo!(),
                State::QueenBlock => todo!(),

                State::KingCaptureDoubleCheck => todo!(),
                State::KingEvade => todo!(),

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