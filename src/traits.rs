
use std::fmt::Debug;

use crate::types::DType;

/// The main trait the defines what types are able to be stored in a `Series`.
pub trait Storable: Debug + Clone + Send + Sized {
    /// Returns the [`DType`] of this type.
    fn dtype() -> DType;
}

/// Core interface used for indices.
pub trait Indexer<L>
where
    L: Storable,
{
    /// Translates a loc-style key into a raw iloc-style key.
    fn loc_to_iloc(&self, loc: &L) -> Option<usize>;
}
