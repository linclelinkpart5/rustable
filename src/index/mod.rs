
pub mod iter;

use std::borrow::Borrow;
use std::cmp::Ordering;
use std::hash::Hash;

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

    pub fn dtype(&self) -> DType {
        L::dtype()
    }

    // TODO: Perhaps have this error when given duplicated keys?
    pub fn from_labels<I: IntoIterator<Item = L>>(iter: I) -> Self {
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

    pub fn loc<Q>(&self, loc: &Q) -> Option<usize>
    where
        L: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.0.get_full(loc).map(|(index, _)| index)
    }

    pub fn loc_multi<Q, A>(&self, locs: &A) -> Option<Vec<usize>>
    where
        L: Borrow<Q>,
        Q: Hash + Eq,
        A: AsRef<[Q]>,
    {
        let locs = locs.as_ref();

        let mut results = Vec::new();
        for loc in locs {
            results.push(self.loc(loc)?);
        }

        Some(results)
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
