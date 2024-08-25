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
        use crate::{assert_serial_eq, Deserialize, SerialSize, Serialize};

        #[test]
        fn test_bool() {
            assert_serial_eq!(bool, &false);
            assert_serial_eq!(bool, &true);
        }

        #[test]
        fn test_u8() {
            type T = u8;

            assert_serial_eq!(T, &0);
            assert_serial_eq!(T, &T::MIN);
            assert_serial_eq!(T, &T::MAX);

            assert_serial_eq!(T, &1);
            assert_serial_eq!(T, &0x34);
            assert_serial_eq!(T, &0x43);
        }

        #[test]
        fn test_i8() {
            type T = i8;

            assert_serial_eq!(T, &0);
            assert_serial_eq!(T, &T::MIN);
            assert_serial_eq!(T, &T::MAX);

            assert_serial_eq!(T, &1);
            assert_serial_eq!(T, &-1);
            assert_serial_eq!(T, &0x34);
            assert_serial_eq!(T, &-0x34);
            assert_serial_eq!(T, &0x43);
            assert_serial_eq!(T, &-0x43);
        }

        #[test]
        fn test_u16() {
            type T = u16;

            assert_serial_eq!(T, &0);
            assert_serial_eq!(T, &T::MIN);
            assert_serial_eq!(T, &T::MAX);

            assert_serial_eq!(T, &1);
            assert_serial_eq!(T, &0x1234);
            assert_serial_eq!(T, &0x4321);
        }

        #[test]
        fn test_i16() {
            type T = i16;

            assert_serial_eq!(T, &0);
            assert_serial_eq!(T, &T::MIN);
            assert_serial_eq!(T, &T::MAX);

            assert_serial_eq!(T, &1);
            assert_serial_eq!(T, &-1);
            assert_serial_eq!(T, &0x1234);
            assert_serial_eq!(T, &-0x1234);
            assert_serial_eq!(T, &0x4321);
            assert_serial_eq!(T, &-0x4321);
        }

        #[test]
        fn test_u32() {
            type T = u32;

            assert_serial_eq!(T, &0);
            assert_serial_eq!(T, &T::MIN);
            assert_serial_eq!(T, &T::MAX);

            assert_serial_eq!(T, &1);
            assert_serial_eq!(T, &0x12345678);
            assert_serial_eq!(T, &0x87654321);
        }

        #[test]
        fn test_i32() {
            type T = i32;

            assert_serial_eq!(T, &0);
            assert_serial_eq!(T, &T::MIN);
            assert_serial_eq!(T, &T::MAX);

            assert_serial_eq!(T, &1);
            assert_serial_eq!(T, &-1);
            assert_serial_eq!(T, &0x12345678);
            assert_serial_eq!(T, &-0x12345678);
            assert_serial_eq!(T, &0x43218765);
            assert_serial_eq!(T, &-0x43218765);
        }

        #[test]
        fn test_u64() {
            type T = u64;

            assert_serial_eq!(T, &0);
            assert_serial_eq!(T, &T::MIN);
            assert_serial_eq!(T, &T::MAX);

            assert_serial_eq!(T, &1);
            assert_serial_eq!(T, &0x123456789ABCDEF8);
            assert_serial_eq!(T, &0x8FEDCBA987654321);
        }

        #[test]
        fn test_i64() {
            type T = i64;

            assert_serial_eq!(T, &0);
            assert_serial_eq!(T, &T::MIN);
            assert_serial_eq!(T, &T::MAX);

            assert_serial_eq!(T, &1);
            assert_serial_eq!(T, &-1);
            assert_serial_eq!(T, &0x123456789ABCDEF8);
            assert_serial_eq!(T, &-0x123456789ABCDEF8);
            assert_serial_eq!(T, &0x43218FEDCBA98765);
            assert_serial_eq!(T, &-0x43218FEDCBA98765);
        }

        #[test]
        fn test_u128() {
            type T = u128;

            assert_serial_eq!(T, &0);
            assert_serial_eq!(T, &T::MIN);
            assert_serial_eq!(T, &T::MAX);

            assert_serial_eq!(T, &1);
            assert_serial_eq!(T, &0x123456789ABCDEF88FEDCBA987654321);
            assert_serial_eq!(T, &0x8FEDCBA987654321123456789ABCDEF8);
        }

        #[test]
        fn test_i128() {
            type T = i128;

            assert_serial_eq!(T, &0);
            assert_serial_eq!(T, &T::MIN);
            assert_serial_eq!(T, &T::MAX);

            assert_serial_eq!(T, &1);
            assert_serial_eq!(T, &-1);
            assert_serial_eq!(T, &0x123456789ABCDEF88FEDCBA987654321);
            assert_serial_eq!(T, &-0x123456789ABCDEF88FEDCBA987654321);
            assert_serial_eq!(T, &0x43218FEDCBA9876556789ABCDEF81234);
            assert_serial_eq!(T, &-0x43218FEDCBA9876556789ABCDEF81234);
        }

        #[test]
        fn test_usize() {
            type T = usize;

            assert_serial_eq!(T, &0);
            assert_serial_eq!(T, &T::MIN);
            assert_serial_eq!(T, &T::MAX);

            assert_serial_eq!(T, &1);
            assert_serial_eq!(T, &(0x123456789ABCDEF88FEDCBA987654321u128 as T));
            assert_serial_eq!(T, &(0x8FEDCBA987654321123456789ABCDEF8u128 as T));
        }

        #[test]
        fn test_isize() {
            type T = isize;

            assert_serial_eq!(T, &0);
            assert_serial_eq!(T, &T::MIN);
            assert_serial_eq!(T, &T::MAX);

            assert_serial_eq!(T, &1);
            assert_serial_eq!(T, &-1);
            assert_serial_eq!(T, &(0x123456789ABCDEF88FEDCBA987654321i128 as T));
            assert_serial_eq!(T, &(-0x123456789ABCDEF88FEDCBA987654321i128 as T));
            assert_serial_eq!(T, &(0x43218FEDCBA9876556789ABCDEF81234i128 as T));
            assert_serial_eq!(T, &(-0x43218FEDCBA9876556789ABCDEF81234i128 as T));
        }

        #[test]
        fn test_f32() {
            type T = f32;
            assert_serial_eq!(T, &0.0);
            assert_serial_eq!(T, &T::MIN);
            assert_serial_eq!(T, &T::MAX);
            assert_serial_eq!(T, &T::INFINITY);
            assert_serial_eq!(T, &T::NEG_INFINITY);
            assert_serial_eq!(T, &T::MIN_POSITIVE);

            assert_serial_eq!(T, &1.0);
            assert_serial_eq!(T, &-1.0);
            assert_serial_eq!(T, &543e21);
            assert_serial_eq!(T, &-543e21);
            assert_serial_eq!(T, &543e-21);
            assert_serial_eq!(T, &-543e-21);

            let mut buf = [0; <T as SerialSize>::SIZE];
            <T as Serialize>::serialize(&T::NAN, &mut buf);
            let de =
                <T as Deserialize>::deserialize(&buf).expect("deserialization failed");
            assert!(de.is_nan());
        }

        #[test]
        fn test_f64() {
            type T = f64;
            assert_serial_eq!(T, &0.0);
            assert_serial_eq!(T, &T::MIN);
            assert_serial_eq!(T, &T::MAX);
            assert_serial_eq!(T, &T::INFINITY);
            assert_serial_eq!(T, &T::NEG_INFINITY);
            assert_serial_eq!(T, &T::MIN_POSITIVE);

            assert_serial_eq!(T, &1.0);
            assert_serial_eq!(T, &-1.0);
            assert_serial_eq!(T, &543e21);
            assert_serial_eq!(T, &-543e21);
            assert_serial_eq!(T, &543e-21);
            assert_serial_eq!(T, &-543e-21);

            let mut buf = [0; <T as SerialSize>::SIZE];
            <T as Serialize>::serialize(&T::NAN, &mut buf);
            let de =
                <T as Deserialize>::deserialize(&buf).expect("deserialization failed");
            assert!(de.is_nan());
        }
    }

    mod nonzero {
        use crate::assert_serial_eq;
        use core::num::*;

        #[test]
        fn test_u8() {
            type T = NonZeroU8;

            assert_serial_eq!(T, &T::MIN);
            assert_serial_eq!(T, &T::MAX);

            assert_serial_eq!(T, &T::new(1).unwrap());
            assert_serial_eq!(T, &T::new(0x34).unwrap());
            assert_serial_eq!(T, &T::new(0x43).unwrap());
        }

        #[test]
        fn test_i8() {
            type T = NonZeroI8;

            assert_serial_eq!(T, &T::MIN);
            assert_serial_eq!(T, &T::MIN);

            assert_serial_eq!(T, &T::new(1).unwrap());
            assert_serial_eq!(T, &T::new(-1).unwrap());
            assert_serial_eq!(T, &T::new(0x34).unwrap());
            assert_serial_eq!(T, &T::new(-0x34).unwrap());
            assert_serial_eq!(T, &T::new(0x43).unwrap());
            assert_serial_eq!(T, &T::new(-0x43).unwrap());
        }

        #[test]
        fn test_u16() {
            type T = NonZeroU16;

            assert_serial_eq!(T, &T::MIN);
            assert_serial_eq!(T, &T::MAX);

            assert_serial_eq!(T, &T::new(1).unwrap());
            assert_serial_eq!(T, &T::new(0x1234).unwrap());
            assert_serial_eq!(T, &T::new(0x4321).unwrap());
        }

        #[test]
        fn test_i16() {
            type T = NonZeroI16;

            assert_serial_eq!(T, &T::MIN);
            assert_serial_eq!(T, &T::MAX);

            assert_serial_eq!(T, &T::new(1).unwrap());
            assert_serial_eq!(T, &T::new(-1).unwrap());
            assert_serial_eq!(T, &T::new(0x1234).unwrap());
            assert_serial_eq!(T, &T::new(-0x1234).unwrap());
            assert_serial_eq!(T, &T::new(0x4321).unwrap());
            assert_serial_eq!(T, &T::new(-0x4321).unwrap());
        }

        #[test]
        fn test_u32() {
            type T = NonZeroU32;

            assert_serial_eq!(T, &T::MIN);
            assert_serial_eq!(T, &T::MAX);

            assert_serial_eq!(T, &T::new(1).unwrap());
            assert_serial_eq!(T, &T::new(0x12345678).unwrap());
            assert_serial_eq!(T, &T::new(0x87654321).unwrap());
        }

        #[test]
        fn test_i32() {
            type T = NonZeroI32;

            assert_serial_eq!(T, &T::MIN);
            assert_serial_eq!(T, &T::MAX);

            assert_serial_eq!(T, &T::new(1).unwrap());
            assert_serial_eq!(T, &T::new(-1).unwrap());
            assert_serial_eq!(T, &T::new(0x12345678).unwrap());
            assert_serial_eq!(T, &T::new(-0x12345678).unwrap());
            assert_serial_eq!(T, &T::new(0x43218765).unwrap());
            assert_serial_eq!(T, &T::new(-0x43218765).unwrap());
        }

        #[test]
        fn test_u64() {
            type T = NonZeroU64;

            assert_serial_eq!(T, &T::MIN);
            assert_serial_eq!(T, &T::MAX);

            assert_serial_eq!(T, &T::new(1).unwrap());
            assert_serial_eq!(T, &T::new(0x123456789ABCDEF8).unwrap());
            assert_serial_eq!(T, &T::new(0x8FEDCBA987654321).unwrap());
        }

        #[test]
        fn test_i64() {
            type T = NonZeroI64;

            assert_serial_eq!(T, &T::MIN);
            assert_serial_eq!(T, &T::MAX);

            assert_serial_eq!(T, &T::new(1).unwrap());
            assert_serial_eq!(T, &T::new(-1).unwrap());
            assert_serial_eq!(T, &T::new(0x123456789ABCDEF8).unwrap());
            assert_serial_eq!(T, &T::new(-0x123456789ABCDEF8).unwrap());
            assert_serial_eq!(T, &T::new(0x43218FEDCBA98765).unwrap());
            assert_serial_eq!(T, &T::new(-0x43218FEDCBA98765).unwrap());
        }

        #[test]
        fn test_u128() {
            type T = NonZeroU128;

            assert_serial_eq!(T, &T::MIN);
            assert_serial_eq!(T, &T::MAX);

            assert_serial_eq!(T, &T::new(1).unwrap());
            assert_serial_eq!(T, &T::new(0x123456789ABCDEF88FEDCBA987654321).unwrap());
            assert_serial_eq!(T, &T::new(0x8FEDCBA987654321123456789ABCDEF8).unwrap());
        }

        #[test]
        fn test_i128() {
            type T = NonZeroI128;

            assert_serial_eq!(T, &T::MIN);
            assert_serial_eq!(T, &T::MAX);

            assert_serial_eq!(T, &T::new(1).unwrap());
            assert_serial_eq!(T, &T::new(-1).unwrap());
            assert_serial_eq!(T, &T::new(0x123456789ABCDEF88FEDCBA987654321).unwrap());
            assert_serial_eq!(T, &T::new(-0x123456789ABCDEF88FEDCBA987654321).unwrap());
            assert_serial_eq!(T, &T::new(0x43218FEDCBA9876556789ABCDEF81234).unwrap());
            assert_serial_eq!(T, &T::new(-0x43218FEDCBA9876556789ABCDEF81234).unwrap());
        }

        #[test]
        fn test_usize() {
            type T = NonZeroUsize;
            type Base = usize;

            assert_serial_eq!(T, &T::MIN);
            assert_serial_eq!(T, &T::MAX);

            assert_serial_eq!(T, &T::new(1).unwrap());
            assert_serial_eq!(
                T,
                &T::new(0x123456789ABCDEF88FEDCBA987654321u128 as Base).unwrap()
            );
            assert_serial_eq!(
                T,
                &T::new(0x8FEDCBA987654321123456789ABCDEF8u128 as Base).unwrap()
            );
        }

        #[test]
        fn test_isize() {
            type T = NonZeroIsize;
            type Base = isize;

            assert_serial_eq!(T, &T::MIN);
            assert_serial_eq!(T, &T::MAX);

            assert_serial_eq!(T, &T::new(1).unwrap());
            assert_serial_eq!(T, &T::new(-1).unwrap());
            assert_serial_eq!(
                T,
                &T::new(0x123456789ABCDEF88FEDCBA987654321i128 as Base).unwrap()
            );
            assert_serial_eq!(
                T,
                &T::new(-0x123456789ABCDEF88FEDCBA987654321i128 as Base).unwrap()
            );
            assert_serial_eq!(
                T,
                &T::new(0x43218FEDCBA9876556789ABCDEF81234i128 as Base).unwrap()
            );
            assert_serial_eq!(
                T,
                &T::new(-0x43218FEDCBA9876556789ABCDEF81234i128 as Base).unwrap()
            );
        }
    }

    mod atomic {
        use crate::assert_serial_eq;
        use core::sync::atomic::*;

        #[test]
        #[cfg(target_has_atomic_load_store = "8")]
        fn test_bool() {
            let load = |atomic: &AtomicBool| atomic.load(Ordering::SeqCst);
            assert_serial_eq!(AtomicBool, &AtomicBool::new(false), load);
            assert_serial_eq!(AtomicBool, &AtomicBool::new(true), load);
        }

        #[test]
        #[cfg(target_has_atomic_load_store = "8")]
        fn test_u8() {
            type T = AtomicU8;
            type Base = u8;

            let load = |atomic: &T| atomic.load(Ordering::SeqCst);
            assert_serial_eq!(T, &T::new(Base::MIN), load);
            assert_serial_eq!(T, &T::new(Base::MAX), load);

            assert_serial_eq!(T, &T::new(1), load);
            assert_serial_eq!(T, &T::new(0x34), load);
            assert_serial_eq!(T, &T::new(0x43), load);
        }

        #[test]
        #[cfg(target_has_atomic_load_store = "8")]
        fn test_i8() {
            type T = AtomicI8;
            type Base = i8;

            let load = |atomic: &T| atomic.load(Ordering::SeqCst);
            assert_serial_eq!(T, &T::new(Base::MIN), load);
            assert_serial_eq!(T, &T::new(Base::MIN), load);

            assert_serial_eq!(T, &T::new(1), load);
            assert_serial_eq!(T, &T::new(-1), load);
            assert_serial_eq!(T, &T::new(0x34), load);
            assert_serial_eq!(T, &T::new(-0x34), load);
            assert_serial_eq!(T, &T::new(0x43), load);
            assert_serial_eq!(T, &T::new(-0x43), load);
        }

        #[test]
        #[cfg(target_has_atomic_load_store = "16")]
        fn test_u16() {
            type T = AtomicU16;
            type Base = u16;

            let load = |atomic: &T| atomic.load(Ordering::SeqCst);
            assert_serial_eq!(T, &T::new(Base::MIN), load);
            assert_serial_eq!(T, &T::new(Base::MAX), load);

            assert_serial_eq!(T, &T::new(1), load);
            assert_serial_eq!(T, &T::new(0x1234), load);
            assert_serial_eq!(T, &T::new(0x4321), load);
        }

        #[test]
        #[cfg(target_has_atomic_load_store = "16")]
        fn test_i16() {
            type T = AtomicI16;
            type Base = i16;

            let load = |atomic: &T| atomic.load(Ordering::SeqCst);
            assert_serial_eq!(T, &T::new(Base::MIN), load);
            assert_serial_eq!(T, &T::new(Base::MAX), load);

            assert_serial_eq!(T, &T::new(1), load);
            assert_serial_eq!(T, &T::new(-1), load);
            assert_serial_eq!(T, &T::new(0x1234), load);
            assert_serial_eq!(T, &T::new(-0x1234), load);
            assert_serial_eq!(T, &T::new(0x4321), load);
            assert_serial_eq!(T, &T::new(-0x4321), load);
        }

        #[test]
        #[cfg(target_has_atomic_load_store = "32")]
        fn test_u32() {
            type T = AtomicU32;
            type Base = u32;

            let load = |atomic: &T| atomic.load(Ordering::SeqCst);
            assert_serial_eq!(T, &T::new(Base::MIN), load);
            assert_serial_eq!(T, &T::new(Base::MAX), load);

            assert_serial_eq!(T, &T::new(1), load);
            assert_serial_eq!(T, &T::new(0x12345678), load);
            assert_serial_eq!(T, &T::new(0x87654321), load);
        }

        #[test]
        #[cfg(target_has_atomic_load_store = "32")]
        fn test_i32() {
            type T = AtomicI32;
            type Base = i32;

            let load = |atomic: &T| atomic.load(Ordering::SeqCst);
            assert_serial_eq!(T, &T::new(Base::MIN), load);
            assert_serial_eq!(T, &T::new(Base::MAX), load);

            assert_serial_eq!(T, &T::new(1), load);
            assert_serial_eq!(T, &T::new(-1), load);
            assert_serial_eq!(T, &T::new(0x12345678), load);
            assert_serial_eq!(T, &T::new(-0x12345678), load);
            assert_serial_eq!(T, &T::new(0x43218765), load);
            assert_serial_eq!(T, &T::new(-0x43218765), load);
        }

        #[test]
        #[cfg(target_has_atomic_load_store = "64")]
        fn test_u64() {
            type T = AtomicU64;
            type Base = u64;

            let load = |atomic: &T| atomic.load(Ordering::SeqCst);
            assert_serial_eq!(T, &T::new(Base::MIN), load);
            assert_serial_eq!(T, &T::new(Base::MAX), load);

            assert_serial_eq!(T, &T::new(1), load);
            assert_serial_eq!(T, &T::new(0x123456789ABCDEF8), load);
            assert_serial_eq!(T, &T::new(0x8FEDCBA987654321), load);
        }

        #[test]
        #[cfg(target_has_atomic_load_store = "64")]
        fn test_i64() {
            type T = AtomicI64;
            type Base = i64;

            let load = |atomic: &T| atomic.load(Ordering::SeqCst);
            assert_serial_eq!(T, &T::new(Base::MIN), load);
            assert_serial_eq!(T, &T::new(Base::MAX), load);

            assert_serial_eq!(T, &T::new(1), load);
            assert_serial_eq!(T, &T::new(-1), load);
            assert_serial_eq!(T, &T::new(0x123456789ABCDEF8), load);
            assert_serial_eq!(T, &T::new(-0x123456789ABCDEF8), load);
            assert_serial_eq!(T, &T::new(0x43218FEDCBA98765), load);
            assert_serial_eq!(T, &T::new(-0x43218FEDCBA98765), load);
        }

        #[test]
        #[cfg(target_has_atomic_load_store = "128")]
        #[cfg(feature = "atomic_int_128")]
        fn test_u128() {
            type T = AtomicU128;
            type Base = u128;

            let load = |atomic: &T| atomic.load(Ordering::SeqCst);
            assert_serial_eq!(T, &T::new(Base::MIN), load);
            assert_serial_eq!(T, &T::new(Base::MAX), load);

            assert_serial_eq!(T, &T::new(1), load);
            assert_serial_eq!(T, &T::new(0x123456789ABCDEF88FEDCBA987654321), load);
            assert_serial_eq!(T, &T::new(0x8FEDCBA987654321123456789ABCDEF8), load);
        }

        #[test]
        #[cfg(target_has_atomic_load_store = "128")]
        #[cfg(feature = "atomic_int_128")]
        fn test_i128() {
            type T = AtomicI128;
            type Base = i128;

            let load = |atomic: &T| atomic.load(Ordering::SeqCst);
            assert_serial_eq!(T, &T::new(Base::MIN), load);
            assert_serial_eq!(T, &T::new(Base::MAX), load);

            assert_serial_eq!(T, &T::new(1), load);
            assert_serial_eq!(T, &T::new(-1), load);
            assert_serial_eq!(T, &T::new(0x123456789ABCDEF88FEDCBA987654321), load);
            assert_serial_eq!(T, &T::new(-0x123456789ABCDEF88FEDCBA987654321), load);
            assert_serial_eq!(T, &T::new(0x43218FEDCBA9876556789ABCDEF81234), load);
            assert_serial_eq!(T, &T::new(-0x43218FEDCBA9876556789ABCDEF81234), load);
        }

        #[test]
        #[cfg(target_has_atomic_load_store = "ptr")]
        fn test_usize() {
            type T = AtomicUsize;
            type Base = usize;

            let load = |atomic: &T| atomic.load(Ordering::SeqCst);
            assert_serial_eq!(T, &T::new(Base::MIN), load);
            assert_serial_eq!(T, &T::new(Base::MAX), load);

            assert_serial_eq!(T, &T::new(1), load);
            assert_serial_eq!(
                T,
                &T::new(0x123456789ABCDEF88FEDCBA987654321u128 as Base),
                load
            );
            assert_serial_eq!(
                T,
                &T::new(0x8FEDCBA987654321123456789ABCDEF8u128 as Base),
                load
            );
        }

        #[test]
        #[cfg(target_has_atomic_load_store = "ptr")]
        fn test_isize() {
            type T = AtomicIsize;
            type Base = isize;

            let load = |atomic: &T| atomic.load(Ordering::SeqCst);
            assert_serial_eq!(T, &T::new(Base::MIN), load);
            assert_serial_eq!(T, &T::new(Base::MAX), load);

            assert_serial_eq!(T, &T::new(1), load);
            assert_serial_eq!(T, &T::new(-1), load);
            assert_serial_eq!(
                T,
                &T::new(0x123456789ABCDEF88FEDCBA987654321i128 as Base),
                load
            );
            assert_serial_eq!(
                T,
                &T::new(-0x123456789ABCDEF88FEDCBA987654321i128 as Base),
                load
            );
            assert_serial_eq!(
                T,
                &T::new(0x43218FEDCBA9876556789ABCDEF81234i128 as Base),
                load
            );
            assert_serial_eq!(
                T,
                &T::new(-0x43218FEDCBA9876556789ABCDEF81234i128 as Base),
                load
            );
        }
    }
}
