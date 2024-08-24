use crate::{Deserialize, SerialSize, Serialize};
use core::mem::MaybeUninit;

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

impl<T: Deserialize, const SIZE: usize> Deserialize for [T; SIZE]
where
    [(); <T as SerialSize>::SIZE]:,
{
    type Error = T::Error;

    fn deserialize_into_uninit<'a>(
        into: &'a mut MaybeUninit<[T; SIZE]>,
        buffer: &[u8; <Self as SerialSize>::SIZE],
    ) -> Result<&'a mut Self, T::Error> {
        fn lower_uninit<T, const N: usize>(
            from: &mut MaybeUninit<[T; N]>,
        ) -> &mut [MaybeUninit<T>; N] {
            // Safety:
            // MaybeUninit<[T; N]> and [MaybeUninit<T>; N]
            // have the same layout and invariants
            unsafe { core::mem::transmute(from) }
        }

        fn lift_uninit<T, const N: usize>(
            from: &mut [MaybeUninit<T>; N],
        ) -> &mut MaybeUninit<[T; N]> {
            // Safety:
            // MaybeUninit<[T; N]> and [MaybeUninit<T>; N]
            // have the same layout and invariants
            unsafe { core::mem::transmute(from) }
        }

        let into = {
            let into = lower_uninit(into);
            for (value, buffer) in into
                .iter_mut()
                .zip(buffer.array_chunks::<{ <T as SerialSize>::SIZE }>())
            {
                <T as Deserialize>::deserialize_into_uninit(value, buffer)?;
            }
            lift_uninit(into)
        };

        Ok(unsafe { into.assume_init_mut() })
    }
}
