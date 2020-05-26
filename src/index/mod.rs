
use std::hash::Hash;

use indexmap::IndexSet;

use crate::traits::Storable;

#[derive(Debug)]
pub struct Index<K>
where
    K: Storable + Eq + Hash,
{
    pub keys: IndexSet<K>,
}

impl<K> Default for Index<K>
where
    K: Storable + Eq + Hash,
{
    fn default() -> Self {
        Self { keys: IndexSet::new() }
    }
}

impl<K> Index<K>
where
    K: Storable + Eq + Hash,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.keys.len()
    }

    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    pub fn capacity(&self) -> usize {
        self.keys.capacity()
    }

    pub fn clear(&mut self) {
        self.keys.clear()
    }

    pub fn push(&mut self, key: K) -> bool {
        self.keys.insert(key)
    }

    pub fn iter(&self) -> ! {
        todo!("Need to create `Iter<K>`")
    }
}
