
pub mod iter;

use std::borrow::Borrow;
use std::cmp::Ordering;
use std::hash::Hash;
use std::iter::FromIterator;
use std::ops::Bound;
use std::ops::RangeBounds;

use indexmap::IndexSet;
use is_sorted::IsSorted;

use crate::traits::Label;
use crate::types::DType;

use self::iter::Iter;
use self::iter::IntoIter;
use self::iter::Difference;
use self::iter::SymmetricDifference;
use self::iter::Intersection;
use self::iter::Union;

#[derive(Debug, Clone, Eq)]
pub struct Index<L>(IndexSet<L>)
where
    L: Label,
;

impl<L> Index<L>
where
    L: Label,
{
    pub fn new() -> Self {
        Self::default()
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

    pub fn iter(&self) -> Iter<'_, L> {
        Iter(self.0.iter())
    }

    pub fn contains<Q>(&self, label: &Q) -> bool
    where
        L: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.0.contains(label)
    }

    pub fn difference<'a>(&'a self, other: &'a Self) -> Difference<'a, L> {
        Difference::new(self, other)
    }

    pub fn symmetric_difference<'a>(&'a self, other: &'a Self) -> SymmetricDifference<'a, L> {
        SymmetricDifference::new(self, other)
    }

    pub fn intersection<'a>(&'a self, other: &'a Self) -> Intersection<'a, L> {
        Intersection::new(self, other)
    }

    pub fn union<'a>(&'a self, other: &'a Self) -> Union<'a, L> {
        Union::new(self, other)
    }

    fn to_nodule(&self, idx: &usize) -> Option<usize> {
        if idx <= &self.len() { Some(*idx) } else { None }
    }

    /// Returns the position of a given label if contained in this `Index`.
    pub fn index_of<Q>(&self, label: &Q) -> Option<usize>
    where
        L: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.0.get_index_of(label)
    }

    pub fn iloc(&self, pos: usize) -> Option<&L> {
        self.0.get_index(pos)
    }

    pub fn iloc_multi<'a, I>(&'a self, pos_iter: I) -> Option<Vec<&'a L>>
    where
        I: IntoIterator<Item = &'a usize>,
    {
        pos_iter
            .into_iter()
            .map(|&p| self.iloc(p))
            .collect::<Option<Vec<_>>>()
    }

    pub fn iloc_range<R>(&self, range: R) -> Option<Vec<&L>>
    where
        R: RangeBounds<usize>,
    {
        // NOTE: Range bounds are always validated, even if range would be empty.
        let start_nodule = match range.start_bound() {
            Bound::Included(idx) => self.to_nodule(idx)?,
            Bound::Excluded(idx) => self.to_nodule(&idx.checked_add(1)?)?,
            Bound::Unbounded => 0,
        };

        let close_nodule = match range.end_bound() {
            Bound::Included(idx) => self.to_nodule(&idx.checked_add(1)?)?,
            Bound::Excluded(idx) => self.to_nodule(idx)?,
            Bound::Unbounded => self.len(),
        };

        (start_nodule..close_nodule)
            .map(|p| self.iloc(p))
            .collect::<Option<Vec<_>>>()
    }

    pub fn bloc<'a, I, A>(&self, bools: I) -> Option<Vec<&L>>
    where
        I: IntoIterator<Item = A>,
        I::IntoIter: ExactSizeIterator,
        A: AsRef<bool>,
    {
        let bools = bools.into_iter();

        if bools.len() != self.len() { None }
        else {
            Some(
                bools
                    .zip(self.iter())
                    .filter_map(|(b, l)| if *b.as_ref() { Some(l) } else { None })
                    .collect()
            )
        }
    }

    pub fn loc<Q>(&self, label: &Q) -> Option<&L>
    where
        L: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.0.get(label)
    }

    pub fn loc_multi<'a, I, Q>(&self, labels: I) -> Option<Vec<&L>>
    where
        I: IntoIterator<Item = &'a Q>,
        L: Borrow<Q>,
        Q: 'a + Hash + Eq + ?Sized,
    {
        labels.into_iter().map(|lbl| self.loc(lbl)).collect()
    }

    pub fn loc_range<'a, R, Q>(&'a self, range: R) -> Option<Vec<&L>>
    where
        R: RangeBounds<&'a Q>,
        L: Borrow<Q>,
        Q: 'a + Hash + Eq + ?Sized,
    {
        let start_bound = match range.start_bound() {
            Bound::Included(lbl) => Bound::Included(self.index_of(lbl)?),
            Bound::Excluded(lbl) => Bound::Excluded(self.index_of(lbl)?),
            Bound::Unbounded => Bound::Unbounded,
        };

        let close_bound = match range.end_bound() {
            Bound::Included(lbl) => Bound::Included(self.index_of(lbl)?),
            Bound::Excluded(lbl) => Bound::Excluded(self.index_of(lbl)?),
            Bound::Unbounded => Bound::Unbounded,
        };

        self.iloc_range((start_bound, close_bound))
    }

    /// Reverses the order of the labels in this `Index` in-place.
    pub fn reverse(&mut self) {
        // TODO: Replace with `IndexSet::reverse()` once added.
        self.0 = self.0.drain(..).rev().collect()
    }

    /// Sorts this `Index` in-place using `Ord::cmp`.
    pub fn sort(&mut self) {
        self.sort_by(Ord::cmp)
    }

    /// Sorts this `Index` in-place using a custom comparison function.
    pub fn sort_by<F>(&mut self, compare: F)
    where
        F: FnMut(&L, &L) -> Ordering,
    {
        self.0.sort_by(compare)
    }

    /// Sorts this `Index` in-place according to a custom key function.
    pub fn sort_by_key<F, K>(&mut self, mut get_key: F)
    where
        F: FnMut(&L) -> K,
        K: Ord,
    {
        // TODO: Replace with `IndexSet::sort_by_key` if/when available.
        self.sort_by(|a, b| Ord::cmp(&get_key(a), &get_key(b)))
    }

    fn arg_sort_impl<F>(&self, mut compare: F) -> Vec<usize>
    where
        F: FnMut(&L, &L) -> Ordering,
    {
        let mut indices = (0..self.len()).collect::<Vec<_>>();

        // Sort this vector of indices, using the original labels as a lookup.
        indices.sort_by(|&a, &b| compare(&self.iloc(a).unwrap(), &self.iloc(b).unwrap()));

        indices
    }

    /// Sorts this `Index` indirectly by returning the numeric indices in sorted
    /// ascending order using `Ord::cmp`.
    pub fn arg_sort(&self) -> Vec<usize> {
        self.arg_sort_impl(Ord::cmp)
    }

    /// Sorts this `Index` indirectly by returning the numeric indices in sorted
    /// ascending order using a custom comparison function.
    pub fn arg_sort_by<F>(&self, compare: F) -> Vec<usize>
    where
        F: FnMut(&L, &L) -> Ordering,
    {
        self.arg_sort_impl(compare)
    }

    /// Sorts this `Index` indirectly by returning the numeric indices in sorted
    /// ascending order using a custom key function.
    pub fn arg_sort_by_key<F, K>(&self, mut get_key: F) -> Vec<usize>
    where
        F: FnMut(&L) -> K,
        K: Ord,
    {
        self.arg_sort_impl(|a, b| Ord::cmp(&get_key(a), &get_key(b)))
    }

    /// Returns `true` if this `Index` is sorted according to `Ord::cmp`.
    // TODO: Replace with stdlib `Iterator::is_sorted()` once stabilized.
    pub fn is_sorted(&self) -> bool {
        IsSorted::is_sorted(&mut self.iter())
    }

    /// Returns `true` if this `Index` is sorted according to a custom comparison function.
    // TODO: Replace with stdlib `Iterator::is_sorted_by()` once stabilized.
    pub fn is_sorted_by<F>(&self, mut compare: F) -> bool
    where
        F: FnMut(&L, &L) -> Ordering,
    {
        IsSorted::is_sorted_by(&mut self.iter(), |a, b| Some(compare(a, b)))
    }

    /// Returns `true` if this `Index` is sorted according to a custom key function.
    // TODO: Replace with stdlib `Iterator::is_sorted_by_key()` once stabilized.
    pub fn is_sorted_by_key<F, K>(&self, mut get_key: F) -> bool
    where
        F: FnMut(&L) -> K,
        K: Ord,
    {
        IsSorted::is_sorted_by_key(&mut self.iter(), |e| get_key(e))
    }

    /// Returns `true` if this `Index` has no labels in common with another `Index`.
    pub fn is_disjoint(&self, other: &Self) -> bool {
        Intersection::new(self, other).next().is_none()
    }

    fn subset_impl(&self, other: &Self, is_strict: bool) -> bool {
        let a_len = self.len();
        let b_len = other.len();

        let len_ok = match is_strict {
            false => a_len <= b_len,
            true => a_len < b_len,
        };

        len_ok && Difference::new(self, other).next().is_none()
    }

    /// Returns `true` if this `Index` is a subset of another `Index`.
    pub fn is_subset(&self, other: &Self) -> bool {
        self.subset_impl(other, false)
    }

    /// Returns `true` if this `Index` is a strict subset of another `Index`.
    pub fn is_strict_subset(&self, other: &Self) -> bool {
        self.subset_impl(other, true)
    }

    /// Returns `true` if this `Index` is a superset of another `Index`.
    pub fn is_superset(&self, other: &Self) -> bool {
        other.is_subset(self)
    }

    /// Returns `true` if this `Index` is a strict superset of another `Index`.
    pub fn is_strict_superset(&self, other: &Self) -> bool {
        other.is_strict_subset(self)
    }
}

