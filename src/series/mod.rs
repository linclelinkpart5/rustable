
use crate::traits::Storable;
use crate::types::DType;

#[derive(Debug, Default)]
pub struct Series<T>
where
    T: Storable,
{
    /// Name of the series, used when adding to or extracting from a `Table`.
    pub name: Option<String>,

    /// Raw storage for the contained values.
    pub values: Vec<T>,
}

impl<T> Series<T>
where
    T: Storable,
{
    // fn dtype_associated() -> DType {
    //     T::dtype()
    // }

    /// Returns the `DType` of this `Series`.
    pub fn dtype(&self) -> DType {
        T::dtype()
    }
}
