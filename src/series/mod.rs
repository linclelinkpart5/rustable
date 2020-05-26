
use crate::traits::Storable;

pub struct Series<T>
where
    T: Storable,
{
    /// Name of the series, used when adding to or extracting from a `Table`.
    pub name: Option<String>,

    /// Raw storage for the contained values.
    pub values: Vec<T>,
}
