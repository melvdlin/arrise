use crate::{Deserialize, SerialSize, Serialize};
use arrise_macro::{
    deserialize_error_assoc_type_for_tuple, deserialize_error_type_for_tuple,
    impl_for_tuples,
};
use core::mem::MaybeUninit;

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

macro_rules! impl_deserialize_for_tuple {
    (($($ts:ident,)*)) => {
        deserialize_error_type_for_tuple!(($($ts,)*));

        impl<$($ts: Deserialize,)*> Deserialize for ($($ts,)*)
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
                    <$ts as Deserialize>::deserialize_into_uninit($ts, head)
                        .map_err(|e| <Self as Deserialize>::Error::$ts(e))?;
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
        impl_deserialize_for_tuple!(($($ts,)*));
    };
}

#[cfg(not(feature = "large_tuples"))]
impl_for_tuples!(12);
#[cfg(feature = "large_tuples")]
impl_for_tuples!(64);
