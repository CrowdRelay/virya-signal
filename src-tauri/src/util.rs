//! Small shared helpers used across the native crate. Kept dependency-free so
//! every module (`api`, `vault`, commands) can pull them in without pulling in
//! unrelated state or error types.

pub(crate) trait OptionValueOrExt<T> {
    fn value_or(self, fallback: T) -> T;
}

impl<T> OptionValueOrExt<T> for Option<T> {
    #[allow(clippy::manual_unwrap_or)]
    fn value_or(self, fallback: T) -> T {
        match self {
            Some(value) => value,
            None => fallback,
        }
    }
}
