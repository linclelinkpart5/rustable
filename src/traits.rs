
use std::fmt::Debug;
use std::fmt::Display;

use crate::types::DType;

pub trait Storable: Debug + Display + Clone + Send {
    /// Returns the [`DType`] of this type.
    fn dtype(&self) -> DType;
}
