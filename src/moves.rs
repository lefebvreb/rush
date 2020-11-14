use crate::piece::Piece;
use crate::square::Square;

/// A convenient enum to manipulate moves
#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Move {
    Quiet {
        from: Square,
        to: Square,
    },
    Capture {
        from: Square,
        to: Square,
        capture: Piece,
    },
    Promote {
        from: Square,
        to: Square,
        promote: Piece,
    },
    PromoteCapture {
        from: Square,
        to: Square,
        capture: Piece,
        promote: Piece,
    },
    EnPassant {
        from: Square,
        to: Square,
    },
    DoublePush {
        from: Square,
        to: Square,
    },
    KingCastle,
    QueenCastle,
}