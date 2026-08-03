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

pub(crate) trait OptionValueOrElseExt<T> {
    fn value_or_else<F>(self, fallback: F) -> T
    where
        F: FnOnce() -> T;
}

impl<T> OptionValueOrElseExt<T> for Option<T> {
    #[allow(clippy::manual_unwrap_or)]
    fn value_or_else<F>(self, fallback: F) -> T
    where
        F: FnOnce() -> T,
    {
        match self {
            Some(value) => value,
            None => fallback(),
        }
    }
}
