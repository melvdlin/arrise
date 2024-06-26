use crate::{Deserialize, DeserializeIntoUninit, SerialSize, Serialize};
use core::char::CharTryFromError;
use core::mem::{size_of, MaybeUninit};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub struct IllegalBitPattern;

macro_rules! to_e_bytes {
    ($expr:expr) => {{
        #[cfg(feature = "primitive_ne")]
        {
            ($expr).to_ne_bytes()
        }
        #[cfg(feature = "primitive_le")]
        {
            ($expr).to_ne_bytes()
        }
        #[cfg(feature = "primitive_be")]
        {
            ($expr).to_be_bytes()
        }
    }};
}

macro_rules! from_e_bytes {
    ($ty:ty, $expr:expr) => {{
        #[cfg(feature = "primitive_ne")]
        {
            <$ty>::from_ne_bytes($expr)
        }
        #[cfg(feature = "primitive_le")]
        {
            <$ty>::from_le_bytes($expr)
        }
        #[cfg(feature = "primitive_be")]
        {
            <$ty>::from_be_bytes($expr)
        }
    }};
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

        impl crate::DeserializeIntoUninit for $ty {
            type Error = core::convert::Infallible;

            fn deserialize_into_uninit<'a>(
                into: &'a mut MaybeUninit<Self>,
                buffer: &[u8; <Self as crate::SerialSize>::SIZE],
            ) -> Result<&'a mut Self, <Self as crate::Deserialize>::Error> {
                Ok(into.write(from_e_bytes!($ty, *buffer)))
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

impl SerialSize for bool {
    const SIZE: usize = <u8 as SerialSize>::SIZE;
}

impl Serialize for bool {
    fn serialize(&self, buffer: &mut [u8; <Self as SerialSize>::SIZE]) {
        *buffer = [*self as u8]
    }
}

impl DeserializeIntoUninit for bool {
    type Error = IllegalBitPattern;

    fn deserialize_into_uninit<'a>(
        into: &'a mut MaybeUninit<Self>,
        buffer: &[u8; <Self as SerialSize>::SIZE],
    ) -> Result<&'a mut Self, Self::Error> {
        Ok(into.write(match *buffer {
            [0] => false,
            [1] => true,
            _ => Err(IllegalBitPattern)?,
        }))
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

impl DeserializeIntoUninit for char {
    type Error = CharTryFromError;

    fn deserialize_into_uninit<'a>(
        into: &'a mut MaybeUninit<Self>,
        buffer: &[u8; <Self as SerialSize>::SIZE],
    ) -> Result<&'a mut Self, Self::Error> {
        Ok(into.write(char::try_from(u32::deserialize(buffer).unwrap())?))
    }
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

        impl DeserializeIntoUninit for $nonzero {
            type Error = IllegalBitPattern;

            fn deserialize_into_uninit<'a>(
                into: &'a mut MaybeUninit<Self>,
                buffer: &[u8; Self::SIZE],
            ) -> Result<&'a mut Self, Self::Error> {
                Ok(into.write(
                    Self::new(<$primitive>::deserialize(buffer).unwrap())
                        .ok_or(IllegalBitPattern)?,
                ))
            }
        }
    };
}

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

macro_rules! impl_atomic {
    ($atomic:ty, $primitive:ty) => {
        impl SerialSize for $atomic {
            const SIZE: usize = size_of::<$primitive>();
        }

        impl Serialize for $atomic {
            fn serialize(&self, buffer: &mut [u8; Self::SIZE]) {
                self.load(core::sync::atomic::Ordering::SeqCst)
                    .serialize(buffer)
            }
        }

        impl DeserializeIntoUninit for $atomic {
            type Error = <$primitive as Deserialize>::Error;

            fn deserialize_into_uninit<'a>(
                into: &'a mut MaybeUninit<Self>,
                buffer: &[u8; Self::SIZE],
            ) -> Result<&'a mut Self, Self::Error> {
                Ok(into.write(Self::new(<$primitive>::deserialize(buffer)?)))
            }
        }
    };
}

#[cfg(target_has_atomic = "8")]
impl_atomic!(core::sync::atomic::AtomicBool, bool);
#[cfg(target_has_atomic = "8")]
impl_atomic!(core::sync::atomic::AtomicI8, i8);
#[cfg(target_has_atomic = "16")]
impl_atomic!(core::sync::atomic::AtomicI16, i16);
#[cfg(target_has_atomic = "32")]
impl_atomic!(core::sync::atomic::AtomicI32, i32);
#[cfg(target_has_atomic = "64")]
impl_atomic!(core::sync::atomic::AtomicI64, i64);
#[cfg(target_has_atomic = "ptr")]
impl_atomic!(core::sync::atomic::AtomicIsize, isize);
#[cfg(target_has_atomic = "8")]
impl_atomic!(core::sync::atomic::AtomicU8, u8);
#[cfg(target_has_atomic = "16")]
impl_atomic!(core::sync::atomic::AtomicU16, u16);
#[cfg(target_has_atomic = "32")]
impl_atomic!(core::sync::atomic::AtomicU32, u32);
#[cfg(target_has_atomic = "64")]
impl_atomic!(core::sync::atomic::AtomicU64, u64);
#[cfg(target_has_atomic = "ptr")]
impl_atomic!(core::sync::atomic::AtomicUsize, usize);
