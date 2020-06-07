
pub mod iter;

use std::borrow::Borrow;
use std::cmp::Ordering;
use std::hash::Hash;
use std::iter::FromIterator;
use std::ops::Bound;
use std::ops::RangeBounds;

use indexmap::IndexSet;

use crate::traits::Label;
use crate::types::DType;

use self::iter::Iter;
use self::iter::IntoIter;
use self::iter::Diff;
use self::iter::SymDiff;
use self::iter::Inter;
use self::iter::Union;

#[derive(Debug, Clone)]
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

    /// Sorts this `Index` in ascending order.
    pub fn sort(&mut self) {
        self.sort_by(Ord::cmp)
    }

    /// Sorts this `Index` according to a custom comparator function.
    pub fn sort_by<F>(&mut self, compare: F)
    where
        F: FnMut(&L, &L) -> Ordering,
    {
        self.0.sort_by(compare)
    }

    /// Sorts this `Index` according to a custom key function.
    pub fn sort_by_key<F, K>(&mut self, mut get_key: F)
    where
        F: FnMut(&L) -> K,
        K: Ord,
    {
        self.sort_by(|a, b| Ord::cmp(&get_key(a), &get_key(b)))
    }

    /// Sorts this `Index` indirectly by returning the numeric indices in sorted
    /// ascending order.
    pub fn arg_sort(&self) -> Vec<usize> {
        let mut indices = (0..self.len()).collect::<Vec<_>>();

        // Sort this vector of indices, using the original labels as a lookup.
        indices.sort_by_key(|&i| self.0.get_index(i).unwrap());

        indices
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

#[cfg(test)]
mod tests {
    use super::*;

    use str_macro::str;

    #[test]
    fn sort() {
        let mut i = Index::from_vec(vec![9, 5, 3, 8, 6, 0, 1, 2, 7, 4]);
        i.sort();
        assert_eq!(i.into_iter().collect::<Vec<_>>(), vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

        let mut i = Index::from_vec(vec![
            str!("cam"),
            str!("ben"),
            str!("hal"),
            str!("eli"),
            str!("ida"),
            str!("jim"),
            str!("amy"),
            str!("dee"),
            str!("gus"),
            str!("fay"),
        ]);
        i.sort();
        assert_eq!(i.into_iter().collect::<Vec<_>>(), vec![
            str!("amy"),
            str!("ben"),
            str!("cam"),
            str!("dee"),
            str!("eli"),
            str!("fay"),
            str!("gus"),
            str!("hal"),
            str!("ida"),
            str!("jim"),
        ]);
    }

    #[test]
    fn sort_by() {
        let mut i = Index::from_vec(vec![9, 5, 3, 8, 6, 0, 1, 2, 7, 4]);
        i.sort_by(|a, b| Ord::cmp(&(a % 5), &(b % 5)));
        assert_eq!(i.into_iter().collect::<Vec<_>>(), vec![5, 0, 6, 1, 2, 7, 3, 8, 9, 4]);

        let mut i = Index::from_vec(vec![
            str!("cam"),
            str!("ben"),
            str!("hal"),
            str!("eli"),
            str!("ida"),
            str!("jim"),
            str!("amy"),
            str!("dee"),
            str!("gus"),
            str!("fay"),
        ]);
        i.sort_by(|a, b| a.chars().rev().cmp(b.chars().rev()));
        assert_eq!(i.into_iter().collect::<Vec<_>>(), vec![
            str!("ida"),
            str!("dee"),
            str!("eli"),
            str!("hal"),
            str!("cam"),
            str!("jim"),
            str!("ben"),
            str!("gus"),
            str!("fay"),
            str!("amy"),
        ]);
    }

    #[test]
    fn sort_by_key() {
        let mut i = Index::from_vec(vec![9, 5, 3, 8, 6, 0, 1, 2, 7, 4]);
        i.sort_by_key(|i| (5i32 - i).abs());
        assert_eq!(i.into_iter().collect::<Vec<_>>(), vec![5, 6, 4, 3, 7, 8, 2, 9, 1, 0]);

        let mut i = Index::from_vec(vec![
            str!("cam"),
            str!("ben"),
            str!("hal"),
            str!("eli"),
            str!("ida"),
            str!("jim"),
            str!("amy"),
            str!("dee"),
            str!("gus"),
            str!("fay"),
        ]);
        i.sort_by_key(|s| s.chars().nth(1));
        assert_eq!(i.into_iter().collect::<Vec<_>>(), vec![
            str!("cam"),
            str!("hal"),
            str!("fay"),
            str!("ida"),
            str!("ben"),
            str!("dee"),
            str!("jim"),
            str!("eli"),
            str!("amy"),
            str!("gus"),
        ]);
    }

    #[test]
    fn arg_sort() {
        use std::collections::HashSet;

        let i = Index::from_vec(vec![9, 5, 3, 8, 6, 0, 1, 2, 7, 4]);
        let res = i.arg_sort();
        assert_eq!(res.iter().min(), Some(&0));
        assert_eq!(res.iter().max(), Some(&(i.len() - 1)));
        assert_eq!(res.iter().collect::<HashSet<_>>().len(), i.len());
        assert_eq!(res, vec![5, 6, 7, 2, 9, 1, 4, 8, 3, 0]);
        assert_eq!(i.iloc_multi(&res), Some(vec![&0, &1, &2, &3, &4, &5, &6, &7, &8, &9]));

        let i = Index::from_vec(vec![
            str!("cam"),
            str!("ben"),
            str!("hal"),
            str!("eli"),
            str!("ida"),
            str!("jim"),
            str!("amy"),
            str!("dee"),
            str!("gus"),
            str!("fay"),
        ]);
        let res = i.arg_sort();
        assert_eq!(res.iter().min(), Some(&0));
        assert_eq!(res.iter().max(), Some(&(i.len() - 1)));
        assert_eq!(res.iter().collect::<HashSet<_>>().len(), i.len());
        assert_eq!(res, vec![6, 1, 0, 7, 3, 9, 8, 2, 4, 5]);
        assert_eq!(i.iloc_multi(&res), Some(vec![
            &str!("amy"),
            &str!("ben"),
            &str!("cam"),
            &str!("dee"),
            &str!("eli"),
            &str!("fay"),
            &str!("gus"),
            &str!("hal"),
            &str!("ida"),
            &str!("jim"),
        ]));
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
