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
static mut SQUARES: [(Square, Square); 8192] = [(Square::A1, Square::A1); 8192];

// Returns true if the move is valid on an empty board.
// Pawn moves are never reversible so we don't take them into account.
#[cold]
unsafe fn is_valid(color: Color, piece: Piece, from: Square, to: Square) -> bool {
    match piece {
        Piece::Rook   => attacks::rook(from, from.into()).contains(to),
        Piece::Bishop => attacks::bishop(from, from.into()).contains(to),
        Piece::Knight => attacks::knight(from).contains(to),
        Piece::Queen  => attacks::queen(from, from.into()).contains(to),
        Piece::King   => attacks::knight(from).contains(to),
        _ => unreachable!(),
    }
}

// Inserts into the cuckoo table, only if the move is valid.
#[cold]
unsafe fn insert(color: Color, piece: Piece, from: Square, to: Square) {
    if !is_valid(color, piece, from, to) {             
        return;
    }
    
    let mut zobrist = !(Zobrist::from((color, piece, from)) ^ Zobrist::from((color, piece, to)));
    let mut i = zobrist.h1();
    let mut squares = (from, to);

    loop {
        // Take that spot
        std::mem::swap(&mut CUCKOO[i], &mut zobrist);
        std::mem::swap(&mut SQUARES[i], &mut squares);

        // The spot was empty, we are done
        if zobrist == Zobrist::ZERO {
            break;
        }

        // Find a new spot for the old bucket we displaced
        i = match zobrist.h1() {
            j if i == j => zobrist.h2(),
            j => j,
        };
    }
}

// Initializes the cuckoo tables.
#[cold]
pub(crate) unsafe fn init() {
    for color in Color::COLORS {
        for &piece in &Piece::PIECES[1..] {
            for from in Square::SQUARES {
                for &to in &Square::SQUARES[from.idx()+1..] {
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
#[inline(always)]
pub(crate) fn is_hash_of_legal_move(board: &Board, diff: Zobrist) -> bool {
    let mut i = diff.h1();
    unsafe {
        if CUCKOO[i] == diff || CUCKOO[{i = diff.h2(); i}] == diff {
            let (from, to) = SQUARES[i];
            board.is_path_clear(from, to)
        } else {
            false
        }
    }
}