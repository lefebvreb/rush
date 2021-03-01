// Precalculated table of left shifts, such that
// SHIFTS[i] == 1u64 << i for 0 <= i < 64
pub(crate) const SHIFTS: [u64; 64] = [
    0x1, 0x2, 0x4, 0x8, 0x10, 0x20, 0x40, 0x80,
    0x100, 0x200, 0x400, 0x800, 0x1000, 0x2000, 0x4000, 0x8000,
    0x10000, 0x20000, 0x40000, 0x80000, 0x100000, 0x200000, 0x400000, 0x800000,
    0x1000000, 0x2000000, 0x4000000, 0x8000000, 0x10000000, 0x20000000, 0x40000000, 0x80000000,
    0x100000000, 0x200000000, 0x400000000, 0x800000000, 0x1000000000, 0x2000000000, 0x4000000000, 0x8000000000,
    0x10000000000, 0x20000000000, 0x40000000000, 0x80000000000, 0x100000000000, 0x200000000000, 0x400000000000, 0x800000000000,
    0x1000000000000, 0x2000000000000, 0x4000000000000, 0x8000000000000, 0x10000000000000, 0x20000000000000, 0x40000000000000, 0x80000000000000,
    0x100000000000000, 0x200000000000000, 0x400000000000000, 0x800000000000000, 0x1000000000000000, 0x2000000000000000, 0x4000000000000000, 0x8000000000000000,
];

// Perform a parallel bits extract (pext) using the intrinsic (fast)
#[inline(always)]
#[cfg(target_feature = "bmi2")]
pub(crate) fn pext(a: u64, mask: u64) -> u64 {
    unsafe {std::arch::x86_64::_pext_u64(a, mask)}
}

// Perform a parallel bits extract (pext) without the intrinsic (slow)
#[inline(always)]
#[cfg(not(target_feature = "bmi2"))]
pub(crate) fn pext(a: u64, mut mask: u64) -> u64 {
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

// Perform a parallel bits deposit (pdep) using the intrinsic (fast)
#[inline(always)]
#[cfg(target_feature = "bmi2")]
pub(crate) fn pdep(a: u64, mask: u64) -> u64 {
    unsafe {std::arch::x86_64::_pdep_u64(a, mask)}
}

// Perform a parallel bits deposit (pdep) without the intrinsic (slow)
#[inline(always)]
#[cfg(not(target_feature = "bmi2"))]
pub(crate) fn pdep(a: u64, mut mask: u64) -> u64 {
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