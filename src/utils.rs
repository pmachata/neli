use std::ops::{AddAssign, BitOr, Deref, Sub, SubAssign};

/// Struct representing a single bit flag
pub struct U32BitFlag(u32);

impl U32BitFlag {
    pub fn new(bit_num: u32) -> Self {
        U32BitFlag(bit_num)
    }

    fn into_bitmask(self) -> U32Bitmask {
        U32Bitmask::from(num_to_set_mask(self.0))
    }
}

/// Struct for handling `u32` bitmask operations
pub struct U32Bitmask(u32);

impl U32Bitmask {
    pub fn empty() -> Self {
        U32Bitmask(0)
    }

    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    pub fn is_set(&self, bit: u32) -> bool {
        let set_mask = num_to_set_mask(bit);
        set_mask & self.0 == set_mask
    }
}

impl BitOr<U32Bitmask> for U32Bitmask {
    type Output = U32Bitmask;

    fn bitor(self, rhs: U32Bitmask) -> Self::Output {
        U32Bitmask::from(self.0 | *rhs)
    }
}

impl BitOr<U32BitFlag> for U32Bitmask {
    type Output = U32Bitmask;

    fn bitor(self, rhs: U32BitFlag) -> Self::Output {
        self | rhs.into_bitmask()
    }
}

impl AddAssign<U32BitFlag> for U32Bitmask {
    fn add_assign(&mut self, rhs: U32BitFlag) {
        self.0 |= *U32Bitmask::from(rhs)
    }
}

impl<'a> AddAssign<U32BitFlag> for &'a mut U32Bitmask {
    fn add_assign(&mut self, rhs: U32BitFlag) {
        self.0 |= *U32Bitmask::from(rhs)
    }
}

impl Sub<U32Bitmask> for U32Bitmask {
    type Output = U32Bitmask;

    fn sub(self, rhs: U32Bitmask) -> Self::Output {
        U32Bitmask::from(self.0 & !*rhs)
    }
}

impl Sub<U32BitFlag> for U32Bitmask {
    type Output = U32Bitmask;

    fn sub(self, rhs: U32BitFlag) -> Self::Output {
        self - rhs.into_bitmask()
    }
}

impl SubAssign<U32BitFlag> for U32Bitmask {
    fn sub_assign(&mut self, rhs: U32BitFlag) {
        self.0 &= !*U32Bitmask::from(rhs)
    }
}

impl<'a> SubAssign<U32BitFlag> for &'a mut U32Bitmask {
    fn sub_assign(&mut self, rhs: U32BitFlag) {
        self.0 &= !*U32Bitmask::from(rhs)
    }
}

impl From<U32BitFlag> for U32Bitmask {
    fn from(v: U32BitFlag) -> Self {
        v.into_bitmask()
    }
}

impl From<u32> for U32Bitmask {
    fn from(v: u32) -> Self {
        U32Bitmask(v)
    }
}

impl Deref for U32Bitmask {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Conversion between a group number and the necessary bitmask
/// to perform a bitwise OR that will set the bit
#[inline]
fn num_to_set_mask(grp: u32) -> u32 {
    1 << (grp - 1)
}
