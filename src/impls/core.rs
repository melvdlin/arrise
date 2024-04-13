use crate::{DeserializeIntoUninit, SerialSize, Serialize};
use arrise_macro::{
    deserialize_error_assoc_type_for_tuple, deserialize_error_type_for_tuple, impl_for_tuples,
};
use core::mem::{size_of, MaybeUninit};

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

impl<T: DeserializeIntoUninit, const SIZE: usize> DeserializeIntoUninit for [T; SIZE]
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
                <T as DeserializeIntoUninit>::deserialize_into_uninit(value, buffer)?;
            }
            lift_uninit(into)
        };

        Ok(unsafe { into.assume_init_mut() })
    }
}

macro_rules! impl_serial_size_for_tuple {
    (($($ts:ident,)*)) => {
        impl<$($ts: SerialSize,)*> SerialSize for ($($ts,)*) {
            const SIZE: usize = 0 $(+ <$ts as SerialSize>::SIZE)*;
        }
    };
}

macro_rules! impl_serialize_for_tuple {
    (($($ts:ident,)*)) => {
        impl<$($ts: Serialize,)*> Serialize for ($($ts,)*)
        where
            [(); <($($ts,)*) as SerialSize>::SIZE $(- <$ts as SerialSize>::SIZE)*]:, {

            #[allow(unused_variables, non_snake_case)]
            fn serialize(&self, buffer: &mut [u8; <Self as SerialSize>::SIZE]) {
                let ($($ts,)*) = self;
                $(
                let (head, buffer) = split_array::split_arr_mut(buffer);
                <$ts as Serialize>::serialize($ts, head);
                )*
            }
        }
    };
}

macro_rules! impl_deserialize_into_uninit_for_tuple {
    (($($ts:ident,)*)) => {
        deserialize_error_type_for_tuple!(($($ts,)*));

        impl<$($ts: DeserializeIntoUninit,)*> DeserializeIntoUninit for ($($ts,)*)
        where
            [(); <($($ts,)*) as SerialSize>::SIZE $(- <$ts as SerialSize>::SIZE)*]:, {

            type Error = deserialize_error_assoc_type_for_tuple!(($($ts,)*));

            #[allow(unused_variables, non_snake_case)]
            fn deserialize_into_uninit<'a>(
                into: &'a mut MaybeUninit<Self>,
                buffer: &[u8; Self::SIZE]
            ) -> Result<&'a mut Self, Self::Error> {
                fn lower_uninit<$($ts,)*>(
                    from: &mut MaybeUninit<($($ts,)*)>,
                ) -> &mut ($(MaybeUninit<$ts>,)*) {
                    // Safety:
                    // MaybeUninit<(T0, T1, ...)> and (MaybeUninit<T1>, MaybeUninit<T2>, ...)
                    // have the same layout and invariants
                    unsafe { core::mem::transmute(from) }
                }

                fn lift_uninit<$($ts,)*>(
                    from: &mut ($(MaybeUninit<$ts>,)*),
                ) -> &mut MaybeUninit<($($ts,)*)> {
                    // Safety:
                    // MaybeUninit<(T0, T1, ...)> and (MaybeUninit<T1>, MaybeUninit<T2>, ...)
                    // have the same layout and invariants
                    unsafe { core::mem::transmute(from) }
                }

                let into = {
                    let into = lower_uninit(into);
                    let ($($ts,)*) = into;

                    $(
                    let (head, buffer) = split_array::split_arr(buffer);
                    <$ts as DeserializeIntoUninit>::deserialize_into_uninit($ts, head)
                        .map_err(|e| <Self as DeserializeIntoUninit>::Error::$ts(e))?;
                    )*

                    lift_uninit(into)
                };

                Ok(unsafe { into.assume_init_mut() })
            }
        }
    };
}

macro_rules! impl_for_tuple {
    (($($ts:ident,)*)) => {
        impl_serial_size_for_tuple!(($($ts,)*));
        impl_serialize_for_tuple!(($($ts,)*));
        impl_deserialize_into_uninit_for_tuple!(($($ts,)*));
    };
}

#[cfg(not(feature = "large_tuples"))]
impl_for_tuples!(12);
#[cfg(feature = "large_tuples")]
impl_for_tuples!(64);
