/// Precalculated table of left shifts, such that
/// SHIFTS[i] == 1u64 << i for 0 <= i < 64
pub const SHIFTS: [u64; 64] = {
    let mut tab = [0; 64];
    let mut i = 0;
    while i < 64 {
        tab[i] = 1u64 << i;
        i += 1;
    }
    tab
};

/// The best available implementation of the _pext_u64 instruction
#[inline(always)]
pub fn pext(a: u64, mut mask: u64) -> u64 {
    if is_x86_feature_detected!("bmi2") {
        unsafe {std::arch::x86_64::_pext_u64(a, mask)}
    } else {
        let (mut i, mut res) = (0, 0);

        while mask != 0 {
            let tmp = mask;
            mask &= mask - 1;
            if (mask ^ tmp) & a != 0 {
                res |= SHIFTS[i];
            }
            i += 1;
        }

        res
    }
}

/// The best available implementation of the _pdep_u64 instruction
#[inline(always)]
pub fn pdep(a: u64, mut mask: u64) -> u64 {
    if is_x86_feature_detected!("bmi2") {
        unsafe {std::arch::x86_64::_pdep_u64(a, mask)}
    } else {
        let (mut i, mut res) = (0, 0);

        while mask != 0 {
            let tmp = mask;
            mask &= mask - 1;
            if a & SHIFTS[i] != 0 {
                res |= mask ^ tmp;
            }
            i += 1;
        }

        res
    }
}