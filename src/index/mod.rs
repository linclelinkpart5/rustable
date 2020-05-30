
pub mod iter;

use std::borrow::Borrow;
use std::cmp::Ordering;
use std::hash::Hash;
use std::iter::FromIterator;

use indexmap::IndexSet as Set;

use crate::traits::Label;
use crate::types::DType;

use self::iter::Iter;
use self::iter::IntoIter;
use self::iter::Diff;
use self::iter::SymDiff;
use self::iter::Inter;
use self::iter::Union;
use self::iter::LocMulti;

#[derive(Debug, Clone)]
pub struct Index<L: Label>(Set<L>);

impl<L: Label> Index<L> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_vec(vec: Vec<L>) -> Self {
        Self::from_iter(vec)
    }

    pub fn dtype(&self) -> DType {
        L::dtype()
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

    pub fn push(&mut self, key: L) -> bool {
        self.0.insert(key)
    }

    // NOTE: This will always be in "iloc" order, so no need to provide a "full"
    //       flavor of this method.
    pub fn iter(&self) -> Iter<'_, L> {
        Iter(self.0.iter())
    }

    pub fn contains<Q>(&self, label: &Q) -> bool
    where
        L: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.0.contains(label)
    }

    pub fn diff<'a>(&'a self, other: &'a Self) -> Diff<'a, L> {
        Diff::new(self, other)
    }

    pub fn sym_diff<'a>(&'a self, other: &'a Self) -> SymDiff<'a, L> {
        SymDiff::new(self, other)
    }

    pub fn inter<'a>(&'a self, other: &'a Self) -> Inter<'a, L> {
        Inter::new(self, other)
    }

    pub fn union<'a>(&'a self, other: &'a Self) -> Union<'a, L> {
        Union::new(self, other)
    }

    pub fn sort(&mut self) {
        self.0.sort()
    }

    pub fn sort_by<F: FnMut(&L, &L) -> Ordering>(&mut self, compare: F) {
        self.0.sort_by(compare)
    }

    pub fn iloc(&self, iloc: usize) -> Option<usize> {
        if iloc < self.0.len() { Some(iloc) } else { None }
    }

    pub fn loc<Q>(&self, loc: &Q) -> Option<usize>
    where
        L: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.0.get_full(loc).map(|(index, _)| index)
    }

    pub fn loc_multi<'a, I, Q>(&'a self, locs: I) -> LocMulti<'a, I::IntoIter, L, Q>
    where
        I: IntoIterator<Item = &'a Q>,
        L: Borrow<Q>,
        Q: Hash + Eq,
    {
        LocMulti::new(locs.into_iter(), self)
    }
}

// Handles loading from `&[0u32, 1, 2]`.
impl<'a, L> Index<L>
where
    L: Label + Copy,
{
    pub fn from_slice(slice: &[L]) -> Self {
        Self::from_iter(slice)
    }
}

impl<L> FromIterator<L> for Index<L>
where
    L: Label,
{
    fn from_iter<I: IntoIterator<Item = L>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

// Handles `Index::from(&[0u32, 1, 2])`.
impl<'a, L: 'a> FromIterator<&'a L> for Index<L>
where
    L: Label + Copy,
{
    fn from_iter<I: IntoIterator<Item = &'a L>>(iter: I) -> Self {
        Self(iter.into_iter().copied().collect())
    }
}

// NOTE: This is needed because `#[derive(Default)]` only works if the type `L`
// is also `Default`, for some reason!
impl<L: Label> Default for Index<L> {
    fn default() -> Self {
        Self(Set::new())
    }
}

impl<L: Label> IntoIterator for Index<L> {
    type Item = L;
    type IntoIter = IntoIter<L>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self.0.into_iter())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loc_multi() {
        let index = Index::from_iter(&[10u32, 20, 30, 40, 50]);

        let mut iter = index.loc_multi(&[]);
        assert_eq!(iter.next(), None);

        let mut iter = index.loc_multi(&[10, 20, 30, 40, 50]);
        assert_eq!(iter.next(), Some(Some(0)));
        assert_eq!(iter.next(), Some(Some(1)));
        assert_eq!(iter.next(), Some(Some(2)));
        assert_eq!(iter.next(), Some(Some(3)));
        assert_eq!(iter.next(), Some(Some(4)));
        assert_eq!(iter.next(), None);

        let mut iter = index.loc_multi(&[50, 40, 30, 20, 10]);
        assert_eq!(iter.next(), Some(Some(4)));
        assert_eq!(iter.next(), Some(Some(3)));
        assert_eq!(iter.next(), Some(Some(2)));
        assert_eq!(iter.next(), Some(Some(1)));
        assert_eq!(iter.next(), Some(Some(0)));
        assert_eq!(iter.next(), None);

        let mut iter = index.loc_multi(&[10, 20, 99, 40, 88]);
        assert_eq!(iter.next(), Some(Some(0)));
        assert_eq!(iter.next(), Some(Some(1)));
        assert_eq!(iter.next(), Some(None));
        assert_eq!(iter.next(), Some(Some(3)));
        assert_eq!(iter.next(), Some(None));
        assert_eq!(iter.next(), None);
    }
}
