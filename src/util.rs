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

/// Spawns a cancellable scoped task with the future type erased at the call
/// site. Leptos' spawner is generic, so each of the ~90 call sites otherwise
/// monomorphizes the whole abort-handle/scope/executor wrapper again. Boxing
/// collapses that plumbing to a single instantiation; the per-call cost is one
/// heap allocation on a path that already crosses IPC.
pub(crate) fn spawn_local<F>(future: F)
where
    F: std::future::Future<Output = ()> + 'static,
{
    let future: std::pin::Pin<Box<dyn std::future::Future<Output = ()>>> = Box::pin(future);
    leptos::task::spawn_local_scoped_with_cancellation(future);
}
