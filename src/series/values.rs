
use crate::traits::Storable;

pub struct DenseValueStore<V: Storable>(Vec<V>);
pub struct SparseValueStore<V: Storable>(Vec<Option<V>>);

impl<V: Storable> SparseValueStore<V> {
    pub fn fill_gaps(self, value: V) -> DenseValueStore<V> {
        DenseValueStore(
            self.0
            .into_iter()
            .map(|v| v.unwrap_or(value.clone()))
            .collect()
        )
    }

    pub fn fill_gaps_with<F>(self, func: F) -> DenseValueStore<V>
    where
        F: Fn() -> V,
    {
        DenseValueStore(
            self.0
            .into_iter()
            .map(|v| v.unwrap_or_else(|| func()))
            .collect()
        )
    }

    pub fn drop_gaps(self) -> DenseValueStore<V> {
        DenseValueStore(
            self.0
            .into_iter()
            .filter_map(|v| v)
            .collect()
        )
    }
}
