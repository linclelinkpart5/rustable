
use std::fmt::Debug;
use std::hash::Hash;

use crate::types::DType;

/// The main trait the defines what types are able to be stored in a `Series`.
pub trait Storable: Debug + Clone + Send + Sized {
    /// Returns the [`DType`] of this type.
    fn dtype() -> DType;
}

/// Trait that defines what is needed for a label in an `Index`.
pub trait Label: Storable + Eq + Hash {}

impl<T: Storable + Eq + Hash> Label for T {}
