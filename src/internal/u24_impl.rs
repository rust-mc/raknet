use core::ops::Not;
use std::fmt;
use std::fmt::{write, Binary, Display, Formatter};

macro_rules! arithmetic_impl {
    ($tr:tt | $tr_assign:tt, $f:tt | $f_assign:tt, $ex:tt) => {
	    use core::ops::{$tr, $tr_assign};

        impl $tr<u24> for u24 {
            type Output = u24;

            fn $f(self, rhs: u24) -> Self::Output {
                (self.0 $ex rhs.0).into()
            }
        }

	    impl $tr<&u24> for u24 {
			type Output = <u24 as $tr<u24>>::Output;

			fn $f(self, rhs: &u24) -> Self::Output {
				(self.0 $ex rhs.0).into()
			}
		}

	    impl $tr<&u24> for &u24 {
			type Output = <u24 as $tr<u24>>::Output;

			fn $f(self, rhs: &u24) -> Self::Output {
				(self.0 $ex rhs.0).into()
			}
		}

	    impl<'a> $tr<u24> for &'a u24 {
			type Output = <u24 as $tr<u24>>::Output;

			fn $f(self, rhs: u24) -> Self::Output {
				(self.0 $ex rhs.0).into()
			}
		}

	    impl $tr_assign<u24> for u24 {
			fn $f_assign(&mut self, rhs: u24) {
				*self = *self $ex rhs
			}
		}

		impl $tr_assign<&u24> for u24 {
			fn $f_assign(&mut self, rhs: &u24) {
				*self = *self $ex rhs
			}
		}
    };
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct u24(u32);

impl u24 {
    /// The smallest value that can be represented by this integer type.
    pub const MIN: Self = Self { 0: 0 };
    /// The largest value that can be represented by this integer type (2^24 − 1)
    pub const MAX: Self = Self { 0: 0x00FFFFFFu32 };

    /// Reverses the byte order of the integer.
    pub const fn swap_bytes(self) -> Self {
        let bytes = self.0.to_be_bytes();
        Self {
            0: u32::from_be_bytes([0x00, bytes[3], bytes[2], bytes[1]]),
        }
    }

    /// Converts an integer from big endian to the target’s endianness.
    ///
    /// On big endian this is a no-op. On little endian the bytes are swapped.
    pub const fn from_be(x: Self) -> Self {
        if cfg!(target_endian = "big") {
            x
        } else {
            x.swap_bytes()
        }
    }

    /// Converts an integer from little endian to the target’s endianness.
    ///
    /// On little endian this is a no-op. On big endian the bytes are swapped.
    pub const fn from_le(x: Self) -> Self {
        if cfg!(target_endian = "little") {
            x
        } else {
            x.swap_bytes()
        }
    }

    /// Converts `self` to big endian from the target’s endianness.
    ///
    /// On big endian this is a no-op. On little endian the bytes are swapped.
    pub const fn to_be(self) -> Self {
        u24::from_be(self)
    }

    /// Converts self to little endian from the target’s endianness.
    ///
    /// On little endian this is a no-op. On big endian the bytes are swapped.
    pub const fn to_le(self) -> Self {
        u24::from_le(self)
    }

    /// Create a native endian integer value from its representation as a byte array in big endian.
    pub const fn from_be_bytes(bytes: [u8; 3]) -> Self {
        Self {
            0: u32::from_be_bytes([0x00, bytes[0], bytes[1], bytes[2]]),
        }
    }

    /// Create a native endian integer value from its representation as a byte array in little endian.
    pub const fn from_le_bytes(bytes: [u8; 3]) -> Self {
        Self {
            0: u32::from_le_bytes([bytes[0], bytes[1], bytes[2], 0x00]),
        }
    }

    /// Return the memory representation of this integer as a byte array in big-endian (network) byte order.
    pub const fn to_be_bytes(self) -> [u8; 3] {
        let bytes = self.0.to_be_bytes();
        [bytes[1], bytes[2], bytes[3]]
    }

    /// Return the memory representation of this integer as a byte array in little-endian byte order.
    pub const fn to_le_bytes(self) -> [u8; 3] {
        let bytes = self.0.to_be_bytes();
        [bytes[3], bytes[2], bytes[1]]
    }

    /// Consumes this `u24`, returning the underlying value.
    pub const fn into_inner(self) -> u32 {
        self.0
    }
}

impl From<u32> for u24 {
    fn from(value: u32) -> Self {
        Self {
            0: value & 0xffffff,
        }
    }
}

impl From<u24> for u32 {
    fn from(value: u24) -> Self {
        value.0
    }
}

impl PartialEq<u32> for u24 {
    fn eq(&self, other: &u32) -> bool {
        self.0 == *other
    }

    fn ne(&self, other: &u32) -> bool {
        self.0 != *other
    }
}

impl PartialEq<u24> for u32 {
    fn eq(&self, other: &u24) -> bool {
        *self == other.0
    }

    fn ne(&self, other: &u24) -> bool {
        *self != other.0
    }
}

arithmetic_impl!(Add | AddAssign, add | add_assign, +);
arithmetic_impl!(BitAnd | BitAndAssign, bitand | bitand_assign, &);
arithmetic_impl!(BitOr | BitOrAssign, bitor | bitor_assign, |);
arithmetic_impl!(BitXor | BitXorAssign, bitxor | bitxor_assign, ^);
arithmetic_impl!(Div | DivAssign, div | div_assign, /);
arithmetic_impl!(Mul | MulAssign, mul | mul_assign, *);
arithmetic_impl!(Rem | RemAssign, rem | rem_assign, %);
arithmetic_impl!(Sub | SubAssign, sub | sub_assign, -);

impl Binary for u24 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let val = self.0;
        fmt::Binary::fmt(&val, f)
    }
}

