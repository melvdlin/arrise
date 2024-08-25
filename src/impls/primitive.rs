use crate::impls::IllegalBitPattern;
use crate::{Deserialize, SerialSize, Serialize};
use core::char::CharTryFromError;
use core::mem::size_of;
use core::ptr::NonNull;

macro_rules! to_e_bytes {
    ($expr:expr) => {
        if cfg!(feature = "primitive_le") {
            $expr.to_le_bytes()
        } else if cfg!(feature = "primitive_be") {
            $expr.to_be_bytes()
        } else {
            $expr.to_ne_bytes()
        }
    };
}

macro_rules! from_e_bytes {
    ($ty:ty, $expr:expr) => {
        if cfg!(feature = "primitive_le") {
            <$ty>::from_le_bytes($expr)
        } else if cfg!(feature = "primitive_be") {
            <$ty>::from_be_bytes($expr)
        } else {
            <$ty>::from_ne_bytes($expr)
        }
    };
}

macro_rules! impl_primitive {
    ($ty:ty) => {
        impl crate::SerialSize for $ty {
            const SIZE: usize = core::mem::size_of::<$ty>();
        }

        impl crate::Serialize for $ty {
            fn serialize(&self, buffer: &mut [u8; <Self as crate::SerialSize>::SIZE]) {
                *buffer = to_e_bytes!(self);
            }
        }

        impl crate::Deserialize for $ty {
            type Error = core::convert::Infallible;

            unsafe fn deserialize_raw(
                into: NonNull<Self>,
                buffer: &[u8; <Self as crate::SerialSize>::SIZE],
            ) -> Result<(), <Self as crate::Deserialize>::Error> {
                Ok(into.write(from_e_bytes!($ty, *buffer)))
            }
        }
    };
}

macro_rules! impl_atomic {
    ($ty:ty, $primitive:ty, $size:expr) => {
        impl_atomic!($ty, $primitive, $size, cfg(all()));
    };

    ($ty:ty, $primitive:ty, $size:expr, $feature_gate:meta) => {
        #[$feature_gate]
        #[cfg(target_has_atomic_load_store = $size)]
        impl crate::SerialSize for $ty {
            const SIZE: usize = size_of::<$primitive>();
        }

        #[$feature_gate]
        #[cfg(target_has_atomic_load_store = $size)]
        impl crate::Serialize for $ty {
            fn serialize(&self, buffer: &mut [u8; <Self as crate::SerialSize>::SIZE]) {
                self.load(core::sync::atomic::Ordering::SeqCst)
                    .serialize(buffer)
            }
        }

        #[$feature_gate]
        #[cfg(target_has_atomic_load_store = $size)]
        impl crate::Deserialize for $ty {
            type Error = <$primitive as Deserialize>::Error;

            unsafe fn deserialize_raw(
                into: NonNull<Self>,
                buffer: &[u8; <Self as crate::SerialSize>::SIZE],
            ) -> Result<(), <Self as crate::Deserialize>::Error> {
                Ok(into.write(Self::new(<$primitive>::deserialize(buffer)?)))
            }
        }
    };
}
macro_rules! impl_nonzero {
    ($nonzero:ty, $primitive:ty) => {
        impl SerialSize for $nonzero {
            const SIZE: usize = size_of::<$primitive>();
        }

        impl Serialize for $nonzero {
            fn serialize(&self, buffer: &mut [u8; Self::SIZE]) {
                self.get().serialize(buffer)
            }
        }

        impl Deserialize for $nonzero {
            type Error = IllegalBitPattern;

            unsafe fn deserialize_raw(
                into: NonNull<Self>,
                buffer: &[u8; Self::SIZE],
            ) -> Result<(), Self::Error> {
                Ok(into.write(
                    Self::new(<$primitive>::deserialize(buffer).unwrap())
                        .ok_or(IllegalBitPattern)?,
                ))
            }
        }
    };
}

impl_primitive!(u8);
impl_primitive!(u16);
impl_primitive!(u32);
impl_primitive!(u64);
impl_primitive!(u128);
impl_primitive!(usize);
impl_primitive!(i8);
impl_primitive!(i16);
impl_primitive!(i32);
impl_primitive!(i64);
impl_primitive!(i128);
impl_primitive!(isize);
impl_primitive!(f32);
impl_primitive!(f64);

