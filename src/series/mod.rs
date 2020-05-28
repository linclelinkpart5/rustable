
use std::borrow::Borrow;
use std::hash::Hash;

use crate::traits::Storable;
use crate::types::DType;
use crate::index::Index;

#[derive(Debug, Default)]
pub struct Series<K, V>
where
    K: Storable + Hash + Eq,
    V: Storable,
{
    index: Option<Index<K>>,
    values: Vec<V>,
}

impl<K, V> Series<K, V>
where
    K: Storable + Hash + Eq,
    V: Storable,
{
    /// Returns the `DType` of this `Series`.
    pub fn dtype(&self) -> DType {
        V::dtype()
    }

    /// Returns a readonly reference to the `Index` contained in this `Series`,
    /// if there is one.
    pub fn index(&self) -> Option<&Index<K>> {
        self.index.as_ref()
    }

    /// Consumes the `Series` and returns the value data store.
    pub fn into_raw(self) -> Vec<V> {
        self.values
    }

    /// Returns a readonly slice of the value data store of this `Series`.
    pub fn as_slice(&self) -> &[V] {
        &self.values
    }

    /// Returns a mutable slice of the value data store of this `Series`.
    pub fn as_mut_slice(&mut self) -> &mut [V] {
        &mut self.values
    }

    /// Given a key, returns a readonly reference to its value in the `Series`,
    /// if it exists.
    pub fn loc<Q>(&self, _loc: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        // Always return `None` if there is no index.
        todo!("Need to implement lookup on `Index` first");
    }

    /// Given a key, returns a mutable reference to its value in the `Series`,
    /// if it exists.
    pub fn loc_mut<Q>(&mut self, _loc: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        // Always return `None` if there is no index.
        todo!("Need to implement lookup on `Index` first");
    }
}
