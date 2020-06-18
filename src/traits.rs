
use std::fmt::Debug;
use std::hash::Hash;

pub trait RawType: Debug + Clone + Send + Sized {}

/// The main trait the defines what types are able to be stored in a `Series`.
pub trait Storable: Debug + Clone + Send + Sized {}

// All `RawType`s can be `Storable`s.
impl <R: RawType> Storable for R {}

// In addition, all `Option<RawType>`s are `Storable`s.
impl <R: RawType> Storable for Option<R> {}

/// Trait that defines what is needed for a label in an `Index`.
pub trait Label: Storable + PartialEq + Eq + Hash + PartialOrd + Ord {}

impl<T: Storable + PartialEq + Eq + Hash + PartialOrd + Ord> Label for T {}
