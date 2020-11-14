// That type may feel useless, and it kind of is, but
// it allows conveniently changing the width used by
// a ply counter. Plus, it makes a few signatures a
// bit more clear.

/// Represent a ply (half-turn) counter
#[derive(Copy, Clone, PartialEq, Default, Debug)]
pub struct Ply(u32);

impl Ply {
    /// Increment the counter
    #[inline(always)]
    pub fn incr(&mut self) {
        self.0 += 1;
    }

     /// Decrement the counter
    #[inline(always)]
    pub fn decr(&mut self) {
        self.0 -= 1;
    }
}