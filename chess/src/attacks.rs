mod slider {
    use crate::bitboard::BitBoard;
    use crate::square::Square;

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
                    bb |= Square::from((sq + (d-1) * dir[i].0) as u8).into();
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
                        let bb3 = Square::from((sq + d * dir[j].0) as u8).into();
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

    #[cold]
    pub(super) unsafe fn init() {
        let i = init_bmi2(&mut BISHOP_BMI2, &BISHOP_DIR, 0);
        init_bmi2(&mut ROOK_BMI2, &ROOK_DIR, i);
    }
}

#[cold]
pub(crate) unsafe fn init() {
    slider::init();
}