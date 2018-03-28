pub trait MessageWithId {
    type Id;

    fn id(&self) -> Self::Id;
}

macro_rules! impl_for_pair_tuples {
    ($type:ty $(,)*) => {
        impl<T> MessageWithId for ($type, T) {
            type Id = $type;

            fn id(&self) -> Self::Id {
                self.0
            }
        }
    };

    ($type:ty $(, $rest:ty)* $(,)*) => {
        impl_for_pair_tuples!( $type );
        impl_for_pair_tuples!( $($rest),* );
    };
}

impl_for_pair_tuples! {
    u8,
    u16,
    u32,
    u64,
    usize,
    i8,
    i16,
    i32,
    i64,
    isize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pair_tuple_implementation_for_u8() {
        let message = (10u8, ());

        assert_eq!(message.id(), 10u8);
    }

    #[test]
    fn pair_tuple_implementation_for_isize() {
        let message = (1_000isize, "dummy string");

        assert_eq!(message.id(), 1_000isize);
    }
}
