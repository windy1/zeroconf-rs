//! Provides builder related helpers

/// Implements a `builder()` function for the specified type
pub trait BuilderDelegate<T: Default> {
    /// Initializes a new default builder of type `T`
    fn builder() -> T {
        T::default()
    }
}
