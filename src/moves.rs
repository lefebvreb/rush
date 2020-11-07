use crate::piece::Piece;
use crate::square::Square;

//#################################################################################################
//
//                                          Move 
//
//#################################################################################################

/// A comapct 32 bit type to represent a single move
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Move(u32);

impl Move {
    /// Create a new quiet Move
    #[inline(always)]
    pub const fn quiet(from: Square, to: Square) -> Move {
        Move((from as u32) << 3 | (to as u32) << 9)
    }

    /// Create a new capture Move
    #[inline(always)]
    pub const fn capture(from: Square, to: Square, capture: Piece) -> Move {
        Move(1 | Move::quiet(from, to).0 | (capture as u32) << 15)
    }

    /// Create a new promotion Move
    #[inline(always)]
    pub const fn promote(from: Square, to: Square, promote: Piece) -> Move {
        Move(2 | Move::quiet(from, to).0 | (promote as u32) << 18)
    }

    /// Create a new promotion and capture Move
    #[inline(always)]
    pub const fn promote_capture(from: Square, to: Square, capture: Piece, promote: Piece) -> Move {
        Move(3 | Move::quiet(from, to).0 | (capture as u32) << 15 | (promote as u32) << 18)
    }

    /// Create a new en passant Move
    #[inline(always)]
    pub const fn en_passant(from: Square, to: Square) -> Move {
        Move(4 | Move::quiet(from, to).0)
    }

    /// Create a new double push Move
    #[inline(always)]
    pub const fn double_push(from: Square, to: Square) -> Move {
        Move(5 | Move::quiet(from, to).0)
    }

    /// Create a new king side castle Move
    #[inline(always)]
    pub const fn kingcastle() -> Move {
        Move(6)
    }

    /// Create a new queen side castle Move
    #[inline(always)]
    pub const fn queencastle() -> Move {
        Move(7)
    }
}

//#################################################################################################
//
//                                          MoveType 
//
//#################################################################################################

/// A convenient enum to manipulate moves
#[derive(PartialEq, Debug)]
pub enum MoveType {
    Quiet {          // 0
        from: Square,
        to: Square,
    },
    Capture {        // 1
        from: Square,
        to: Square,
        capture: Piece,
    },
    Promote {        // 2
        from: Square,
        to: Square,
        promote: Piece,
    },
    PromoteCapture { // 3
        from: Square,
        to: Square,
        capture: Piece,
        promote: Piece,
    },
    EnPassant {      // 4
        from: Square,
        to: Square,
    },
    DoublePush {     // 5
        from: Square,
        to: Square,
    },
    KingCastle,      // 6
    QueenCastle,     // 7
}

impl Into<MoveType> for Move {
    #[inline]
    fn into(self) -> MoveType {
        macro_rules! extract {
            (from) => {
                Square::from(self.0.wrapping_shr(3) & 0x3F)
            };
            (to) => {
                Square::from(self.0.wrapping_shr(9) & 0x3F)
            };
            (capture) => {
                Piece::from((self.0.wrapping_shr(15) & 0x7))
            };
            (promote) => {
                Piece::from((self.0.wrapping_shr(18) & 0x7))
            };
        }

        match self.0 & 0x7 {
            0 => MoveType::Quiet {
                from: extract!(from),
                to: extract!(to),
            },
            1 => MoveType::Capture {
                from: extract!(from),
                to: extract!(to),
                capture: extract!(capture),
            },
            2 => MoveType::Promote {
                from: extract!(from),
                to: extract!(to),
                promote: extract!(promote),
            },
            3 => MoveType::PromoteCapture {
                from: extract!(from),
                to: extract!(to),
                capture: extract!(capture),
                promote: extract!(promote),
            },
            4 => MoveType::EnPassant {
                from: extract!(from),
                to: extract!(to),
            },
            5 => MoveType::DoublePush {
                from: extract!(from),
                to: extract!(to),
            },
            6 => MoveType::KingCastle,
            7 => MoveType::QueenCastle,
            _ => unreachable!(),
        }
    }
}

//#################################################################################################
//
//                                            Test 
//
//#################################################################################################

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correctness() {
        let mv = Move::quiet(Square::C1, Square::C8);
        assert_eq!(MoveType::Quiet {
            from: Square::C1,
            to: Square::C8,
        }, mv.into());

        let mv = Move::capture(Square::D5, Square::D7, Piece::Bishop);
        assert_eq!(MoveType::Capture {
            from: Square::D5,
            to: Square::D7,
            capture: Piece::Bishop,
        }, mv.into());

        let mv = Move::promote(Square::H7, Square::H8, Piece::Queen);
        assert_eq!(MoveType::Promote {
            from: Square::H7,
            to: Square::H8,
            promote: Piece::Queen,
        }, mv.into());

        let mv = Move::promote_capture(Square::B4, Square::A4, Piece::Rook, Piece::Knight);
        assert_eq!(MoveType::PromoteCapture {
            from: Square::B4,
            to: Square::A4,
            capture: Piece::Rook,
            promote: Piece::Knight,
        }, mv.into());

        let mv = Move::double_push(Square::G7, Square::G5);
        assert_eq!(MoveType::DoublePush {
            from: Square::G7,
            to: Square::G5,
        }, mv.into());

        let mv = Move::en_passant(Square::E4, Square::D3);
        assert_eq!(MoveType::EnPassant {
            from: Square::E4,
            to: Square::D3,
        }, mv.into());

        let mv = Move::kingcastle();
        assert_eq!(MoveType::KingCastle, mv.into());

        let mv = Move::queencastle();
        assert_eq!(MoveType::QueenCastle, mv.into());
    }
}