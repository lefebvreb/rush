use std::alloc::{self, Layout};
use std::fs::File;
use std::io::Read;
use std::mem;
use std::ops::Shl;
use std::path::Path;
use std::sync::Arc;

use anyhow::{Error, Result};

use chess::board::Board;
use chess::moves::Move;
use chess::piece::Piece;
use chess::prelude::Color;
use chess::square::Square;

use crate::utils;

/// Returns the heuristic value of a piece, in pawns.
#[inline]
pub(crate) const fn value_of(piece: Piece) -> f32 {
    match piece {
        Piece::Pawn => 1.0,
        Piece::Rook => 5.0,
        Piece::Knight => 3.2,
        Piece::Bishop => 3.3,
        Piece::Queen => 9.0,
        Piece::King => 1000.0,
    }
}

//#################################################################################################
//
//                                         struct Net
//
//#################################################################################################

/// Represents a neural network used for evaluation.
#[derive(Debug)]
pub(crate) struct Net {
    w0: [[f32; Net::SIZE]; Net::HEIGHT],
    b0: [f32; Net::SIZE],
    w1: [[f32; 32]; 2 * Net::SIZE],
    b1: [f32; 32],
    w2: [[f32; 32]; 32],
    b2: [f32; 32],
    w3: [f32; 32],
    b3: f32,
}

// ================================ pub(crate) impl

impl Net {
    /// Loads a neural network from a file located at the given path.
    pub(crate) fn load(path: &Path) -> Result<Arc<Net>> {
        let mut file = File::open(path).map_err(|_| Error::msg("Cannot open network file."))?;

        fn read_f32(file: &mut File, x: &mut f32) -> Result<()> {
            let mut buf = [0; 4];
            file.read(&mut buf).map_err(|_| Error::msg("Not enough bytes in network file."))?;
            *x = f32::from_be_bytes(buf);
            Ok(())
        }

        fn read_vec<const N: usize>(file: &mut File, vec: &mut [f32; N]) -> Result<()> {
            for i in 0..N {
                read_f32(file, &mut vec[i])?;
            }
            Ok(())
        }

        fn read_mat<const N: usize, const M: usize>(file: &mut File, mat: &mut [[f32; M]; N]) -> Result<()> {
            for i in 0..N {
                read_vec(file, &mut mat[i])?;
            }
            Ok(())
        }

        // Done with manual allocation so as not to overflow the stack with the Net struct.
        // SAFE: Arc is specified to accept pointers allocated with std::alloc::alloc()
        Ok(unsafe {
            let ptr = alloc::alloc(Layout::new::<Net>()) as *mut Net;

            read_mat(&mut file, &mut (*ptr).w0)?;
            read_vec(&mut file, &mut (*ptr).b0)?;
            read_mat(&mut file, &mut (*ptr).w1)?;
            read_vec(&mut file, &mut (*ptr).b1)?;
            read_mat(&mut file, &mut (*ptr).w2)?;
            read_vec(&mut file, &mut (*ptr).b2)?;
            read_vec(&mut file, &mut (*ptr).w3)?;
            read_f32(&mut file, &mut (*ptr).b3)?;

            Arc::from_raw(ptr)
        })
    }
}

// ================================ impl

impl Net {
    /// Must be kept in sync with the constant of the same name in the training script.
    const SIZE: usize = 128;

    /// 64 piece's squares x 64 king's square x 5 non-king piece types x 2 colors.
    const HEIGHT: usize = 40960;
}

//#################################################################################################
//
//                                      struct Accumulator
//
//#################################################################################################

/// A struct used to efficiently evaluate the first layer of the neural network.
#[derive(Clone, Debug)]
struct Accumulator {
    white: [f32; Net::SIZE],
    black: [f32; Net::SIZE],
}

// ================================ impl

impl Accumulator {
    /// Creates and initializes a new Accumulator.
    #[inline]
    fn new(net: &Net) -> Accumulator {
        Accumulator {
            white: net.b0,
            black: net.b0,
        }
    }

    /// Concatenates the accumulator into a single array, ready for the
    /// transform part of the network inference.
    #[inline]
    fn cat(&self, color: Color) -> [f32; 2 * Net::SIZE] {
        let mut res = [0.0; 2 * Net::SIZE];
        let (mut left, mut right) = res.split_at_mut(Net::SIZE);

        if color == Color::Black {
            mem::swap(&mut left, &mut right);
        }

        left.clone_from_slice(&self.white);
        right.clone_from_slice(&self.black);

        res
    }