impl Display for u24 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Not for u24 {
    type Output = u24;

    fn not(self) -> Self::Output {
        (!self.0).into()
    }
}

impl Not for &u24 {
    type Output = <u24 as Not>::Output;

    fn not(self) -> Self::Output {
        (!self.0).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from() {
        let n = u24::from(0x12345678u32);
        assert_eq!(n, 0x00345678u32);
    }

    #[test]
    fn swap_bytes() {
        let n = u24::from(0x123456u32);
        let n = n.swap_bytes();
        assert_eq!(n, 0x563412u32);
    }

    #[test]
    fn from_be() {
        let n = u24::from(0x123456u32);

        if cfg!(target_endian = "big") {
            assert_eq!(u24::from_be(n), n);
        } else {
            assert_eq!(u24::from_be(n), n.swap_bytes());
        }
    }

    #[test]
    fn from_le() {
        let n = u24::from(0x123456u32);

        if cfg!(target_endian = "little") {
            assert_eq!(u24::from_le(n), n);
        } else {
            assert_eq!(u24::from_le(n), n.swap_bytes());
        }
    }

    #[test]
    fn to_be() {
        let n = u24::from(0x123456u32);

        if cfg!(target_endian = "big") {
            assert_eq!(n.to_be(), n)
        } else {
            assert_eq!(n.to_be(), n.swap_bytes())
        }
    }

    #[test]
    fn to_le() {
        let n = u24::from(0x123456u32);

        if cfg!(target_endian = "little") {
            assert_eq!(n.to_le(), n)
        } else {
            assert_eq!(n.to_le(), n.swap_bytes())
        }
    }

    #[test]
    fn from_be_bytes() {
        let n = u24::from_be_bytes([0x12, 0x34, 0x56]);
        assert_eq!(n, 0x123456u32)
    }

    #[test]
    fn from_le_bytes() {
        let n = u24::from_le_bytes([0x56, 0x34, 0x12]);
        assert_eq!(n, 0x123456u32)
    }

    #[test]
    fn to_be_bytes() {
        let bytes = u24::from(0x123456u32).to_be_bytes();
        assert_eq!(bytes, [0x12, 0x34, 0x56])
    }

    #[test]
    fn to_le_bytes() {
        let bytes = u24::from(0x123456u32).to_le_bytes();
        assert_eq!(bytes, [0x56, 0x34, 0x12])
    }

    #[test]
    fn add_overflow() {
        let n = u24::MAX;
        let m = u24::from(1);
        assert_eq!(n + m, 0);
    }

    #[test]
    fn add_assign_overflow() {
        let mut n = u24::MAX;
        let m = u24::from(1);
        n += m;
        assert_eq!(n, 0);
    }

    #[test]
    fn fmt_binary() {
        let n = u24::from(42);

        assert_eq!(format!("{:b}", n), "101010");
        assert_eq!(format!("{:#b}", n), "0b101010");
    }
}
