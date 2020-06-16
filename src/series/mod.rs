
pub mod error;

use std::borrow::Borrow;
use std::borrow::Cow;
use std::hash::Hash;

use crate::traits::Storable;
use crate::traits::Label;
use crate::index::Index;

use self::error::DuplicateIndexLabel;

#[derive(Debug)]
pub struct Series<'a, K, V>
where
    K: Label,
    V: Storable,
{
    index: Cow<'a, Index<K>>,
    values: Vec<V>,
}

impl<'a, K, V> Series<'a, K, V>
where
    K: Label,
    V: Storable,
{
    /// Creates a new, empty `Series` with no values and an empty `Index`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new `Series` from an iterable of index label and value pairs.
    /// If duplicated index labels are encountered, an `DuplicateIndexLabel`
    /// error is returned.
    pub fn from_pairs<I>(pairs: I) -> Result<Self, DuplicateIndexLabel<K>>
    where
        I: IntoIterator<Item = (K, V)>,
    {
        let mut pairs = pairs.into_iter();

        // Use `Iterator::size_hint` to try and pre-allocate.
        let (_, opt_upper) = pairs.size_hint();

        let (mut index, mut values) = match opt_upper {
            None => (Index::new(), Vec::new()),
            Some(upper) => (Index::new(), Vec::with_capacity(upper)),
        };

        let index = Cow::Owned(index);

        let series = Self { index, values };

        todo!();
    }

    /// Returns a read-only reference to the `Index` contained in this `Series`,
    /// if there is one.
    pub fn index(&self) -> &Index<K> {
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

impl<'a, K, V> Default for Series<'a, K, V>
where
    K: Label,
    V: Storable,
{
    fn default() -> Self {
        Self {
            index: Cow::Owned(Index::default()),
            values: Vec::default(),
        }
    }
}