impl_atomic!(core::sync::atomic::AtomicBool, bool, "8");
impl_atomic!(core::sync::atomic::AtomicU8, u8, "8");
impl_atomic!(core::sync::atomic::AtomicU16, u16, "16");
impl_atomic!(core::sync::atomic::AtomicU32, u32, "32");
impl_atomic!(core::sync::atomic::AtomicU64, u64, "64");
impl_atomic!(
    core::sync::atomic::AtomicU128,
    u128,
    "128",
    cfg(feature = "atomic_int_128")
);
impl_atomic!(core::sync::atomic::AtomicUsize, usize, "ptr");
impl_atomic!(core::sync::atomic::AtomicI8, i8, "8");
impl_atomic!(core::sync::atomic::AtomicI16, i16, "16");
impl_atomic!(core::sync::atomic::AtomicI32, i32, "32");
impl_atomic!(core::sync::atomic::AtomicI64, i64, "64");
impl_atomic!(
    core::sync::atomic::AtomicI128,
    i128,
    "128",
    cfg(feature = "atomic_int_128")
);
impl_atomic!(core::sync::atomic::AtomicIsize, isize, "ptr");

impl_nonzero!(core::num::NonZeroI8, i8);
impl_nonzero!(core::num::NonZeroI16, i16);
impl_nonzero!(core::num::NonZeroI32, i32);
impl_nonzero!(core::num::NonZeroI64, i64);
impl_nonzero!(core::num::NonZeroI128, i128);
impl_nonzero!(core::num::NonZeroIsize, isize);
impl_nonzero!(core::num::NonZeroU8, u8);
impl_nonzero!(core::num::NonZeroU16, u16);
impl_nonzero!(core::num::NonZeroU32, u32);
impl_nonzero!(core::num::NonZeroU64, u64);
impl_nonzero!(core::num::NonZeroU128, u128);
impl_nonzero!(core::num::NonZeroUsize, usize);

impl SerialSize for bool {
    const SIZE: usize = <u8 as SerialSize>::SIZE;
}

impl Serialize for bool {
    fn serialize(&self, buffer: &mut [u8; <Self as SerialSize>::SIZE]) {
        *buffer = [*self as u8]
    }
}

impl Deserialize for bool {
    type Error = IllegalBitPattern;

    unsafe fn deserialize_raw(
        into: NonNull<Self>,
        buffer: &[u8; <Self as SerialSize>::SIZE],
    ) -> Result<(), Self::Error> {
        into.write(match *buffer {
            | [0] => false,
            | [1] => true,
            | _ => Err(IllegalBitPattern)?,
        });
        Ok(())
    }
}

impl SerialSize for char {
    const SIZE: usize = size_of::<u32>();
}

impl Serialize for char {
    fn serialize(&self, buffer: &mut [u8; <Self as SerialSize>::SIZE]) {
        u32::serialize(&u32::from(*self), buffer)
    }
}

impl Deserialize for char {
    type Error = CharTryFromError;

