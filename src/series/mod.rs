
pub mod error;
pub mod iter;
pub mod values;

use std::borrow::Borrow;
use std::collections::HashSet;
use std::hash::Hash;
use std::iter::FromIterator;

use indexmap::IndexMap;

use crate::index::Index;
use crate::traits::Storable;
use crate::traits::Label;
use crate::traits::RawType;

pub use self::error::DuplicateIndexLabel;
pub use self::error::LengthMismatch;
pub use self::error::OverlappingIndex;
pub use self::iter::Iter;
pub use self::iter::IterMut;
pub use self::iter::IntoIter;

#[derive(Debug)]
pub struct Series<L: Label, V: Storable>(
    pub(crate) Index<L>,
    pub(crate) Vec<V>,
);

impl<L, V> Series<L, V>
where
    L: Label,
    V: Storable,
{
    /// Creates a new, empty `Series` with no values and an empty `Index`.
    pub fn new() -> Self {
        Self::default()
    }

    fn assert_len(&self) {
        assert_eq!(self.0.len(), self.1.len());
    }

    fn new_inner(index: Index<L>, values: Vec<V>) -> Self {
        let new = Self(index, values);
        new.assert_len();
        new
    }

    /// Creates a new `Series` from an iterable of index label/value pairs.
    /// If duplicated index labels are encountered, a `DuplicateIndexLabel`
    /// error is returned.
    pub fn from_iter_checked<I>(iter: I) -> Result<Self, DuplicateIndexLabel<L>>
    where
        I: IntoIterator<Item = (L, V)>,
    {
        let pairs = iter.into_iter();

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

        Ok(Self::new_inner(index, values))
    }

    pub fn from_values(index: Index<L>, values: Vec<V>) -> Result<Self, LengthMismatch<L, V>> {
        // Check if the lengths of the index and values are the same.
        if index.len() != values.len() {
            return Err(LengthMismatch { index, values });
        } else {
            Ok(Self::new_inner(index, values))
        }
    }

    /// Returns a read-only reference to the `Index` of this `Series`.
    pub fn index(&self) -> &Index<L> {
        &self.0
    }

    /// Returns a read-only slice of the values in this `Series`.
    pub fn values(&self) -> &[V] {
        &self.1
    }

    /// Returns a mutable slice of the values in this `Series`.
    pub fn values_mut(&mut self) -> &mut [V] {
        &mut self.1
    }

    /// Consumes the `Series` and returns its `Index`.
    pub fn into_index(self) -> Index<L> {
        self.into_index_values().0
    }

    /// Consumes the `Series` and returns its values.
    pub fn into_values(self) -> Vec<V> {
        self.into_index_values().1
    }

    /// Consumes the `Series` and returns its `Index` and its values.
    pub fn into_index_values(self) -> (Index<L>, Vec<V>) {
        (self.0, self.1)
    }

    /// Clears all label/value pairs from this `Series`.
    pub fn clear(&mut self) {
        self.0.clear();
        self.1.clear();
    }

    /// Returns `true` if this `Series` contains no label/value pairs.
    pub fn is_empty(&self) -> bool {
        self.assert_len();
        self.0.is_empty()
    }

    /// Returns `true` if this `Series` contains the specified label.
    pub fn contains_label<Q>(&self, label: &Q) -> bool
    where
        L: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.0.contains(label)
    }

    /// Given a position, returns a read-only reference to its value in the
    /// `Series`, if it exists.
    pub fn iloc(&self, pos: usize) -> Option<&V> {
        self.1.get(pos)
    }

    /// Given a position, returns a mutable reference to its value in the
    /// `Series`, if it exists.
    pub fn iloc_mut(&mut self, pos: usize) -> Option<&mut V> {
        self.1.get_mut(pos)
    }

    /// Given a label, returns a read-only reference to its value in the
    /// `Series`, if it exists.
    pub fn loc<Q>(&self, label: &Q) -> Option<&V>
    where
        L: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.0.index_of(&label).and_then(move |pos| self.1.get(pos))
    }

    /// Given a label, returns a mutable reference to its value in the
    /// `Series`, if it exists.
    pub fn loc_mut<Q>(&mut self, label: &Q) -> Option<&mut V>
    where
        L: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.0.index_of(&label).and_then(move |pos| self.1.get_mut(pos))
    }

    /// Returns an iterator that yields all label/value pairs in this `Series`
    /// in order.
    pub fn iter(&self) -> Iter<L, V> {
        Iter::new(self)
    }

    /// Returns an iterator that yields all label/value pairs in this `Series`
    /// in order, with mutable references to the values.
    pub fn iter_mut(&mut self) -> IterMut<L, V> {
        IterMut::new(self)
    }

    /// Retains only the label/value pairs specified by the predicate.
    /// The predicate accepts references to a label and a value.
    pub fn retain<F>(&mut self, mut pred: F)
    where
        F: FnMut(&L, &V) -> bool,
    {
        let pos_to_drop =
            self
            .iter()
            .enumerate()
            .filter(|(_, (l, v))| pred(l, v))
            .map(|(p, _)| p)
            .collect::<HashSet<_>>()
        ;

        // Only do work if there are any pairs to drop.
        if !pos_to_drop.is_empty() {
            let mut p = 0usize;
            self.0.retain(|_| { (!pos_to_drop.contains(&p), p += 1).0 });

            let mut p = 0usize;
            self.1.retain(|_| { (!pos_to_drop.contains(&p), p += 1).0 });
        }

        // Assert that the index and value lengths are the same.
        self.assert_len();
    }

    /// Retains only the label/value pairs specified by the predicate.
    /// The predicate accepts a reference to a label.
    pub fn retain_labels<F>(&mut self, mut pred: F)
    where
        F: FnMut(&L) -> bool,
    {
        self.retain(|l, _| pred(l));
    }

    /// Retains only the label/value pairs specified by the predicate.
    /// The predicate accepts a reference to a value.
    pub fn retain_values<F>(&mut self, mut pred: F)
    where
        F: FnMut(&V) -> bool,
    {
        self.retain(|_, v| pred(v));
    }

    /// Applies a function to each value in this `Series`, and produces a new
    /// `Series` with transformed values.
    pub fn map<F, C>(self, map_func: F) -> Series<L, C>
    where
        F: FnMut(V) -> C,
        C: Storable,
    {
        let (index, values) = (self.0, self.1);

        let mapped_values =
            values
            .into_iter()
            .map(map_func)
            .collect()
        ;

        Series::new_inner(index, mapped_values)
    }

    pub fn concat_checked(self, other: Self) -> Result<Self, OverlappingIndex> {
        Ok(self)
    }
}

