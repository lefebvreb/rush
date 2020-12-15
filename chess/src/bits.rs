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


#[inline(always)]
#[cfg(target_feature = "bmi2")]
pub fn pext(a: u64, mask: u64) -> u64 {
    unsafe {std::arch::x86_64::_pext_u64(a, mask)}
}

/// Performs a parallel bits extract (pext)
#[inline(always)]
#[cfg(not(target_feature = "bmi2"))]
pub fn pext(a: u64, mut mask: u64) -> u64 {
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

#[inline(always)]
#[cfg(target_feature = "bmi2")]
pub fn pdep(a: u64, mask: u64) -> u64 {
    unsafe {std::arch::x86_64::_pdep_u64(a, mask)}
}

#[inline(always)]
#[cfg(not(target_feature = "bmi2"))]
pub fn pdep(a: u64, mut mask: u64) -> u64 {
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