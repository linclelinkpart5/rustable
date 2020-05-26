
use std::fmt::Debug;

use crate::types::DType;

/// The main trait the defines what types are able to be stored in a `Table`.
pub trait Storable: Debug + Clone + Send + Sized {
    /// Returns the [`DType`] of this type.
    fn dtype() -> DType;
}
