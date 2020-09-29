//! Crate prelude

pub use crate::browser::TMdnsBrowser;
pub use crate::event_loop::TEventLoop;
pub use crate::service::TMdnsService;
pub use crate::txt_record::TTxtRecord;

/// Implements a `builder()` function for the specified type
pub trait BuilderDelegate<T: Default> {
    /// Initializes a new default builder of type `T`
    fn builder() -> T {
        T::default()
    }
}
