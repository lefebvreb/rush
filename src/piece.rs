/// Represent a piece
#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Piece {
    Pawn   = 0,
    Rook   = 1,
    Knight = 2,
    Bishop = 3,
    Queen  = 4,
    King   = 5,
}

impl Piece {
    /// List of pieces, ordered by their values
    pub const PIECES: [Piece; 6] = [
        Piece::Pawn, Piece::Rook, Piece::Knight,
        Piece::Bishop, Piece::Queen, Piece::King,
    ];
}