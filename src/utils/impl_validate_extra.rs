#[macro_export]
macro_rules! impl_validate_extra {
    ($item: ty) => {
        impl crate::validators::ValidateExtra for $item {}
    };
}
