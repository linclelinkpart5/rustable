
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
use self::iter::Diff;
use self::iter::SymDiff;
use self::iter::Inter;
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

    pub fn from_vec(vec: Vec<L>) -> Self {
        Self::from_iter(vec)
    }

    pub fn from_slice(slice: &[L]) -> Self
    where
        L: Copy,
    {
        Self::from_iter(slice)
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
        Q: Hash + Eq + ?Sized,
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
        Index::from_vec(vec)
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

    use std::collections::HashSet;

    use str_macro::str;
    use proptest::prelude::*;
    use prop::collection::hash_set;

    const MAX_NUM_LABELS: usize = 1000;

    // TODO: Figure out how to use `<L: Label>` in proptests.
    fn _arb_index<L: Label + Arbitrary>() -> impl Strategy<Value = Index<L>> {
        _arb_labels::<L>().prop_map(|c| Index::from_iter(c))
    }

    fn arb_index_i32() -> impl Strategy<Value = Index<i32>> {
        arb_labels_i32().prop_map(|c| Index::from_iter(c))
    }

    // TODO: Figure out how to use `<L: Label>` in proptests.
    fn _arb_labels<L: Label + Arbitrary>() -> impl Strategy<Value = Vec<L>> {
        hash_set(any::<L>(), 0..MAX_NUM_LABELS).prop_map(|m| m.into_iter().collect())
    }

    fn arb_labels_i32() -> impl Strategy<Value = Vec<i32>> {
        hash_set(any::<i32>(), 0..MAX_NUM_LABELS).prop_map(|m| m.into_iter().collect())
    }

    // `Index::reverse` should reverse the order of the labels in-place.
    proptest! {
        #[test]
        fn reverse_inverts_order(labels in arb_labels_i32()) {
            let mut expected = labels.clone();
            expected.reverse();

            let mut index = Index::from_vec(labels);
            index.reverse();
            let produced: Vec<_> = index.into();

            assert_eq!(produced, expected);
        }
    }

    // Calling `Index::reverse` twice should produce the original `Index`.
    proptest! {
        #[test]
        fn reverse_twice_is_identity(index in arb_index_i32()) {
            let expected = index.clone();
            let mut produced = index;
            produced.reverse();
            produced.reverse();

            assert_eq!(produced, expected);
        }
    }

    // `Index::sort` should produce the same result as `Index::sort_by` called
    // with `Ord::cmp`.
    proptest! {
        #[test]
        fn sort_as_sort_by(index in arb_index_i32()) {
            let mut produced = index.clone();
            let mut expected = index;

            produced.sort();
            expected.sort_by(Ord::cmp);

            assert_eq!(produced, expected);
        }
    }

    // `Index::sort` should produce the same result as `Index::sort_by_key` called
    // with an identity function.
    proptest! {
        #[test]
        fn sort_as_sort_by_key(index in arb_index_i32()) {
            let mut produced = index.clone();
            let mut expected = index;

            produced.sort();
            expected.sort_by_key(|&l| l);

            assert_eq!(produced, expected);
        }
    }

    // `Index::sort` should sort the `Index` in-place.
    proptest! {
        #[test]
        fn sort_orders_labels(labels in arb_labels_i32()) {
            let mut expected = labels.clone();
            expected.sort();

            let mut index = Index::from(labels);
            index.sort();
            let produced: Vec<_> = index.into();

            assert_eq!(produced, expected);
        }
    }

    // `Index::arg_sort` should produce the positions each label would be moved
    // to if `Index::sort` were to be called.
    proptest! {
        #[test]
        fn arg_sort_produces_sorted_positions(index in arb_index_i32()) {
            let produced = index.arg_sort();

            let mut pairs = index.into_iter().enumerate().collect::<Vec<_>>();
            pairs.sort_by_key(|&(_, l)| l);

            let expected = pairs.into_iter().map(|(i, _)| i).collect::<Vec<_>>();

            assert_eq!(produced, expected);
        }
    }

    // The positions produced by `Index::arg_sort` should exactly cover all
    // values from `0..len()`.
    proptest! {
        #[test]
        fn arg_sort_produces_complete_positions(index in arb_index_i32()) {
            let produced = index.arg_sort().into_iter().collect::<HashSet<_>>();
            let expected = (0usize..index.len()).collect::<HashSet<_>>();

            assert_eq!(produced, expected);
        }
    }

    // `Index::iloc` should produce `Some(&L)` if the position is in bounds, and
    // `None` otherwise.
    proptest! {
        #[test]
        fn iloc_produces_opt_label_ref(
            (labels, pos) in
                arb_labels_i32()
                .prop_flat_map(|l| {
                    let n = l.len();
                    (Just(l), 0..(2 * n))
                })
        )
        {
            let index = Index::from_vec(labels.clone());

            let expected = if pos < index.len() { Some(&labels[pos]) } else { None };
            let produced = index.iloc(pos);

            assert_eq!(produced, expected);
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

        let single = Index::from_slice(&[10]);
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

        let index = Index::from_slice(&[10]);
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
