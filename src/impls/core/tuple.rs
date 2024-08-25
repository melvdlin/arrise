use crate::{Deserialize, SerialSize, Serialize};
use arrise_macro::{
    deserialize_error_assoc_type_for_tuple, deserialize_error_type_for_tuple,
    impl_for_tuples,
};
use core::ptr::NonNull;

macro_rules! impl_serial_size_for_tuple {
    (($($ts:ident,)*)) => {
        impl<$($ts: SerialSize,)*> SerialSize for ($($ts,)*) {
            const SIZE: usize = 0 $(+ <$ts as SerialSize>::SIZE)*;
        }
    };
}

macro_rules! impl_serialize_for_tuple {
    (($($fields:tt: $ts:ident,)*)) => {
        impl<$($ts: Serialize,)*> Serialize for ($($ts,)*)
        where
            [(); <($($ts,)*) as SerialSize>::SIZE $(- <$ts as SerialSize>::SIZE)*]:, {

            #[allow(unused_variables, non_snake_case)]
            fn serialize(&self, buffer: &mut [u8; <Self as SerialSize>::SIZE]) {
                $(
                let (head, buffer) = split_array::split_arr_mut(buffer);
                <$ts as Serialize>::serialize(&self.$fields, head);
                )*
            }
        }
    };
}

macro_rules! impl_deserialize_for_tuple {
    (($($fields:tt: $ts:ident,)*)) => {
        deserialize_error_type_for_tuple!(($($ts,)*));

        impl<$($ts: Deserialize,)*> Deserialize for ($($ts,)*)
        where
            [(); <($($ts,)*) as SerialSize>::SIZE $(- <$ts as SerialSize>::SIZE)*]:, {

            type Error = deserialize_error_assoc_type_for_tuple!(($($ts,)*));

            #[allow(unused_variables, non_snake_case)]
            unsafe fn deserialize_raw(
                into: NonNull<Self>,
                buffer: &[u8; Self::SIZE]
            ) -> Result<(), Self::Error> {
                let into = into.as_ptr();
                #[allow(unused_unsafe)]
                unsafe {
                    $(
                    let (head, buffer) = split_array::split_arr(buffer);
                    // Safety:
                    // - taking a raw ref of a place is always safe
                    // - `into` is valid for writes, therefore,
                    //   any derived pointer is also valid for writes
                    let field = NonNull::new_unchecked(&raw mut (*into).$fields);
                    <$ts as Deserialize>::deserialize_raw(field, head)
                        .map_err(|e| <Self as Deserialize>::Error::$ts(e))?;
                    )*
                }

                Ok(())
            }
        }
    };
}

macro_rules! impl_for_tuple {
    (($($fields:tt: $ts:ident,)*)) => {
        impl_serial_size_for_tuple!(($($ts,)*));
        impl_serialize_for_tuple!(($($fields: $ts,)*));
        impl_deserialize_for_tuple!(($($fields: $ts,)*));
    };
}

#[cfg(not(feature = "large_tuples"))]
impl_for_tuples!(12);
#[cfg(feature = "large_tuples")]
impl_for_tuples!(64);

#[cfg(test)]
mod tests {
    use crate::assert_serial_eq;

    #[test]
    fn test_complex() {
        assert_serial_eq!(
            (
                u64,
                bool,
                f32,
                Option<[f64; 3]>,
                ((usize, bool), Option<i128>, [(bool, Option<f64>, i16); 2])
            ),
            &(
                1234u64,
                true,
                12.3456f32,
                Some([4.5f64, 9.133f64, -14.231f64]),
                (
                    (2, false),
                    None,
                    [(true, None, 2i16), (false, Some(23.5), 38i16)]
                )
            )
        );
    }
}
