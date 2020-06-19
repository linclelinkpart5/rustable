
pub mod error;
pub mod iter;
pub mod values;

use std::borrow::Borrow;
use std::borrow::Cow;
use std::collections::HashSet;
use std::hash::Hash;

use crate::index::Index;
use crate::traits::Storable;
use crate::traits::Label;
use crate::traits::RawType;

pub use self::error::DuplicateIndexLabel;
pub use self::iter::Iter;
pub use self::iter::IterMut;

#[derive(Debug)]
pub struct Series<'a, L: Label, V: Storable>(
    pub(crate) Cow<'a, Index<L>>,
    pub(crate) Cow<'a, [V]>,
);

impl<'a, L, V> Series<'a, L, V>
where
    L: Label,
    V: Storable,
{
    /// Creates a new, empty `Series` with no values and an empty `Index`.
    pub fn new() -> Self {
        Self::default()
    }

    fn invariant(&self) {
        assert_eq!(self.0.len(), self.1.len());
    }

    fn new_inner(index: Cow<'a, Index<L>>, values: Cow<'a, [V]>) -> Self {
        let new = Self(index, values);
        new.invariant();
        new
    }

    /// Creates a new `Series` from an iterable of index label/value pairs.
    /// If duplicated index labels are encountered, a `DuplicateIndexLabel`
    /// error is returned.
    pub fn from_pairs<I>(pairs: I) -> Result<Self, DuplicateIndexLabel<L>>
    where
        I: IntoIterator<Item = (L, V)>,
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

        Ok(Self::new_inner(Cow::Owned(index), Cow::Owned(values)))
    }

    /// Returns a read-only reference to the `Index` of this `Series`.
    pub fn index(&self) -> &Index<L> {
        self.0.as_ref()
    }

    /// Returns a read-only slice of the values in this `Series`.
    pub fn values(&self) -> &[V] {
        &self.1
    }

    /// Returns a mutable slice of the values in this `Series`.
    pub fn values_mut(&mut self) -> &mut [V] {
        self.1.to_mut()
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
        (self.0.into_owned(), self.1.into_owned())
    }

    /// Clears all label/value pairs from this `Series`.
    pub fn clear(&mut self) {
        self.0.to_mut().clear();
        self.1.to_mut().clear();
    }

    /// Returns `true` if this `Series` contains no label/value pairs.
    pub fn is_empty(&self) -> bool {
        self.invariant();
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
        self.0.index_of(&label).and_then(move |pos| self.1.to_mut().get_mut(pos))
    }

    // Returns an iterator visiting all label/value pairs in this `Series`
    // in order.
    pub fn iter(&'a self) -> Iter<'a, L, V> {
        Iter::new(self)
    }

    // Returns an iterator visiting all label/value pairs in this `Series`
    // in order, with mutable references to the values.
    pub fn iter_mut(&'a mut self) -> IterMut<'a, L, V> {
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
            self.0.to_mut().retain(|_| { (pos_to_drop.contains(&p), p += 1).0 });

            let mut p = 0usize;
            self.1.to_mut().retain(|_| { (pos_to_drop.contains(&p), p += 1).0 });
        }

        // Assert that the index and value vector lengths are the same.
        self.invariant();
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
}

impl<'a, L: Label, R: RawType + Storable> Series<'a, L, Option<R>> {
    fn fill_handler<F>(self, fill_func: F) -> Series<'a, L, R>
    where
        F: FnMut(Option<R>) -> R,
    {
        let (index, values) = (self.0, self.1.into_owned());

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
    pub fn fill_none(self, value: R) -> Series<'a, L, R> {
        self.fill_handler(|v| v.unwrap_or_else(|| value.clone()))
    }

    /// Consumes a `Series` containing `Option` values, fills `None`s with the
    /// result of the given function, and returns a new `Series` without `None`s.
    pub fn fill_none_with<F>(self, func: F) -> Series<'a, L, R>
    where
        F: Fn() -> R,
    {
        self.fill_handler(|v| v.unwrap_or_else(|| func()))
    }

    /// Consumes a `Series` containing `Option` values, drops the label/value
    /// pairs with a value of `None`, and returns a new `Series` without `None`s.
    pub fn drop_none(self) -> Series<'a, L, R> {
        todo!("Need to also drop the corresponding labels from the index")
    }
}

impl<'a, L, V> Default for Series<'a, L, V>
where
    L: Label,
    V: Storable,
{
    fn default() -> Self {
        Self(Cow::Owned(Index::default()), Cow::Owned(Vec::default()))
    }
}

