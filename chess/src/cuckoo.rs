use crate::attacks;
use crate::board::Board;
use crate::color::Color;
use crate::piece::Piece;
use crate::square::Square;
use crate::zobrist::Zobrist;

//#################################################################################################
//
//                                       tables
//
//#################################################################################################

// The cuckoo tables.
static mut CUCKOO: [Zobrist; 8192] = [Zobrist::ZERO; 8192];
static mut SQUARES: [Option<(Square, Square)>; 8192] = [None; 8192];

// Returns true if the move is valid on an empty board.
// Pawn moves are never reversible so we don't take them into account.
#[cold]
unsafe fn is_valid(piece: Piece, from: Square, to: Square) -> bool {
    let occ = from.into();

    match piece {
        Piece::Rook => attacks::rook(from, occ),
        Piece::Bishop => attacks::bishop(from, occ),
        Piece::Knight => attacks::knight(from),
        Piece::Queen => attacks::queen(from, occ),
        Piece::King => attacks::knight(from),
        _ => unreachable!(),
    }.contains(to)
}

// Inserts into the cuckoo table, only if the move is valid.
#[cold]
unsafe fn insert(color: Color, piece: Piece, from: Square, to: Square) {
    if !is_valid(piece, from, to) {
        return;
    }
    
    let mut zobrist = !(Zobrist::from((color, piece, from)) ^ Zobrist::from((color, piece, to)));
    let mut i = zobrist.h1();
    let mut squares = Some((from, to));

    loop {
        // Take that spot.
        std::mem::swap(&mut CUCKOO[i], &mut zobrist);
        std::mem::swap(&mut SQUARES[i], &mut squares);

        // The spot was empty, we are done.
        if zobrist == Zobrist::ZERO {
            break;
        }

        if i == zobrist.h1() {
            i = zobrist.h2();
        } else {
            i = zobrist.h1();
        }
    }
}

// Initializes the cuckoo tables.
#[cold]
pub(crate) unsafe fn init() {
    for color in Color::COLORS {
        for &piece in &Piece::PIECES[1..] {
            for from in Square::SQUARES {
                for &to in &Square::SQUARES[usize::from(from)+1..] {
                    insert(color, piece, from, to);
                }
            }
        }
    }
}

//#################################################################################################
//
//                               is_hash_of_legal_move()
//
//#################################################################################################

// Returns true if the provided zobrist is the hash of a legal reversible move.
// Uses cuckoo hashing to reduce the memory footprint of the hash table.
#[inline]
pub(crate) fn is_hash_of_legal_move(board: &Board, diff: Zobrist) -> bool {
    // SAFETY: h1 and h2 always yield numbers that are < 8192
    unsafe {
        let mut i = diff.h1();

        if *CUCKOO.get_unchecked(i) != diff {
            i = diff.h2();
            if *CUCKOO.get_unchecked(i) != diff {
                return false;
            }
        }

        let (from, to) = SQUARES.get_unchecked(i).unwrap();
        board.is_path_clear(from, to)
    }
}