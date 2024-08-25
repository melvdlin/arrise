use crate::{Deserialize, SerialSize, Serialize};
use core::mem::transmute;
use core::ptr::NonNull;

impl<T: SerialSize, const SIZE: usize> SerialSize for [T; SIZE] {
    const SIZE: usize = SIZE * size_of::<T>();
}

impl<T: Serialize, const SIZE: usize> Serialize for [T; SIZE]
where
    [(); <T as SerialSize>::SIZE]:,
{
    #[inline(always)]
    fn serialize(&self, buffer: &mut [u8; <Self as SerialSize>::SIZE]) {
        if size_of::<T>() == 0 {
            return;
        }
        for (value, buffer) in self
            .iter()
            .zip(buffer.array_chunks_mut::<{ <T as SerialSize>::SIZE }>())
        {
            value.serialize(buffer)
        }
    }
}

impl<T: Deserialize, const LEN: usize> Deserialize for [T; LEN]
where
    [(); <T as SerialSize>::SIZE]:,
{
    type Error = T::Error;

    unsafe fn deserialize_raw(
        into: NonNull<[T; LEN]>,
        buffer: &[u8; <Self as SerialSize>::SIZE],
    ) -> Result<(), T::Error> {
        unsafe {
            // Safety:
            // - `[MaybeUninit<T>; LEN]` has the same layout as `[T; LEN]`

            let into = transmute::<NonNull<[T; LEN]>, NonNull<T>>(into);

            for (i, buffer) in
                (0..LEN).zip(buffer.array_chunks::<{ <T as SerialSize>::SIZE }>())
            {
                // Safety:
                // `i` is bounded by LEN and therefore never exceeds the allocation
                <T as Deserialize>::deserialize_raw(into.add(i), buffer)?;
            }
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    //! see [`crate::impls::core::tuple::tests::test_complex`] for an array (de-)serialisation test
}
