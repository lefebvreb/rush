
use crate::bitboard::BitBoard;
use crate::color::Color;
use crate::square::Square;

//#################################################################################################
//
//                                sliders attacks tables
//
//#################################################################################################

struct BMI2Info {
    offset: usize,
    mask1: BitBoard,
    mask2: BitBoard,
}

impl BMI2Info {
    const ZERO: BMI2Info = BMI2Info {
        offset: 0, 
        mask1: BitBoard::EMPTY, 
        mask2: BitBoard::EMPTY
    };
}

type BMI2Array = [BMI2Info; 64];

static mut BISHOP_BMI2: BMI2Array = [BMI2Info::ZERO; 64];
static mut ROOK_BMI2  : BMI2Array = [BMI2Info::ZERO; 64];

static mut SLIDER_ATTACKS: [u16; 107648] = [0; 107648];

type DIR = [(i32, i32); 4];

const BISHOP_DIR: DIR = [
    (-9, -17), (-7, -15), (7, 15), (9, 17),
];

const ROOK_DIR: DIR = [
    (-8, -16), (-1, -1), (1, 1), (8, 16),
];

unsafe fn init_bmi2(info: &mut BMI2Array, dir: &DIR, mut idx: usize) -> usize {
    for sq in 0..64 {
        info[sq as usize].offset = idx as usize;

        let sq88 = sq + (sq & !7);
        let mut bb = BitBoard::EMPTY;
        for i in 0..4 {
            if (sq88 + dir[i].1) & 0x88 != 0 {
                continue;
            }

            let mut d = 2;
            while (sq88 + d * dir[i].1) & 0x88 == 0 {
                bb |= BitBoard::from((sq + (d-1) * dir[i].0) as u8);
                d += 1;
            }
        }
        info[sq as usize].mask1 = bb;

        let squares: Vec<_> = bb.iter_squares().collect();
        let n = squares.len();

        for i in 0..(1 << n) {
            bb = BitBoard::EMPTY;

            for j in 0..n {
                if i & (1 << j) != 0 {
                    bb |= squares[j].into();
                }
            }

            let mut bb2 = BitBoard::EMPTY;
            for j in 0..4 {
                let mut d = 1;
                while (sq88 + d * dir[j].1) & 0x88 == 0 {
                    let bb3 = BitBoard::from((sq + d * dir[j].0) as u8);
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
            SLIDER_ATTACKS[idx] = bb2.pext(info[sq as usize].mask2).lower16();
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

static mut KING_ATTACKS: [BitBoard; 64] = [BitBoard::EMPTY; 64];
static mut KNIGHT_ATTACKS: [BitBoard; 64] = [BitBoard::EMPTY; 64];

static mut WHITE_PAWN_ATTACKS: [BitBoard; 64] = [BitBoard::EMPTY; 64];
static mut BLACK_PAWN_ATTACKS: [BitBoard; 64] = [BitBoard::EMPTY; 64];

static mut WHITE_PAWN_PUSHES: [Option<Square>; 64] = [None; 64];
static mut BLACK_PAWN_PUSHES: [Option<Square>; 64] = [None; 64];
static mut WHITE_PAWN_DOUBLE_PUSHES: [Option<Square>; 64] = [None; 64];
static mut BLACK_PAWN_DOUBLE_PUSHES: [Option<Square>; 64] = [None; 64];

//#################################################################################################
//
//                                          init
//
//#################################################################################################

fn to_maybe_square(bb: BitBoard) -> Option<Square> {
    match bb.count() {
        0 => None,
        1 => Some(bb.as_square_unchecked()),
        _ => panic!(),
    }
}

#[cold]
pub(crate) unsafe fn init() {
    // Slider attacks
    let i = init_bmi2(&mut BISHOP_BMI2, &BISHOP_DIR, 0);
    init_bmi2(&mut ROOK_BMI2, &ROOK_DIR, i);

    for sq in Square::SQUARES {
        // Kings attacks
        for dir in [
            (1, 1), (1, 0), (1, -1), (0, -1),
            (-1, -1), (-1, 0), (-1, 1), (0, 1),
        ] {
            KING_ATTACKS[sq.idx()] |= sq.displace(dir);
        }

        // Knights attacks
        for dir in [
            (1, 2), (2, 1), (2, -1), (1, -2),
            (-1, -2), (-2, -1), (-2, 1), (-1, 2),
        ] {
            KNIGHT_ATTACKS[sq.idx()] |= sq.displace(dir);
        }

        // White pawns attacks
        WHITE_PAWN_ATTACKS[sq.idx()] |= sq.displace((1, 1)) | sq.displace((-1, 1));

        // White pawns attacks
        BLACK_PAWN_ATTACKS[sq.idx()] |= sq.displace((1, -1)) | sq.displace((-1, -1));

        // White pawns pushes
        WHITE_PAWN_PUSHES[sq.idx()] = to_maybe_square(sq.displace((0, 1)));

        // White pawns pushes
        BLACK_PAWN_PUSHES[sq.idx()] = to_maybe_square(sq.displace((0, -1)));

        // White pawns double pushes
        if sq.y() == 1 {
            WHITE_PAWN_DOUBLE_PUSHES[sq.idx()] = to_maybe_square(sq.displace((0, 2)));
        }

        // Black pawns double pushes
        if sq.y() == 6 {
            BLACK_PAWN_DOUBLE_PUSHES[sq.idx()] = to_maybe_square(sq.displace((0, -2)));
        }
    }
}

//#################################################################################################
//
//                                          accessers
//
//#################################################################################################

// Return the attacks BitBoard of a Pawn of Color color located on square sq with Board occupancy occ
#[inline(always)]
fn pawn_attacks(color: Color, sq: Square) -> BitBoard {
    unsafe {
        match color {
            Color::White => WHITE_PAWN_ATTACKS[sq.idx()],
            Color::Black => BLACK_PAWN_ATTACKS[sq.idx()],
        }
    }    
}

// Return the attacks BitBoard of a Rook located on square sq, with Board occupancy occ
#[inline(always)]
fn rook_attacks(sq: Square, occ: BitBoard) -> BitBoard {
    unsafe {
        let info = &ROOK_BMI2[sq.idx()];
        let mask = SLIDER_ATTACKS[info.offset + occ.pext(info.mask1).0 as usize];
        BitBoard(mask as u64).pdep(info.mask2)
    }
}

// Return the attacks BitBoard of a Knight located on square sq
#[inline(always)]
fn knight_attacks(sq: Square) -> BitBoard {
    unsafe {
        KNIGHT_ATTACKS[sq.idx()]
    }
}

// Return the attacks BitBoard of a Bishop located on square sq, with Board occupancy occ
#[inline(always)]
fn bishop_attacks(sq: Square, occ: BitBoard) -> BitBoard {
    unsafe {
        let info = &BISHOP_BMI2[sq.idx()];
        let mask = SLIDER_ATTACKS[info.offset + occ.pext(info.mask1).0 as usize];
        BitBoard(mask as u64).pdep(info.mask2)
    }
}

// Return the attacks BitBoard of a Queen located on square sq, with Board occupancy occ
#[inline(always)]
fn queen_attacks(sq: Square, occ: BitBoard) -> BitBoard {
    bishop_attacks(sq, occ) | rook_attacks(sq, occ)
}

// Return the attacks BitBoard of a King located on square sq
#[inline(always)]
fn king_attacks(sq: Square) -> BitBoard {
    unsafe {
        KING_ATTACKS[sq.idx()]
    }
}