    unsafe fn deserialize_raw(
        into: NonNull<Self>,
        buffer: &[u8; <Self as SerialSize>::SIZE],
    ) -> Result<(), Self::Error> {
        into.write(char::try_from(u32::deserialize(buffer).unwrap())?);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    mod primitive {
        use crate::assert_serial_eq;

        #[test]
        fn test_bool() {
            assert_serial_eq!(bool, &true);
            assert_serial_eq!(bool, &false);
        }

        #[test]
        fn test_u8() {
            assert_serial_eq!(u8, &0);
            assert_serial_eq!(u8, &u8::MIN);
            assert_serial_eq!(u8, &u8::MAX);

            assert_serial_eq!(u8, &1);
            assert_serial_eq!(u8, &0x34);
            assert_serial_eq!(u8, &0x43);
        }

        #[test]
        fn test_i8() {
            assert_serial_eq!(i8, &0);
            assert_serial_eq!(i8, &i8::MIN);
            assert_serial_eq!(i8, &i8::MAX);

            assert_serial_eq!(i8, &1);
            assert_serial_eq!(i8, &-1);
            assert_serial_eq!(i8, &0x34);
            assert_serial_eq!(i8, &-0x34);
            assert_serial_eq!(i8, &0x43);
            assert_serial_eq!(i8, &-0x43);
        }

        #[test]
        fn test_u16() {
            assert_serial_eq!(u16, &0);
            assert_serial_eq!(u16, &u16::MIN);
            assert_serial_eq!(u16, &u16::MAX);

            assert_serial_eq!(u16, &1);
            assert_serial_eq!(u16, &0x1234);
            assert_serial_eq!(u16, &0x4321);
        }

        #[test]
        fn test_i16() {
            assert_serial_eq!(i16, &0);
            assert_serial_eq!(i16, &i16::MIN);
            assert_serial_eq!(i16, &i16::MAX);

            assert_serial_eq!(i16, &1);
            assert_serial_eq!(i16, &-1);
            assert_serial_eq!(i16, &0x1234);
            assert_serial_eq!(i16, &-0x1234);
            assert_serial_eq!(i16, &0x4321);
            assert_serial_eq!(i16, &-0x4321);
        }

        #[test]
        fn test_u32() {
            assert_serial_eq!(u32, &0);
            assert_serial_eq!(u32, &u32::MIN);
            assert_serial_eq!(u32, &u32::MAX);

            assert_serial_eq!(u32, &1);
            assert_serial_eq!(u32, &0x12345678);
            assert_serial_eq!(u32, &0x87654321);
        }

        #[test]
        fn test_i32() {
            assert_serial_eq!(i32, &0);
            assert_serial_eq!(i32, &i32::MIN);
            assert_serial_eq!(i32, &i32::MAX);

            assert_serial_eq!(i32, &1);
            assert_serial_eq!(i32, &-1);
            assert_serial_eq!(i32, &0x12345678);
            assert_serial_eq!(i32, &-0x12345678);
            assert_serial_eq!(i32, &0x43218765);
            assert_serial_eq!(i32, &-0x43218765);
        }

        #[test]
        fn test_u64() {
            assert_serial_eq!(u64, &0);
            assert_serial_eq!(u64, &u64::MIN);
            assert_serial_eq!(u64, &u64::MAX);

            assert_serial_eq!(u64, &1);
            assert_serial_eq!(u64, &0x123456789ABCDEF8);
            assert_serial_eq!(u64, &0x8FEDCBA987654321);
        }

        #[test]
        fn test_i64() {
            assert_serial_eq!(i64, &0);
            assert_serial_eq!(i64, &i64::MIN);
            assert_serial_eq!(i64, &i64::MAX);

            assert_serial_eq!(i64, &1);
            assert_serial_eq!(i64, &-1);
            assert_serial_eq!(i64, &0x123456789ABCDEF8);
            assert_serial_eq!(i64, &-0x123456789ABCDEF8);
            assert_serial_eq!(i64, &0x43218FEDCBA98765);
            assert_serial_eq!(i64, &-0x43218FEDCBA98765);
        }

        #[test]
        fn test_u128() {
            assert_serial_eq!(u128, &0);
            assert_serial_eq!(u128, &u128::MIN);
            assert_serial_eq!(u128, &u128::MAX);

            assert_serial_eq!(u128, &1);
            assert_serial_eq!(u128, &0x123456789ABCDEF88FEDCBA987654321);
            assert_serial_eq!(u128, &0x8FEDCBA987654321123456789ABCDEF8);
        }

        #[test]
        fn test_i128() {
            assert_serial_eq!(i128, &0);
            assert_serial_eq!(i128, &i128::MIN);
            assert_serial_eq!(i128, &i128::MAX);

            assert_serial_eq!(i128, &1);
            assert_serial_eq!(i128, &-1);
            assert_serial_eq!(i128, &0x123456789ABCDEF88FEDCBA987654321);
            assert_serial_eq!(i128, &-0x123456789ABCDEF88FEDCBA987654321);
            assert_serial_eq!(i128, &0x43218FEDCBA9876556789ABCDEF81234);
            assert_serial_eq!(i128, &-0x43218FEDCBA9876556789ABCDEF81234);
        }

        #[test]
        fn test_usize() {
            assert_serial_eq!(usize, &0);
            assert_serial_eq!(usize, &usize::MIN);
            assert_serial_eq!(usize, &usize::MAX);

            assert_serial_eq!(usize, &1);
            assert_serial_eq!(usize, &(0x123456789ABCDEF88FEDCBA987654321u128 as usize));
            assert_serial_eq!(usize, &(0x8FEDCBA987654321123456789ABCDEF8u128 as usize));
        }

        #[test]
        fn test_isize() {
            assert_serial_eq!(isize, &0);
            assert_serial_eq!(isize, &isize::MIN);
            assert_serial_eq!(isize, &isize::MAX);

            assert_serial_eq!(isize, &1);
            assert_serial_eq!(isize, &-1);
            assert_serial_eq!(isize, &(0x123456789ABCDEF88FEDCBA987654321i128 as isize));
            assert_serial_eq!(isize, &(-0x123456789ABCDEF88FEDCBA987654321i128 as isize));
            assert_serial_eq!(isize, &(0x43218FEDCBA9876556789ABCDEF81234i128 as isize));
            assert_serial_eq!(isize, &(-0x43218FEDCBA9876556789ABCDEF81234i128 as isize));
        }
    }

    mod nonzero {}

    mod atomic {}
}