    #[inline]
    fn add_w(&mut self, feature: usize, net: &Net) {
        for i in 0..Net::SIZE {
            self.white[i] += net.w0[feature][i];
        }
    }

    #[inline]
    fn add_b(&mut self, feature: usize, net: &Net) {
        for i in 0..Net::SIZE {
            self.black[i] += net.w0[feature][i];
        }
    }

    #[inline]
    fn sub_w(&mut self, feature: usize, net: &Net) {
        for i in 0..Net::SIZE {
            self.white[i] -= net.w0[feature][i];
        }
    }

    #[inline]
    fn sub_b(&mut self, feature: usize, net: &Net) {
        for i in 0..Net::SIZE {
            self.black[i] -= net.w0[feature][i];
        }
    }
}

//#################################################################################################
//
//                                       struct GlobalInfo
//
//#################################################################################################

/// A struct designed to handle evaluation of the board.
#[derive(Debug)]
pub(crate) struct Eval {
    king_w: usize,
    king_b: usize,

    acc: Accumulator,
    prev_acc: Vec<Accumulator>,

    net: Arc<Net>,
}

// ================================ pub(crate) impl

impl Eval {
    /// Creates a new Eval struct.
    pub(crate) fn new(net: Arc<Net>) -> Eval {
        Eval {
            king_w: 0,
            king_b: 0,
            acc: Accumulator::new(&net),
            prev_acc: Vec::new(),
            net,
        }
    }

    /// Resets the Eval struct for the given state.
    #[inline]
    pub(crate) fn reset(&mut self, board: &Board) {
        self.prev_acc.clear();
        self.acc = Accumulator::new(&self.net);
        
        self.update_king(Color::White, board);
        self.update_king(Color::Black, board);

        for sq in board.get_occupancy().all().iter_squares() {
            let (color, piece) = board.get_piece(sq).unwrap();
            if piece != Piece::King {
                self.add_piece(color, piece, sq);
            }
        }
    }

    /// Updates the evaluation score from the position and the
    /// last move played.
    #[inline]
    pub(crate) fn do_move(&mut self, board: &mut Board, mv: Move) {
        let (from, to) = mv.squares();
        let (color, piece) = board.get_piece(from).unwrap();

        // If it's a king move, update the half that needs to be.
        if piece == Piece::King {
            self.prev_acc.push(self.acc.clone());

            board.do_move(mv);
            self.update_side(color, board);

            // If it's a king capture, remove the capturee from the other side's accumulator.
            if mv.is_capture() {
                if color == Color::White {
                    let feature = self.feature_b(Color::Black, mv.get_capture(), to);
                    self.acc.sub_b(feature, &self.net);
                } else {
                    let feature = self.feature_w(Color::White, mv.get_capture(), to);
                    self.acc.sub_w(feature, &self.net);
                }
            }

            // If it's a castle, update the position of the rook on the other side's accumulator.
            if mv.is_castle() {
                let (from, to) = match mv.to() {
                    Square::G1 => (Square::H1, Square::F1),
                    Square::C1 => (Square::A1, Square::D1),
                    Square::G8 => (Square::H8, Square::F8),
                    Square::C8 => (Square::A8, Square::D8),
                    _ => unreachable!(),
                };

                if color == Color::White {
                    let feature_1 = self.feature_b(color, piece, from);
                    let feature_2 = self.feature_b(color, piece, from);

                    self.acc.sub_b(feature_1, &self.net);
                    self.acc.add_b(feature_2, &self.net);
                } else {
                    let feature_1 = self.feature_w(color, piece, from);
                    let feature_2 = self.feature_w(color, piece, from);

                    self.acc.sub_w(feature_1, &self.net);
                    self.acc.add_w(feature_2, &self.net);
                }
            }

            return;
        }

        // Remove the piece from it's old position.
        self.remove_piece(color, piece, from);

        // Place the new piece at it's position.
        if mv.is_promote() {
            self.add_piece(color, mv.get_promote(), to);
        } else {
            self.add_piece(color, piece, to);
        }

        // If it's a capture, remove the capturee.
        if mv.is_capture() {
            self.remove_piece(color.invert(), mv.get_capture(), to);
        } else if mv.is_en_passant() {
            self.remove_piece(color.invert(), Piece::Pawn, board.get_ep_square().unwrap());
        }

        board.do_move(mv);
    }

