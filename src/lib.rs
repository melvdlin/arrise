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

/// Deserialize `Self` from a const-sized byte buffer.
pub trait Deserialize: SerialSize + Sized {
    /// The error type that can occur during deserialization.
    type Error;

    /// Deserialize a `Self` from the provided `buffer`.
    fn deserialize(buffer: &[u8; Self::SIZE]) -> Result<Self, Self::Error>;
}

/// Deserialize `Self` from a const-sized byte buffer directly into a pre-allocated [`MaybeUninit<Self>`].
pub trait DeserializeIntoUninit: SerialSize + Sized {
    /// The error type that can occur during deserialization.
    type Error;

    /// Deserialize a `Self` from the provided `buffer`.
    /// Returns a mutable refernce to the initialized contents of `into`.
    ///
    /// The usual rules of [`MaybeUninit::write`] apply.
    fn deserialize_into_uninit<'a>(
        into: &'a mut MaybeUninit<Self>,
        buffer: &[u8; Self::SIZE],
    ) -> Result<&'a mut Self, Self::Error>;
}

impl<T> Deserialize for T
where
    T: DeserializeIntoUninit,
{
    type Error = <Self as DeserializeIntoUninit>::Error;

    fn deserialize(buffer: &[u8; Self::SIZE]) -> Result<Self, Self::Error> {
        let mut result = MaybeUninit::uninit();
        <Self as DeserializeIntoUninit>::deserialize_into_uninit(&mut result, buffer)?;
        // Safety: we just initialized this
        Ok(unsafe { result.assume_init() })
    }
}