impl<L: Label, R: RawType + Storable> Series<L, Option<R>> {
    fn fill_handler<F>(self, fill_func: F) -> Series<L, R>
    where
        F: FnMut(Option<R>) -> R,
    {
        let (index, values) = (self.0, self.1);

        // NOTE: This should preserve the number and order of values!
        let filled_values =
            values
            .into_iter()
            .map(fill_func)
            .collect()
        ;

        Series::new_inner(index, filled_values)
    }

    /// Consumes a `Series` containing `Option` values, fills `None`s with the
    /// given value, and returns a new `Series` without `None`s.
    pub fn fill_none(self, value: R) -> Series<L, R> {
        self.fill_handler(|v| v.unwrap_or_else(|| value.clone()))
    }

    /// Consumes a `Series` containing `Option` values, fills `None`s with the
    /// result of the given function, and returns a new `Series` without `None`s.
    pub fn fill_none_with<F>(self, mut func: F) -> Series<L, R>
    where
        F: FnMut() -> R,
    {
        self.fill_handler(|v| v.unwrap_or_else(|| func()))
    }

    /// Consumes a `Series` containing `Option` values, drops the label/value
    /// pairs with a value of `None`, and returns a new `Series` without `None`s.
    pub fn drop_none(self) -> Series<L, R> {
        // NOTE: The `Index` may not need to be modified if no values end up
        //       getting dropped. The values will always need to be modified.
        let mut index = self.0;
        let values = self.1;

        let mut pos_to_drop = HashSet::new();
        let mut raw_values = Vec::<R>::with_capacity(values.len());

        for (pos, value) in values.into_iter().enumerate() {
            match value {
                Some(rv) => { raw_values.push(rv); },
                None => { pos_to_drop.insert(pos); }
            }
        }

        // Only mutate the `Index` if there are any elements to drop.
        if !pos_to_drop.is_empty() {
            let mut p = 0usize;
            index.retain(|_| { (!pos_to_drop.contains(&p), p += 1).0 });
        }

        Series::new_inner(index, raw_values)
    }
}