    /// Updates the evaluation score from the position and the
    /// last move unplayed.
    #[inline]
    pub(crate) fn undo_move(&mut self, board: &mut Board, mv: Move) {
        board.undo_move(mv);

        let (from, to) = mv.squares();
        let (color, piece) = board.get_piece(from).unwrap();

        if piece == Piece::King {
            self.acc = self.prev_acc.pop().unwrap();
            self.update_king(color, board);
            return;
        }

        // Replace the piece at it's old position.
        self.add_piece(color, piece, from);

        // Remove the new piece from it's new position.
        if mv.is_promote() {
            self.remove_piece(color, mv.get_promote(), to);
        } else {
            self.remove_piece(color, piece, to);
        }

        // If it's a capture, replace the capturee.
        if mv.is_capture() {
            self.add_piece(color.invert(), mv.get_capture(), to);
        } else if mv.is_en_passant() {
            self.add_piece(color.invert(), Piece::Pawn, board.get_ep_square().unwrap());
        }
    }

    /// Returns the value of the evaluation.
    #[inline]
    pub(crate) fn get(&self, color: Color) -> f32 {
        #[inline(always)]
        fn clamp(buf: &mut [f32]) {
            for i in 0..buf.len() {
                buf[i] = buf[i].max(0.0).min(1.0);
            }
        }

        // First layer.
        let mut buf0 = self.acc.cat(color);
        clamp(&mut buf0);

        // Second layer.
        let mut buf1 = self.net.b1;
        for i in 0..32 {
            for j in 0..(2 * Net::SIZE) {
                buf1[i] += self.net.w1[j][i] * buf0[j];
            }
        }
        clamp(&mut buf1);

        // Third layer.
        let mut buf2 = self.net.b2;
        for i in 0..32 {
            for j in 0..32 {
                buf2[i] += self.net.w2[j][i] * buf1[j];
            }
        }
        clamp(&mut buf2);

        // Last layer.
        let mut res = self.net.b3;
        for i in 0..32 {
            res += self.net.w3[i] * buf2[i];
        }
        
        // For negamax frameworks, the evaluation needs to be inverted for black
        if color == Color::Black {
            res = -res;
        }

        res
    }
}

// ================================ impl

impl Eval {
    /// Computes the feature associated with a color, piece, square triplet for white.
    #[inline]
    fn feature_w(&self, color: Color, piece: Piece, sq: Square) -> usize {
        self.king_w + (((usize::from(piece) << 1) + usize::from(color)) << 6) + usize::from(sq)
    }

    /// Computes the feature associated with a color, piece, square triplet for black.
    #[inline]
    fn feature_b(&self, color: Color, piece: Piece, sq: Square) -> usize {
        self.king_b + (((usize::from(piece) << 1) + 1 - usize::from(color)) << 6) + (usize::from(sq) ^ 56)
    }

    /// Takes the given piece triplet into account.
    #[inline]
    fn add_piece(&mut self, color: Color, piece: Piece, sq: Square) {
        let feature_w = self.feature_w(color, piece, sq);
        let feature_b = self.feature_b(color, piece, sq);

        self.acc.add_w(feature_w, &self.net);
        self.acc.add_b(feature_b, &self.net);
    }

    /// Removes the given piece triplet from the accumulator.
    #[inline]
    fn remove_piece(&mut self, color: Color, piece: Piece, sq: Square) {
        let feature_w = self.feature_w(color, piece, sq);
        let feature_b = self.feature_b(color, piece, sq);

        self.acc.sub_w(feature_w, &self.net);
        self.acc.sub_b(feature_b, &self.net);
    }

    /// Updates the king square value of the specified color.
    #[inline]
    fn update_king(&mut self, color: Color, board: &Board) {
        if color == Color::White {
            self.king_w = 640 * usize::from(board.king_sq(Color::White));
        } else {
            self.king_b = 640 * (usize::from(board.king_sq(Color::Black)) ^ 56);
        }
    }

    #[inline]
    fn update_side(&mut self, color: Color, board: &Board) {
        self.update_king(color, board);

        if color == Color::White {
            self.acc.white = self.net.b0;

            for sq in board.get_occupancy().all().iter_squares() {
                let (color, piece) = board.get_piece(sq).unwrap();

                if piece != Piece::King {
                    let feature = self.feature_w(color, piece, sq);
                    self.acc.add_w(feature, &self.net);
                }
            }
        } else {
            self.acc.black = self.net.b0;

            for sq in board.get_occupancy().all().iter_squares() {
                let (color, piece) = board.get_piece(sq).unwrap();

                if piece != Piece::King {
                    let feature = self.feature_b(color, piece, sq);
                    self.acc.add_b(feature, &self.net);
                }
            }
        }
    }
}