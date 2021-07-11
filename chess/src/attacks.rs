
use crate::bitboard::BitBoard;
use crate::color::Color;
use crate::square::Square;

//#################################################################################################
//
//                                sliders attacks tables
//
//#################################################################################################

// A struct containing the informations necessary for a bmi2 lookup.
#[derive(Debug)]
struct Bmi2Info {
    offset: usize,
    mask1: BitBoard,
    mask2: BitBoard,
}

impl Bmi2Info {
    // A default value for that particular struct.
    const ZERO: Bmi2Info = Bmi2Info {
        offset: 0, 
        mask1: BitBoard::EMPTY, 
        mask2: BitBoard::EMPTY
    };
}

// An array of 64 bmi2 infos, one for each square.
type Bmi2Array = [Bmi2Info; 64];

// The bmi2 infos associated with bishops and rooks, for every square on the board.
static mut BISHOP_BMI2: Bmi2Array = [Bmi2Info::ZERO; 64];
static mut ROOK_BMI2  : Bmi2Array = [Bmi2Info::ZERO; 64];

// The array that contains every attack pattern, indexed through bmi2 infos with pext and pdep.
static mut SLIDER_ATTACKS: [u16; 107648] = [0; 107648];

// For use with the 0x88 trick.
type Dirs = [(i32, i32); 4];
const BISHOP_DIR: Dirs = [
    (-9, -17), (-7, -15), (7, 15), (9, 17),
];
const ROOK_DIR: Dirs = [
    (-8, -16), (-1, -1), (1, 1), (8, 16),
];

// Generates the bmi2 infos for a certain piece, with given dirs.
// Uses some space in the SLIDER_ATTACKS array and return the index of the next
// available spot.
#[cold]
unsafe fn init_bmi2(info: &mut Bmi2Array, dirs: &Dirs, mut idx: usize) -> usize {
    let mut squares = Vec::new();

    for sq in 0..64 {
        info[sq as usize].offset = idx as usize;

        let sq88 = sq + (sq & !7);
        let mut bb = BitBoard::EMPTY;
        for dir in dirs {
            if (sq88 + dir.1) & 0x88 != 0 {
                continue;
            }

            let mut d = 2;
            while (sq88 + d * dir.1) & 0x88 == 0 {
                bb |= Square::from((sq + (d-1) * dir.0) as i8).into();
                d += 1;
            }
        }
        info[sq as usize].mask1 = bb;

        squares.clear();
        for sq in bb.iter_squares() {
            squares.push(sq);
        }

        for i in 0..(1 << squares.len()) {
            bb = BitBoard::EMPTY;

            for (j, &square) in squares.iter().enumerate() {
                if i & (1 << j) != 0 {
                    bb |= square.into();
                }
            }

            let mut bb2 = BitBoard::EMPTY;
            for dir in dirs {
                let mut d = 1;
                while (sq88 + d * dir.1) & 0x88 == 0 {
                    let bb3 = Square::from((sq + d * dir.0) as i8).into();
                    bb2 |= bb3;
                    if (bb & bb3).not_empty() {
                        break;
                    }
                    d += 1;
                }
            }

            if i == 0 {
                info[sq as usize].mask2 = bb2;
            }
            SLIDER_ATTACKS[idx] = (bb2.pext(info[sq as usize].mask2).0 & 0xFFFF) as u16;
            idx += 1;
        }
    }

    idx
}

//#################################################################################################
//
//                               jumpers attacks tables
//
//#################################################################################################

// King attacks.
static mut KING_ATTACKS: [BitBoard; 64] = [BitBoard::EMPTY; 64];

// Knight attacks.
static mut KNIGHT_ATTACKS: [BitBoard; 64] = [BitBoard::EMPTY; 64];

// Pawn attacks.
static mut WHITE_PAWN_ATTACKS: [BitBoard; 64] = [BitBoard::EMPTY; 64];
static mut BLACK_PAWN_ATTACKS: [BitBoard; 64] = [BitBoard::EMPTY; 64];

// Pawn pushes.
static mut WHITE_PAWN_PUSHES: [Option<Square>; 64] = [None; 64];
static mut BLACK_PAWN_PUSHES: [Option<Square>; 64] = [None; 64];

// Pawn double pushes.
static mut WHITE_PAWN_DOUBLE_PUSHES: [Option<Square>; 64] = [None; 64];
static mut BLACK_PAWN_DOUBLE_PUSHES: [Option<Square>; 64] = [None; 64];

//#################################################################################################
//
//                                          init
//
//#################################################################################################

// Turns an Option<Square> into a bitboard, an empty one if the Option is None.
#[cold]
fn to_bitboard(sq: Option<Square>) -> BitBoard {
    sq.map_or(BitBoard::EMPTY, |sq| sq.into())
}

