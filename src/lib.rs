//! AAAA
//!
//! HELP I'M STUCK IN A CRATE DOC QUICK CARGO IS COMING

#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(array_chunks)]
#![cfg_attr(not(feature = "std"), no_std)]
#![feature(cfg_target_has_atomic)]
#![cfg_attr(feature = "atomic_int_128", feature(integer_atomics))]

use core::mem::MaybeUninit;
use core::ptr::NonNull;

pub mod impls;

/// A trait indicating the size of the serialized form of `Self`.
pub trait SerialSize {
    /// The size of the buffer to serialize into.
    const SIZE: usize;
}

/// Serialize `Self` into a const-sized byte buffer.
pub trait Serialize: SerialSize {
    /// Serialize `self` into the provided `buffer`.
    fn serialize(&self, buffer: &mut [u8; Self::SIZE]);
}

/// Deserialize `Self` from a const-sized byte buffer directly into a pre-allocated [`MaybeUninit<Self>`].
pub trait Deserialize: SerialSize + Sized {
    /// The error type that can occur during deserialization.
    type Error;

    /// Deserialize a `Self` from the proided `buffer`.
    fn deserialize(buffer: &[u8; Self::SIZE]) -> Result<Self, Self::Error> {
        let mut result = MaybeUninit::uninit();
        <Self as Deserialize>::deserialize_into_uninit(&mut result, buffer)?;
        // Safety: we just initialized this
        Ok(unsafe { result.assume_init() })
    }

    /// Deserialize a `Self` from the provided `buffer`.
    /// Returns a mutable refernce to the initialized contents of `into`.
    ///
    /// The usual rules of [`MaybeUninit::write`] apply.
    fn deserialize_into_uninit<'a>(
        into: &'a mut MaybeUninit<Self>,
        buffer: &[u8; Self::SIZE],
    ) -> Result<&'a mut Self, Self::Error> {
        unsafe {
            // Safety:
            // `into` is a mutable reference, therefore, any derived pointer
            // is valid for writes (and therefore non-null).
            let into_ptr = NonNull::new_unchecked(into.as_mut_ptr());
            Self::deserialize_raw(into_ptr, buffer)?;

            // Safety:
            // `deserialize_raw` just initialised `into`.
            Ok(into.assume_init_mut())
        }
    }

    /// Deserialize a `Self` from the provided `buffer`.
    /// If this function returns `Ok`, `into` will contain a valid `Self`.
    /// Returns a mutable refernce to the initialized contents of `into`.
    ///
    /// # Safety
    /// `into` must be valid for writes.
    /// For details, see [`core::ptr::write`].
    unsafe fn deserialize_raw(
        into: NonNull<Self>,
        buffer: &[u8; Self::SIZE],
    ) -> Result<(), Self::Error>;
}

#[allow(unused)]
macro_rules! assert_serial_eq {
    ($ty:ty, $x:expr) => {{
        use core::ops::Deref;
        use $crate::{Deserialize, SerialSize, Serialize};

        let mut buf = [0u8; <$ty as SerialSize>::SIZE];
        <$ty as Serialize>::serialize($x, &mut buf);
        let de = <$ty as Deserialize>::deserialize(&buf).expect("deserialization failed");
        assert_eq!($x.deref(), (&de).deref());
    }};
}

#[allow(unused)]
use assert_serial_eq;
