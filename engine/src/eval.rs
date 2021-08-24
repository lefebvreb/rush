use std::fs::File;
use std::io::Read;
use std::mem;
use std::path::Path;
use std::sync::Arc;

use anyhow::{Error, Result};

use chess::board::Board;
use chess::moves::Move;
use chess::piece::Piece;
use chess::prelude::Color;

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
    pub w0: [[f32; Net::SIZE]; Net::HEIGHT],
    pub b0: [f32; Net::SIZE],
    pub w1: [[f32; 32]; 2 * Net::SIZE],
    pub b1: [f32; 32],
    pub w2: [[f32; 32]; 32],
    pub b2: [f32; 32],
    pub w3: [f32; 32],
    pub b3: f32,
}

// ================================ pub(crate) impl

impl Net {
    pub fn load(path: &Path) -> Result<Net> {
        let mut file = File::open(path).map_err(|_| Error::msg("Cannot open network file."))?;

        fn read_f32(file: &mut File) -> Result<f32> {
            let mut buf = [0; 4];
            file.read(&mut buf).map_err(|_| Error::msg("Not enough bytes in network file."))?;
            Ok(f32::from_be_bytes(buf))
        }

        fn read_vec<const N: usize>(file: &mut File) -> Result<[f32; N]> {
            let mut res = [0.0; N];
            for i in 0..N {
                res[i] = read_f32(file)?;
            }
            Ok(res)
        }

        fn read_mat<const N: usize, const M: usize>(file: &mut File) -> Result<[[f32; M]; N]> {
            let mut res = [[0.0; M]; N];
            for i in 0..N {
                res[i] = read_vec(file)?;
            }
            Ok(res)
        }

        Ok(Net {
            w0: read_mat(&mut file)?,
            b0: read_vec(&mut file)?,
            w1: read_mat(&mut file)?,
            b1: read_vec(&mut file)?,
            w2: read_mat(&mut file)?,
            b2: read_vec(&mut file)?,
            w3: read_vec(&mut file)?,
            b3: read_f32(&mut file)?,
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
//                                       struct GlobalInfo
//
//#################################################################################################

/// A struct designed to handle evaluation of the board.
#[derive(Clone, Debug)]
pub(crate) struct Eval {
    white_acc: [f32; Net::SIZE],
    black_acc: [f32; Net::SIZE],
}

// ================================ pub(crate) impl

impl Eval {
    /// Creates a new Eval struct.
    pub(crate) fn new(net: Arc<Net>) -> Eval {
        todo!()
    }

    /// Resets the Eval struct for the given state.
    #[inline]
    pub(crate) fn reset(&mut self, board: &Board) {
        todo!()
    }

    /// Updates the evaluation score from the position and the
    /// last move played.
    #[inline]
    pub(crate) fn do_move(&mut self, board: &Board, mv: Move) {
        todo!()
    }

    /// Updates the evaluation score from the position and the
    /// last move unplayed.
    #[inline]
    pub(crate) fn undo_move(&mut self, board: &Board, mv: Move) {
        todo!()
    }

    /// Returns the value of the evaluation.
    #[inline]
    pub(crate) fn get(&self) -> f32 {
        todo!()
    }
}