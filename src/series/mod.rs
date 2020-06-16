
pub mod error;

use std::borrow::Borrow;
use std::borrow::Cow;
use std::hash::Hash;

use crate::traits::Storable;
use crate::traits::Label;
use crate::index::Index;

use self::error::DuplicateIndexLabel;

#[derive(Debug)]
pub struct Series<'a, K, V>(Cow<'a, Index<K>>, Vec<V>)
where
    K: Label,
    V: Storable,
;

impl<'a, K, V> Series<'a, K, V>
where
    K: Label,
    V: Storable,
{
    /// Creates a new, empty `Series` with no values and an empty `Index`.
    pub fn new() -> Self {
        Self::default()
    }

    fn new_inner(index: Cow<'a, Index<K>>, values: Vec<V>) -> Self {
        assert_eq!(index.len(), values.len());

        Self(index, values)
    }

    /// Creates a new `Series` from an iterable of index label and value pairs.
    /// If duplicated index labels are encountered, a `DuplicateIndexLabel`
    /// error is returned.
    pub fn from_pairs<I>(pairs: I) -> Result<Self, DuplicateIndexLabel<K>>
    where
        I: IntoIterator<Item = (K, V)>,
    {
        let pairs = pairs.into_iter();

        // Use `Iterator::size_hint` to try and pre-allocate.
        let (_, opt_upper_len) = pairs.size_hint();

        let (mut index, mut values) = match opt_upper_len {
            None => (Index::new(), Vec::new()),
            Some(upper_len) => (
                Index::with_capacity(upper_len),
                Vec::with_capacity(upper_len),
            ),
        };

        // Add in all of the pairs.
        for (label, value) in pairs {
            // Report an error if a duplicated label is found.
            if index.contains(&label) {
                // Need to use `return`, as using `?` moves the `label` var.
                return Err(DuplicateIndexLabel { label });
            }

            index.push(label);
            values.push(value);
        }

        Ok(Self::new_inner(Cow::Owned(index), values))
    }

    /// Returns a read-only reference to the `Index` contained in this `Series`.
    pub fn index(&self) -> &Index<K> {
        self.0.as_ref()
    }

    /// Returns a read-only slice of the value data store of this `Series`.
    pub fn values(&self) -> &[V] {
        &self.1
    }

    /// Returns a mutable slice of the value data store of this `Series`.
    pub fn values_mut(&mut self) -> &mut [V] {
        &mut self.1
    }

    /// Consumes the `Series` and returns the value data store.
    pub fn into_values(self) -> Vec<V> {
        self.1
    }

    /// Given a key, returns a read-only reference to its value in the `Series`,
    /// if it exists.
    pub fn loc<Q>(&self, label: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.0.index_of(&label).and_then(move |pos| self.1.get(pos))
    }

    /// Given a key, returns a mutable reference to its value in the `Series`,
    /// if it exists.
    pub fn loc_mut<Q>(&mut self, label: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.0.index_of(&label).and_then(move |pos| self.1.get_mut(pos))
    }
}

impl<'a, K, V> Default for Series<'a, K, V>
where
    K: Label,
    V: Storable,
{
    fn default() -> Self {
        Self(Cow::Owned(Index::default()), Vec::default())
    }
}