impl<L, V> Default for Series<L, V>
where
    L: Label,
    V: Storable,
{
    fn default() -> Self {
        Self(Index::default(), Vec::default())
    }
}

impl<L, V> IntoIterator for Series<L, V>
where
    L: Label,
    V: Storable,
{
    type Item = (L, V);
    type IntoIter = IntoIter<L, V>;

    /// Returns an iterator that consumes this `Series` and yields all
    /// label/value pairs in order.
    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self)
    }
}

impl<L, V> FromIterator<(L, V)> for Series<L, V>
where
    L: Label,
    V: Storable,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (L, V)>,
    {
        let index_map: IndexMap<L, V> = IndexMap::from_iter(iter);

        let mut index = Index::with_capacity(index_map.len());
        let mut values = Vec::with_capacity(index_map.len());

        for (l, v) in index_map {
            index.push(l);
            values.push(v);
        }

        Self::new_inner(index, values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::iter::FromIterator;

    #[test]
    fn fill_none() {
        let s = Series::from_iter_checked(vec![
            (0, Some('a')),
            (1, None),
            (2, Some('b')),
            (3, Some('c')),
            (4, None),
            (5, None),
            (6, Some('d')),
            (7, Some('e')),
            (8, None),
            (9, Some('f')),
        ]).unwrap();

        let filled_s = s.fill_none('x');

        let (index, values) = filled_s.into_index_values();

        assert_eq!(index, Index::from_iter(0..=9));
        assert_eq!(values, vec!['a', 'x', 'b', 'c', 'x', 'x', 'd', 'e', 'x', 'f']);
    }

    #[test]
    fn fill_none_with() {
        let mut caps = false;
        let fill_func = move || {
            let ch = if caps { 'X' } else { 'x' };
            caps = !caps;
            ch
        };

        let s = Series::from_iter_checked(vec![
            (0, Some('a')),
            (1, None),
            (2, Some('b')),
            (3, Some('c')),
            (4, None),
            (5, None),
            (6, Some('d')),
            (7, Some('e')),
            (8, None),
            (9, Some('f')),
        ]).unwrap();

        let filled_s = s.fill_none_with(fill_func);

        let (index, values) = filled_s.into_index_values();

        assert_eq!(index, Index::from_iter(0..=9));
        assert_eq!(values, vec!['a', 'x', 'b', 'c', 'X', 'x', 'd', 'e', 'X', 'f']);
    }

    #[test]
    fn drop_none() {
        let s = Series::from_iter_checked(vec![
            (0, Some('a')),
            (1, None),
            (2, Some('b')),
            (3, Some('c')),
            (4, None),
            (5, None),
            (6, Some('d')),
            (7, Some('e')),
            (8, None),
            (9, Some('f')),
        ]).unwrap();

        let dropped_s = s.drop_none();

        let (index, values) = dropped_s.into_index_values();

        assert_eq!(index, Index::from_iter(&[0, 2, 3, 6, 7, 9]));
        assert_eq!(values, vec!['a', 'b', 'c', 'd', 'e', 'f']);
    }
}
