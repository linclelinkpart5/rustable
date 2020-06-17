
pub mod error;
pub mod dense;
pub mod sparse;
pub mod values;

use crate::traits::Storable;
use crate::traits::Label;

pub use self::dense::SeriesDense;
pub use self::sparse::SeriesSparse;

pub enum Series<'a, L: Label, V: Storable> {
    Dense(SeriesDense<'a, L, V>),
    Sparse(SeriesSparse<'a, L, V>),
}
