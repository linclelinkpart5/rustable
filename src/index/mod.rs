
pub mod iter;

use std::borrow::Borrow;
use std::cmp::Ordering;
use std::hash::Hash;
use std::iter::FromIterator;
use std::ops::Bound;
use std::ops::RangeBounds;

use indexmap::IndexSet as Set;

use crate::traits::Label;
use crate::types::DType;

use self::iter::Iter;
use self::iter::IntoIter;
use self::iter::Diff;
use self::iter::SymDiff;
use self::iter::Inter;
use self::iter::Union;

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

    pub fn iloc(&self, idx: usize) -> Option<usize> {
        // NOTE: Doing this in a weird way to reuse internal indexing logic.
        self.0.get_index(idx).map(|_| idx)
    }

    pub fn iloc_multi<I>(&self, idxs: I) -> Option<Vec<usize>>
    where
        I: IntoIterator<Item = usize>,
    {
        idxs.into_iter().map(|idx| self.iloc(idx)).collect::<Option<Vec<_>>>()
    }

    pub fn iloc_range<R>(&self, range: R) -> Option<Vec<usize>>
    where
        R: RangeBounds<usize>,
    {
        // TODO: When `rust: range_is_empty #48111` is stabilized,
        //       use that as a short circuit.
        let start_idx = match range.start_bound() {
            Bound::Included(idx) => self.iloc(*idx)?,
            Bound::Excluded(idx) => self.iloc(*idx)? + 1,
            Bound::Unbounded => 0,
        };

        let close_idx = match range.end_bound() {
            Bound::Included(idx) => self.iloc(*idx)? + 1,
            Bound::Excluded(idx) => self.iloc(*idx)?,
            Bound::Unbounded => self.len(),
        };

        self.iloc_multi(start_idx..close_idx)
    }

    pub fn bloc<A>(&self, bools: A) -> Option<Vec<usize>>
    where
        A: AsRef<[bool]>,
    {
        let bools = bools.as_ref();

        if bools.len() != self.len() { None }
        else {
            Some(
                bools
                .iter()
                .enumerate()
                .filter_map(|(i, &b)| if b { Some(i) } else { None })
                .collect()
            )
        }
    }

    pub fn loc<Q>(&self, loc: &Q) -> Option<usize>
    where
        L: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.0.get_full(loc).map(|(idx, _)| idx)
    }

    pub fn loc_multi<'a, I, Q: 'a>(&self, lbls: I) -> Option<Vec<usize>>
    where
        I: IntoIterator<Item = &'a Q>,
        L: Borrow<Q>,
        Q: Hash + Eq,
    {
        lbls.into_iter().map(|lbl| self.loc(lbl)).collect::<Option<Vec<_>>>()
    }

    pub fn loc_range<'a, R, Q: 'a>(&self, range: R) -> Option<Vec<usize>>
    where
        R: RangeBounds<&'a Q>,
        L: Borrow<Q>,
        Q: Hash + Eq,
    {
        let start_idx = match range.start_bound() {
            Bound::Included(lbl) => self.loc(*lbl)?,
            Bound::Excluded(lbl) => self.loc(*lbl)? + 1,
            Bound::Unbounded => 0,
        };

        let close_idx = match range.end_bound() {
            Bound::Included(lbl) => self.loc(*lbl)? + 1,
            Bound::Excluded(lbl) => self.loc(*lbl)?,
            Bound::Unbounded => self.len(),
        };

        self.iloc_multi(start_idx..close_idx)
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

impl<L: Label> FromIterator<L> for Index<L> {
    fn from_iter<I: IntoIterator<Item = L>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

// Handles `Index::from(&[0u32, 1, 2])`.
impl<'a, L: Label + Copy + 'a> FromIterator<&'a L> for Index<L> {
    fn from_iter<I: IntoIterator<Item = &'a L>>(iter: I) -> Self {
        Self(iter.into_iter().copied().collect())
    }
}

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

        let out = index.loc_multi(&[]);
        assert_eq!(Some(vec![]), out);

        let out = index.loc_multi(&[30]);
        assert_eq!(Some(vec![2]), out);

        let out = index.loc_multi(&[10, 20, 30, 40, 50]);
        assert_eq!(Some(vec![0, 1, 2, 3, 4]), out);

        let out = index.loc_multi(&[50, 40, 30, 20, 10]);
        assert_eq!(Some(vec![4, 3, 2, 1, 0]), out);

        let out = index.loc_multi(&[99]);
        assert_eq!(None, out);

        let out = index.loc_multi(&[99, 20, 30, 40, 50]);
        assert_eq!(None, out);

        let out = index.loc_multi(&[10, 20, 30, 40, 99]);
        assert_eq!(None, out);
    }
}
