//! Pieces of rust's convert.rs that include `try_` varients of `From` and `Into`.
//!
//! I want to use them, and I don't want to use nightly.

/// An attempted conversion that consumes `self`, which may or may not be expensive.
///
/// Library authors should not directly implement this trait, but should prefer implementing
/// the `TryFrom` trait, which offers greater flexibility and provides an equivalent `TryInto`
/// implementation for free, thanks to a blanket implementation in the standard library.
pub trait TryInto<T>: Sized {
    /// The type returned in the event of a conversion error.
    type Err;

    /// Performs the conversion.
    fn try_into(self) -> Result<T, Self::Err>;
}

/// Attempt to construct `Self` via a conversion.
pub trait TryFrom<T>: Sized {
    /// The type returned in the event of a conversion error.
    type Err;

    /// Performs the conversion.
    fn try_from(T) -> Result<Self, Self::Err>;
}
