use crate::impls::IllegalBitPattern;
use crate::{Deserialize, SerialSize, Serialize};
use core::fmt::Debug;
use core::mem::{transmute, MaybeUninit};
use split_array::SplitArray;

impl<T: SerialSize> SerialSize for Option<T> {
    const SIZE: usize = <bool as SerialSize>::SIZE + <T as SerialSize>::SIZE;
}

impl<T: Serialize> Serialize for Option<T>
where
    [(); <Self as SerialSize>::SIZE - <bool as SerialSize>::SIZE]:,
    [(); <Self as SerialSize>::SIZE
        - <bool as SerialSize>::SIZE
        - <T as SerialSize>::SIZE]:,
{
    fn serialize(&self, buffer: &mut [u8; <Self as SerialSize>::SIZE]) {
        let (head, buffer) = buffer.split_arr_mut();
        match self.as_ref() {
            | None => false.serialize(head),
            | Some(data) => {
                true.serialize(head);
                let (head, tail) = buffer.split_arr_mut();
                data.serialize(head);
                debug_assert_eq!(0, tail.len());
            }
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum DeserializeOptionError<T> {
    IllegalTag,
    Data(T),
}

impl<T> From<IllegalBitPattern> for DeserializeOptionError<T> {
    fn from(_error: IllegalBitPattern) -> Self {
        Self::IllegalTag
    }
}

impl<T: Deserialize> Deserialize for Option<T>
where
    [(); <Self as SerialSize>::SIZE - <bool as SerialSize>::SIZE]:,
    [(); <Self as SerialSize>::SIZE
        - <bool as SerialSize>::SIZE
        - <T as SerialSize>::SIZE]:,
{
    type Error = DeserializeOptionError<<T as Deserialize>::Error>;

    fn deserialize_into_uninit<'a>(
        into: &'a mut MaybeUninit<Option<T>>,
        buffer: &[u8; <Self as SerialSize>::SIZE],
    ) -> Result<&'a mut Self, Self::Error> {
        let (head, buffer) = buffer.split_arr();

        Ok(if <bool as Deserialize>::deserialize(head)? {
            let (head, tail) = buffer.split_arr();

            debug_assert_eq!(0, tail.len());

            unsafe {
                let into = transmute::<
                    &'a mut MaybeUninit<Option<T>>,
                    &'a mut MaybeUninit<Option<MaybeUninit<T>>>,
                >(into)
                .write(Some(MaybeUninit::uninit()));

                let inner = into.as_mut().unwrap_unchecked();

                T::deserialize_into_uninit(inner, head)
                    .map_err(DeserializeOptionError::Data)?;

                transmute::<&'a mut Option<MaybeUninit<T>>, &'a mut Option<T>>(into)
            }
        } else {
            into.write(None)
        })
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use crate::{assert_serial_eq, Deserialize, SerialSize, Serialize};

    type Data = [usize; 10];

    const X: Data = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];

    const fn same_type<const SIZE: usize, T>(x: [T; SIZE]) -> [T; SIZE] {
        x
    }

    #[test]
    fn test_data() {
        assert_serial_eq!(Data, &X);
    }

    #[test]
    fn test_none() {
        let none = None;
        assert_serial_eq!(Option<Data>, &none);
    }

    #[test]
    fn test_some() {
        let none = None;
        let some = Some(X);

        same_type([some, none]);

        assert_serial_eq!(Option<Data>, &some);
    }

    #[test]
    fn test_nested() {
        let none = None;
        let some = Some(X);
        let some_none = Some(none);
        let some_some = Some(some);

        same_type([some, none]);
        same_type([some_none, some_some]);

        assert_serial_eq!(Option<Option<Data>>, &some_none);
        assert_serial_eq!(Option<Option<Data>>, &some_some);
    }
}
