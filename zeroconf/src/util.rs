use crate::Result;
use std::future::Future;
use std::pin::Pin;

pub(crate) type PinnedFuture<'a, T> = Pin<Box<(dyn Future<Output = Result<T>> + 'a)>>;

/// Implements a `builder()` function for the specified type
pub trait BuilderDelegate<T: Default> {
    /// Initializes a new default builder of type `T`
    fn builder() -> T {
        T::default()
    }
}