impl<L> From<Index<L>> for Vec<L>
where
    L: Label,
{
    fn from(index: Index<L>) -> Self {
        index.0.into_iter().collect()
    }
}

impl<L> From<Vec<L>> for Index<L>
where
    L: Label,
{
    fn from(vec: Vec<L>) -> Self {
        Index::from_iter(vec)
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

impl<'a, L> FromIterator<&'a L> for Index<L>
where
    L: 'a + Label + Copy
{
    fn from_iter<I: IntoIterator<Item = &'a L>>(iter: I) -> Self {
        Self(iter.into_iter().copied().collect())
    }
}

impl<L> Default for Index<L>
where
    L: Label,
{
    fn default() -> Self {
        Self(IndexSet::new())
    }
}

impl<L> IntoIterator for Index<L>
where
    L: Label,
{
    type Item = L;
    type IntoIter = IntoIter<L>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self.0.into_iter())
    }
}

impl<L> PartialEq<Index<L>> for Index<L>
where
    L: Label,
{
    fn eq(&self, other: &Index<L>) -> bool {
        Iterator::eq(self.iter(), other.iter())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::testing::label::LabelGen;
    use crate::testing::index::IndexGen;

    use proptest::prelude::proptest;
    use proptest::strategy::Just;
    use proptest::strategy::Strategy;

    // `Index::reverse` should reverse the order of the labels in-place.
    proptest! {
        #[test]
        fn reverse_inverts_order(labels in LabelGen::ordered::<i32>()) {
            let mut expected = labels.clone();
            expected.reverse();

            let mut index = Index::from_iter(labels);
            Index::reverse(&mut index);
            let produced: Vec<_> = index.into();

            assert_eq!(produced, expected);
        }
    }

    // Calling `Index::reverse` twice should produce the original `Index`.
    proptest! {
        #[test]
        fn reverse_twice_is_identity(index in IndexGen::index::<i32>()) {
            let expected = index.clone();
            let mut produced = index;
            Index::reverse(&mut produced);
            Index::reverse(&mut produced);

            assert_eq!(produced, expected);
        }
    }

    // `Index::sort` should produce the same result as `Index::sort_by`
    // called with `Ord::cmp`.
    proptest! {
        #[test]
        fn sort_as_sort_by(index in IndexGen::index::<i32>()) {
            let mut produced = index.clone();
            let mut expected = index;

            Index::sort(&mut produced);
            Index::sort_by(&mut expected, Ord::cmp);

            assert_eq!(produced, expected);
        }
    }

    // `Index::sort` should produce the same result as `Index::sort_by_key`
    // called with an identity function.
    proptest! {
        #[test]
        fn sort_as_sort_by_key(index in IndexGen::index::<i32>()) {
            let mut produced = index.clone();
            let mut expected = index;

            Index::sort(&mut produced);
            Index::sort_by_key(&mut expected, |&l| l);

            assert_eq!(produced, expected);
        }
    }

    // `Index::sort` should sort the `Index` in-place.
    proptest! {
        #[test]
        fn sort_orders_labels(labels in LabelGen::ordered::<i32>()) {
            let mut expected = labels.clone();
            expected.sort();

            let mut index = Index::from(labels);
            Index::sort(&mut index);
            let produced: Vec<_> = index.into();

            assert_eq!(produced, expected);
        }
    }

    // `Index::arg_sort` should produce the positions each label would be moved
    // to if `Index::sort` were to be called.
    proptest! {
        #[test]
        fn arg_sort_produces_sorted_positions(index in IndexGen::index::<i32>()) {
            let produced = Index::arg_sort(&index);

            let mut pairs = index.into_iter().enumerate().collect::<Vec<_>>();
            pairs.sort_by_key(|&(_, l)| l);

            let expected = pairs.into_iter().map(|(i, _)| i).collect::<Vec<_>>();

            assert_eq!(produced, expected);
        }
    }

    // The positions produced by `Index::arg_sort` should exactly cover all
    // values from `0..len()` with no holes or duplicates.
    proptest! {
        #[test]
        fn arg_sort_produces_complete_positions(index in IndexGen::index::<i32>()) {
            let mut produced = Index::arg_sort(&index);
            produced.sort();
            let expected = (0usize..index.len()).collect::<Vec<_>>();

            assert_eq!(produced, expected);
        }
    }

    // `Index::iloc` should produce `Some(&L)` if the position is in bounds, and
    // `None` otherwise.
    proptest! {
        #[test]
        fn iloc_produces_opt_label_ref(
            (labels, pos) in
                LabelGen::ordered::<i32>()
                .prop_flat_map(|l| {
                    let n = l.len();
                    (Just(l), 0..(2 * n + 1))
                })
        )
        {
            let index = Index::from(labels.clone());

            let expected = if pos < index.len() { Some(&labels[pos]) } else { None };
            let produced = Index::iloc(&index, pos);

            assert_eq!(produced, expected);
        }
    }

    // `Index::is_disjoint` should return `true` if two `Index` objects if there
    // are any labels in common between them, and `false` otherwise.
    proptest! {
        #[test]
        fn is_disjoint_tests_mutual_exclusivity(
            labels_a in LabelGen::unordered::<i32>(),
            labels_b in LabelGen::unordered::<i32>(),
        )
        {
            let expected = labels_a.is_disjoint(&labels_b);

            let index_a = Index::from_iter(labels_a);
            let index_b = Index::from_iter(labels_b);

            let produced = Index::is_disjoint(&index_a, &index_b);

            assert_eq!(produced, expected);
        }
    }

    // Empty `Index` objects should be disjoint with any other `Index` object,
    // including themselves.
    proptest! {
        #[test]
        fn empty_index_always_disjoint(index in IndexGen::index::<i32>()) {
            let empty = Index::new();

            assert!(Index::is_disjoint(&empty, &index));
        }
    }

    // `Index::is_disjoint` should be symmetric.
    proptest! {
        #[test]
        fn is_disjoint_is_symmetric(
            index_a in IndexGen::index::<i32>(),
            index_b in IndexGen::index::<i32>(),
        )
        {
            assert_eq!(
                Index::is_disjoint(&index_a, &index_b),
                Index::is_disjoint(&index_b, &index_a),
            );
        }
    }

    // `Index::is_subset` should return `true` for two `Index` objects if
    // all of the labels in the first `Index` are contained in the second.
    proptest! {
        #[test]
        fn is_subset_tests_inclusion(
            labels_a in LabelGen::unordered::<i32>(),
            labels_b in LabelGen::unordered::<i32>(),
        )
        {
            let expected = labels_a.is_subset(&labels_b);

            let index_a = Index::from_iter(labels_a);
            let index_b = Index::from_iter(labels_b);

            let produced = Index::is_subset(&index_a, &index_b);

            assert_eq!(produced, expected);
        }
    }

    // Empty `Index` objects should be a subset of any other `Index` object,
    // including themselves.
    proptest! {
        #[test]
        fn empty_index_always_subset(index in IndexGen::index::<i32>()) {
            let empty = Index::new();

            assert!(Index::is_subset(&empty, &index));
        }
    }

    // `Index::is_subset` should be anti-symmetric with respect to
    // `Index::is_superset`.
    proptest! {
        #[test]
        fn is_subset_is_anti_symmetric(
            index_a in IndexGen::index::<i32>(),
            index_b in IndexGen::index::<i32>(),
        )
        {
            assert_eq!(
                Index::is_subset(&index_a, &index_b),
                Index::is_superset(&index_b, &index_a),
            );
        }
    }

    #[test]
    fn iloc() {
        let i = Index::from_iter("ideographs".chars());

        assert_eq!(i.iloc(0), Some(&'i'));
        assert_eq!(i.iloc(1), Some(&'d'));
        assert_eq!(i.iloc(2), Some(&'e'));
        assert_eq!(i.iloc(3), Some(&'o'));
        assert_eq!(i.iloc(4), Some(&'g'));
        assert_eq!(i.iloc(5), Some(&'r'));
        assert_eq!(i.iloc(6), Some(&'a'));
        assert_eq!(i.iloc(7), Some(&'p'));
        assert_eq!(i.iloc(8), Some(&'h'));
        assert_eq!(i.iloc(9), Some(&'s'));
        assert_eq!(i.iloc(42), None);
    }

    #[test]
    fn iloc_multi() {
        let i = Index::from_iter("ideographs".chars());

        assert_eq!(
            i.iloc_multi(&[]),
            Some(vec![]),
        );
        assert_eq!(
            i.iloc_multi(&[2]),
            Some(vec![&'e']),
        );
        assert_eq!(
            i.iloc_multi(&[4, 3, 3, 9, 2]),
            Some(vec![&'g', &'o', &'o', &'s', &'e']),
        );
        assert_eq!(
            i.iloc_multi(&[9, 7, 8, 2, 5, 2]),
            Some(vec![&'s', &'p', &'h', &'e', &'r', &'e']),
        );
        assert_eq!(
            i.iloc_multi(&[42]),
            None,
        );
        assert_eq!(
            i.iloc_multi(&[42, 1, 2, 3, 4]),
            None,
        );
        assert_eq!(
            i.iloc_multi(&[0, 1, 2, 3, 42]),
            None,
        );
    }

    #[test]
    fn iloc_range() {
        let i = Index::from_iter("ideographs".chars());

        // Test normal usage.

        assert_eq!(
            i.iloc_range(2..7),
            Some(vec![&'e', &'o', &'g', &'r', &'a']),
        );
        assert_eq!(
            i.iloc_range(2..=7),
            Some(vec![&'e', &'o', &'g', &'r', &'a', &'p']),
        );
        assert_eq!(
            i.iloc_range(2..),
            Some(vec![&'e', &'o', &'g', &'r', &'a', &'p', &'h', &'s']),
        );
        assert_eq!(
            i.iloc_range(..7),
            Some(vec![&'i', &'d', &'e', &'o', &'g', &'r', &'a']),
        );
        assert_eq!(
            i.iloc_range(..=7),
            Some(vec![&'i', &'d', &'e', &'o', &'g', &'r', &'a', &'p']),
        );
        assert_eq!(
            i.iloc_range(..),
            Some(vec![&'i', &'d', &'e', &'o', &'g', &'r', &'a', &'p', &'h', &'s']),
        );
        assert_eq!(
            i.iloc_range(4..5),
            Some(vec![&'g']),
        );
        assert_eq!(
            i.iloc_range(4..=5),
            Some(vec![&'g', &'r']),
        );
        assert_eq!(
            i.iloc_range(4..4),
            Some(vec![]),
        );
        assert_eq!(
            i.iloc_range(4..=4),
            Some(vec![&'g']),
        );
        assert_eq!(
            i.iloc_range(6..3),
            Some(vec![]),
        );
        assert_eq!(
            i.iloc_range(6..=3),
            Some(vec![]),
        );
        assert_eq!(
            i.iloc_range(5..4),
            Some(vec![]),
        );
        assert_eq!(
            i.iloc_range(5..=4),
            Some(vec![]),
        );
        assert_eq!(
            i.iloc_range(0..42),
            None,
        );
        assert_eq!(
            i.iloc_range(0..=42),
            None,
        );
        assert_eq!(
            i.iloc_range(42..9),
            None,
        );
        assert_eq!(
            i.iloc_range(42..=9),
            None,
        );
        assert_eq!(
            i.iloc_range(..42),
            None,
        );
        assert_eq!(
            i.iloc_range(..=42),
            None,
        );
        assert_eq!(
            i.iloc_range(42..),
            None,
        );

        // Test expected invariants.

        assert_eq!(
            i.iloc_range(..),
            i.iloc_range(0..i.len()),
        );
        assert_eq!(
            i.iloc_range(..),
            i.iloc_range(0..=9),
        );
        assert_eq!(
            i.iloc_range(..7),
            i.iloc_range(0..7),
        );
        assert_eq!(
            i.iloc_range(..=7),
            i.iloc_range(0..=7),
        );
        assert_eq!(
            i.iloc_range(2..),
            i.iloc_range(2..=9),
        );

        // Test edge cases.

        let empty: Index<char> = Index::new();
        assert_eq!(empty.iloc_range(0..0), Some(vec![]));
        assert_eq!(empty.iloc_range(0..=0), None);
        assert_eq!(empty.iloc_range(0..), Some(vec![]));
        assert_eq!(empty.iloc_range(..0), Some(vec![]));
        assert_eq!(empty.iloc_range(..=0), None);
        assert_eq!(empty.iloc_range(..), Some(vec![]));

        let single = Index::from_iter(&[10]);
        assert_eq!(single.iloc_range(0..1), Some(vec![&10]));
        assert_eq!(single.iloc_range(0..=0), Some(vec![&10]));
        assert_eq!(single.iloc_range(0..=1), None);
        assert_eq!(single.iloc_range(1..1), Some(vec![]));
        assert_eq!(single.iloc_range(1..=1), None);
        assert_eq!(single.iloc_range(2..2), None);
        assert_eq!(single.iloc_range(2..1), None);
        assert_eq!(single.iloc_range(2..=2), None);
        assert_eq!(single.iloc_range(2..=1), None);
        assert_eq!(single.iloc_range(..2), None);
        assert_eq!(single.iloc_range(..=2), None);
        assert_eq!(single.iloc_range(2..), None);
    }

    #[test]
    fn loc() {
        let i = Index::from_iter("ideographs".chars());

        assert_eq!(i.loc(&'i'), Some(&'i'));
        assert_eq!(i.loc(&'d'), Some(&'d'));
        assert_eq!(i.loc(&'e'), Some(&'e'));
        assert_eq!(i.loc(&'o'), Some(&'o'));
        assert_eq!(i.loc(&'g'), Some(&'g'));
        assert_eq!(i.loc(&'r'), Some(&'r'));
        assert_eq!(i.loc(&'a'), Some(&'a'));
        assert_eq!(i.loc(&'p'), Some(&'p'));
        assert_eq!(i.loc(&'h'), Some(&'h'));
        assert_eq!(i.loc(&'s'), Some(&'s'));
        assert_eq!(i.loc(&'x'), None);
    }

    #[test]
    fn loc_multi() {
        let i = Index::from_iter("ideographs".chars());

        assert_eq!(
            i.loc_multi(&[]),
            Some(vec![]),
        );
        assert_eq!(
            i.loc_multi(&['e']),
            Some(vec![&'e']),
        );
        assert_eq!(
            i.loc_multi(&['g', 'o', 'o', 's', 'e']),
            Some(vec![&'g', &'o', &'o', &'s', &'e']),
        );
        assert_eq!(
            i.loc_multi(&['s', 'p', 'h', 'e', 'r', 'e']),
            Some(vec![&'s', &'p', &'h', &'e', &'r', &'e']),
        );
        assert_eq!(
            i.loc_multi(&['x']),
            None,
        );
        assert_eq!(
            i.loc_multi(&['x', 'd', 'e', 'o', 'g']),
            None,
        );
        assert_eq!(
            i.loc_multi(&['i', 'd', 'e', 'o', 'x']),
            None,
        );
    }

    #[test]
    fn loc_range() {
        let i = Index::from_iter("ideographs".chars());

        // Test normal usage.

        assert_eq!(
            i.loc_range(&'e'..&'p'),
            Some(vec![&'e', &'o', &'g', &'r', &'a']),
        );
        assert_eq!(
            i.loc_range(&'e'..=&'p'),
            Some(vec![&'e', &'o', &'g', &'r', &'a', &'p']),
        );
        assert_eq!(
            i.loc_range(&'e'..),
            Some(vec![&'e', &'o', &'g', &'r', &'a', &'p', &'h', &'s']),
        );
        assert_eq!(
            i.loc_range(..&'p'),
            Some(vec![&'i', &'d', &'e', &'o', &'g', &'r', &'a']),
        );
        assert_eq!(
            i.loc_range(..=&'p'),
            Some(vec![&'i', &'d', &'e', &'o', &'g', &'r', &'a', &'p']),
        );
        assert_eq!(
            i.loc_range(..),
            Some(vec![&'i', &'d', &'e', &'o', &'g', &'r', &'a', &'p', &'h', &'s']),
        );
        assert_eq!(
            i.loc_range(&'g'..&'r'),
            Some(vec![&'g']),
        );
        assert_eq!(
            i.loc_range(&'g'..=&'r'),
            Some(vec![&'g', &'r']),
        );
        assert_eq!(
            i.loc_range(&'g'..&'g'),
            Some(vec![]),
        );
        assert_eq!(
            i.loc_range(&'g'..=&'g'),
            Some(vec![&'g']),
        );
        assert_eq!(
            i.loc_range(&'a'..&'o'),
            Some(vec![]),
        );
        assert_eq!(
            i.loc_range(&'a'..=&'o'),
            Some(vec![]),
        );
        assert_eq!(
            i.loc_range(&'r'..&'g'),
            Some(vec![]),
        );
        assert_eq!(
            i.loc_range(&'r'..=&'g'),
            Some(vec![]),
        );
        assert_eq!(
            i.loc_range(&'i'..&'x'),
            None,
        );
        assert_eq!(
            i.loc_range(&'i'..=&'x'),
            None,
        );
        assert_eq!(
            i.loc_range(&'x'..&'s'),
            None,
        );
        assert_eq!(
            i.loc_range(&'x'..=&'s'),
            None,
        );
        assert_eq!(
            i.loc_range(..&'x'),
            None,
        );
        assert_eq!(
            i.loc_range(..=&'x'),
            None,
        );
        assert_eq!(
            i.loc_range(&'x'..),
            None,
        );

        // Test expected invariants.

        assert_eq!(
            i.loc_range(..),
            i.loc_range(&'i'..=&'s'),
        );
        assert_eq!(
            i.loc_range(..),
            i.iloc_range(..),
        );
        assert_eq!(
            i.loc_range(..&'p'),
            i.loc_range(&'i'..&'p'),
        );
        assert_eq!(
            i.loc_range(..&'p'),
            i.iloc_range(..7),
        );
        assert_eq!(
            i.loc_range(..=&'p'),
            i.loc_range(&'i'..=&'p'),
        );
        assert_eq!(
            i.loc_range(..=&'p'),
            i.iloc_range(..=7),
        );
        assert_eq!(
            i.loc_range(&'e'..),
            i.loc_range(&'e'..=&'s'),
        );
        assert_eq!(
            i.loc_range(&'e'..),
            i.iloc_range(2..),
        );

        // Test edge cases.

        let empty: Index<char> = Index::new();
        assert_eq!(empty.loc_range(&'x'..&'x'), None);
        assert_eq!(empty.loc_range(&'x'..=&'x'), None);
        assert_eq!(empty.loc_range(&'x'..), None);
        assert_eq!(empty.loc_range(..&'x'), None);
        assert_eq!(empty.loc_range(..=&'x'), None);
        assert_eq!(empty.loc_range(..), Some(vec![]));

        let index = Index::from_iter(&[10]);
        assert_eq!(index.loc_range(&10..&20), None);
        assert_eq!(index.loc_range(&10..=&10), Some(vec![&10]));
        assert_eq!(index.loc_range(&10..=&20), None);
        assert_eq!(index.loc_range(&20..&20), None);
        assert_eq!(index.loc_range(&20..=&20), None);
        assert_eq!(index.loc_range(..&20), None);
        assert_eq!(index.loc_range(..=&20), None);
        assert_eq!(index.loc_range(&20..), None);

        // let i = Index::from_vec(vec![
        //     str!("ab"),
        //     str!("cd"),
        //     str!("ef"),
        //     str!("gh"),
        //     str!("ij"),
        //     str!("kl"),
        //     str!("mn"),
        //     str!("op"),
        //     str!("qr"),
        //     str!("st"),
        // ]);

        // assert_eq!(i.loc_range("ef".."op"), Some(vec![2, 3, 4, 5, 6]));
        // assert_eq!(i.loc_range("ef"..="op"), Some(vec![2, 3, 4, 5, 6, 7]));
        // assert_eq!(i.loc_range("ef"..), Some(vec![2, 3, 4, 5, 6, 7, 8, 9]));
        // assert_eq!(i.loc_range(.."op"), Some(vec![0, 1, 2, 3, 4, 5, 6]));
        // assert_eq!(i.loc_range(..="op"), Some(vec![0, 1, 2, 3, 4, 5, 6, 7]));
        // assert_eq!(i.loc_range::<_, str>(..), Some(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]));
        // assert_eq!(i.loc_range("ij".."kl"), Some(vec![4]));
        // assert_eq!(i.loc_range("ij"..="kl"), Some(vec![4, 5]));
        // assert_eq!(i.loc_range("ij".."ij"), Some(vec![]));
        // assert_eq!(i.loc_range("ij"..="ij"), Some(vec![4]));
        // assert_eq!(i.loc_range("mn".."gh"), Some(vec![]));
        // assert_eq!(i.loc_range("mn"..="gh"), Some(vec![]));
        // assert_eq!(i.loc_range("kl".."ij"), Some(vec![]));
        // assert_eq!(i.loc_range("kl"..="ij"), Some(vec![]));
        // assert_eq!(i.loc_range("ab".."??"), None);
        // assert_eq!(i.loc_range("ab"..="??"), None);
        // assert_eq!(i.loc_range("??".."ab"), None);
        // assert_eq!(i.loc_range("??"..="ab"), None);
        // assert_eq!(i.loc_range(.."??"), None);
        // assert_eq!(i.loc_range(..="??"), None);
        // assert_eq!(i.loc_range("??"..), None);

        // assert_eq!(i.loc_range::<_, str>(..), i.iloc_range(&0..&i.len()));
        // assert_eq!(i.loc_range(.."op"), i.iloc_range(..&7));
        // assert_eq!(i.loc_range(..="op"), i.iloc_range(..=&7));
        // assert_eq!(i.loc_range("ef"..), i.iloc_range(&2..));

        // assert_eq!(i.loc_range::<_, str>(..), i.loc_range("ab"..="st"));
        // assert_eq!(i.loc_range(.."op"), i.loc_range("ab".."op"));
        // assert_eq!(i.loc_range(..="op"), i.loc_range("ab"..="op"));
        // assert_eq!(i.loc_range("ef"..), i.loc_range("ef"..="st"));
    }
}
