
use crate::traits::RawType;
use crate::traits::Storable;

pub struct ValueStore<V: Storable>(Vec<V>);

impl<V: Storable> ValueStore<V> {
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl<R: RawType + Storable> ValueStore<Option<R>> {
    pub fn fill_none(self, value: R) -> ValueStore<R> {
        ValueStore(
            self.0
            .into_iter()
            .map(|v| v.unwrap_or(value.clone()))
            .collect()
        )
    }

    pub fn fill_none_with<F>(self, func: F) -> ValueStore<R>
    where
        F: Fn() -> R,
    {
        ValueStore(
            self.0
            .into_iter()
            .map(|v| v.unwrap_or_else(|| func()))
            .collect()
        )
    }

    pub fn drop_none(self) -> ValueStore<R> {
        ValueStore(
            self.0
            .into_iter()
            .filter_map(|v| v)
            .collect()
        )
    }
}