// Initializes the sliders attacks and their bmi infos, then initialize the 
// jumpers attacks.
#[cold]
pub(crate) unsafe fn init() {
    // Slider attacks
    let idx = init_bmi2(&mut BISHOP_BMI2, &BISHOP_DIR, 0);
    init_bmi2(&mut ROOK_BMI2, &ROOK_DIR, idx);

    for sq in Square::SQUARES {
        // Kings attacks
        for dir in [
            (1, 1), (1, 0), (1, -1), (0, -1),
            (-1, -1), (-1, 0), (-1, 1), (0, 1),
        ] {
            KING_ATTACKS[usize::from(sq)] |= to_bitboard(sq.displace(dir));
        }

        // Knights attacks
        for dir in [
            (1, 2), (2, 1), (2, -1), (1, -2),
            (-1, -2), (-2, -1), (-2, 1), (-1, 2),
        ] {
            KNIGHT_ATTACKS[usize::from(sq)] |= to_bitboard(sq.displace(dir));
        }

        // Pawns attacks
        WHITE_PAWN_ATTACKS[usize::from(sq)] = to_bitboard(sq.displace((1, 1)))  | to_bitboard(sq.displace((-1, 1)));
        BLACK_PAWN_ATTACKS[usize::from(sq)] = to_bitboard(sq.displace((1, -1))) | to_bitboard(sq.displace((-1, -1)));

        // Pawn pushes
        WHITE_PAWN_PUSHES[usize::from(sq)] = sq.displace((0,  1));
        BLACK_PAWN_PUSHES[usize::from(sq)] = sq.displace((0, -1));

        // Pawn double pushes
        match sq.y() {
            1 => WHITE_PAWN_DOUBLE_PUSHES[usize::from(sq)] = sq.displace((0,  2)),
            6 => BLACK_PAWN_DOUBLE_PUSHES[usize::from(sq)] = sq.displace((0, -2)),
            _ => (),
        }
    }
}

//#################################################################################################
//
//                                          accessers
//
//#################################################################################################

// Returns the attacks BitBoard of a Pawn of Color color located on square sq with Board occupancy occ.
#[inline]
pub(crate) fn pawn(color: Color, sq: Square) -> BitBoard {
    unsafe {
        match color {
            Color::White => WHITE_PAWN_ATTACKS[usize::from(sq)],
            Color::Black => BLACK_PAWN_ATTACKS[usize::from(sq)],
        }
    }
}

// Returns the square, if available, of the position the pawn
// would occupy if it was pushed.
#[inline]
pub(crate) fn pawn_push(color: Color, sq: Square) -> Option<Square> {
    unsafe {
        match color {
            Color::White => WHITE_PAWN_PUSHES[usize::from(sq)],
            Color::Black => BLACK_PAWN_PUSHES[usize::from(sq)],
        }
    }
}

// Returns the square, if available, of the position the pawn
// would occupy if it was double pushed.
#[inline]
pub(crate) fn pawn_double_push(color: Color, sq: Square) -> Option<Square> {
    unsafe {
        match color {
            Color::White => WHITE_PAWN_DOUBLE_PUSHES[usize::from(sq)],
            Color::Black => BLACK_PAWN_DOUBLE_PUSHES[usize::from(sq)],
        }
    }
}

// Returns the attacks BitBoard of a Rook located on square sq, with Board occupancy occ.
#[inline]
pub(crate) fn rook(sq: Square, occ: BitBoard) -> BitBoard {
    unsafe {
        let info = &ROOK_BMI2[usize::from(sq)];
        let mask = SLIDER_ATTACKS[info.offset + occ.pext(info.mask1).0 as usize];
        BitBoard(mask as u64).pdep(info.mask2)
    }
}

// Returns the attacks BitBoard of a Knight located on square sq.
#[inline]
pub(crate) fn knight(sq: Square) -> BitBoard {
    unsafe {
        KNIGHT_ATTACKS[usize::from(sq)]
    }
}

// Returns the attacks BitBoard of a Bishop located on square sq, with Board occupancy occ.
#[inline]
pub(crate) fn bishop(sq: Square, occ: BitBoard) -> BitBoard {
    unsafe {
        let info = &BISHOP_BMI2[usize::from(sq)];
        let mask = SLIDER_ATTACKS[info.offset + occ.pext(info.mask1).0 as usize];
        BitBoard(mask as u64).pdep(info.mask2)
    }
}

// Returns the attacks BitBoard of a Queen located on square sq, with Board occupancy occ.
#[inline]
pub(crate) fn queen(sq: Square, occ: BitBoard) -> BitBoard {
    bishop(sq, occ) | rook(sq, occ)
}

// Returns the attacks BitBoard of a King located on square sq.
#[inline]
pub(crate) fn king(sq: Square) -> BitBoard {
    unsafe {
        KING_ATTACKS[usize::from(sq)]
    }
}