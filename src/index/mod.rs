
pub mod iter;

use std::borrow::Borrow;
use std::hash::Hash;

use indexmap::IndexSet as Set;

use crate::traits::Storable;

use self::iter::Iter;
use self::iter::IntoIter;

#[derive(Debug)]
pub struct Index<K>(Set<K>)
where
    K: Storable + Eq + Hash,
;

impl<K> Index<K>
where
    K: Storable + Eq + Hash,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_labels<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = K>,
    {
        Self(iter.into_iter().collect())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }

    pub fn clear(&mut self) {
        self.0.clear()
    }

    pub fn push(&mut self, key: K) -> bool {
        self.0.insert(key)
    }

    // NOTE: This will always be in "iloc" order, so no need to provide a "full"
    //       flavor of this method.
    pub fn iter(&self) -> Iter<'_, K> {
        Iter(self.0.iter())
    }

    pub fn contains<Q>(&self, label: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.0.contains(label)
    }
}

// NOTE: This is needed because `#[derive(Default)]` only works if the type `K`
// is also `Default`, for some reason!
impl<K> Default for Index<K>
where
    K: Storable + Eq + Hash,
{
    fn default() -> Self {
        Self(Set::new())
    }
}

impl<K> IntoIterator for Index<K>
where
    K: Storable + Eq + Hash,
{
    type Item = K;
    type IntoIter = IntoIter<K>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self.0.into_iter())
    }
}
