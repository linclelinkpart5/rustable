
pub mod iter;

use std::borrow::Borrow;
use std::cmp::Ordering;
use std::hash::Hash;
use std::iter::FromIterator;
use std::ops::Bound;
use std::ops::RangeBounds;
use std::ops::Range;
use std::ops::RangeFrom;
use std::ops::RangeTo;
use std::ops::RangeInclusive;
use std::ops::RangeToInclusive;
use std::ops::RangeFull;

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
pub struct Index<L: Label>(IndexSet<L>);

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

    pub fn sort(&mut self) {
        self.0.sort()
    }

    pub fn sort_by<F: FnMut(&L, &L) -> Ordering>(&mut self, compare: F) {
        self.0.sort_by(compare)
    }

    pub fn iloc(&self, idx: usize) -> Option<usize> {
        if idx < self.0.len() { Some(idx) } else { None }
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
        // TODO: ALWAYS VALIDATE BOUNDS, EVEN IF RANGE WOULD BE EMPTY.
        // TODO: When `rust: range_is_empty #48111` is stabilized,
        //       use that as a short circuit.
        let start_idx = match range.start_bound() {
            Bound::Included(idx) => Some(*idx).filter(|x| x < &self.len())?,
            Bound::Excluded(idx) => Some(*idx).filter(|x| x <= &self.len())? + 1,
            Bound::Unbounded => 0,
        };

        let close_idx = match range.end_bound() {
            Bound::Included(idx) => Some(*idx).filter(|x| x < &self.len())? + 1,
            Bound::Excluded(idx) => Some(*idx).filter(|x| x <= &self.len())?,
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
        Q: Hash + Eq + ?Sized,
    {
        self.0.get_full(loc).map(|(idx, _)| idx)
    }

    pub fn loc_multi<'a, I, Q: 'a>(&self, lbls: I) -> Option<Vec<usize>>
    where
        I: IntoIterator<Item = &'a Q>,
        L: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        lbls.into_iter().map(|lbl| self.loc(lbl)).collect::<Option<Vec<_>>>()
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
        Self(IndexSet::new())
    }
}

impl<L: Label> IntoIterator for Index<L> {
    type Item = L;
    type IntoIter = IntoIter<L>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self.0.into_iter())
    }
}

/// Trait to help abstract over the differences among range types.
trait LocRange<R> {
    fn loc_range(&self, range: R) -> Option<Vec<usize>>;
}

fn generic_loc_range_impl<'a, R, L, Q: 'a>(index: &Index<L>, range: R) -> Option<Vec<usize>>
    where
        R: RangeBounds<&'a Q>,
        L: Borrow<Q> + Label,
        Q: Hash + Eq + ?Sized,
{
    let start_idx = match range.start_bound() {
        Bound::Included(lbl) => index.loc(lbl)?,
        Bound::Excluded(lbl) => index.loc(lbl)? + 1,
        Bound::Unbounded => 0,
    };

    let close_idx = match range.end_bound() {
        Bound::Included(lbl) => index.loc(lbl)? + 1,
        Bound::Excluded(lbl) => index.loc(lbl)?,
        Bound::Unbounded => index.len(),
    };

    index.iloc_multi(start_idx..close_idx)
}

impl<L> LocRange<RangeFull> for Index<L>
where
    L: Label,
{
    fn loc_range(&self, _range: RangeFull) -> Option<Vec<usize>> {
        generic_loc_range_impl::<RangeFull, L, L>(self, ..)
    }
}

impl<'a, L, Q> LocRange<Range<&'a Q>> for Index<L>
where
    L: Label + Borrow<Q>,
    Q: 'a + Hash + Eq + ?Sized,
{
    fn loc_range(&self, range: Range<&'a Q>) -> Option<Vec<usize>> {
        generic_loc_range_impl(self, range)
    }
}

impl<'a, L, Q> LocRange<RangeFrom<&'a Q>> for Index<L>
where
    L: Label + Borrow<Q>,
    Q: 'a + Hash + Eq + ?Sized,
{
    fn loc_range(&self, range: RangeFrom<&'a Q>) -> Option<Vec<usize>> {
        generic_loc_range_impl(self, range)
    }
}

impl<'a, L, Q> LocRange<RangeTo<&'a Q>> for Index<L>
where
    L: Label + Borrow<Q>,
    Q: 'a + Hash + Eq + ?Sized,
{
    fn loc_range(&self, range: RangeTo<&'a Q>) -> Option<Vec<usize>> {
        generic_loc_range_impl(self, range)
    }
}

impl<'a, L, Q> LocRange<RangeToInclusive<&'a Q>> for Index<L>
where
    L: Label + Borrow<Q>,
    Q: 'a + Hash + Eq + ?Sized,
{
    fn loc_range(&self, range: RangeToInclusive<&'a Q>) -> Option<Vec<usize>> {
        generic_loc_range_impl(self, range)
    }
}

impl<'a, L, Q> LocRange<RangeInclusive<&'a Q>> for Index<L>
where
    L: Label + Borrow<Q>,
    Q: 'a + Hash + Eq + ?Sized,
{
    fn loc_range(&self, range: RangeInclusive<&'a Q>) -> Option<Vec<usize>> {
        generic_loc_range_impl(self, range)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loc() {
        let i = Index::from_iter("ideographs".chars());

        assert_eq!(Some(0), i.loc(&'i'));
        assert_eq!(Some(1), i.loc(&'d'));
        assert_eq!(Some(2), i.loc(&'e'));
        assert_eq!(Some(3), i.loc(&'o'));
        assert_eq!(Some(4), i.loc(&'g'));
        assert_eq!(Some(5), i.loc(&'r'));
        assert_eq!(Some(6), i.loc(&'a'));
        assert_eq!(Some(7), i.loc(&'p'));
        assert_eq!(Some(8), i.loc(&'h'));
        assert_eq!(Some(9), i.loc(&'s'));
        assert_eq!(None, i.loc(&'x'));
    }

    #[test]
    fn loc_multi() {
        let i = Index::from_slice(&[10u32, 20, 30, 40, 50]);

        assert_eq!(i.loc_multi(&[]), Some(vec![]));
        assert_eq!(i.loc_multi(&[30]), Some(vec![2]));
        assert_eq!(i.loc_multi(&[10, 20, 30, 40]), Some(vec![0, 1, 2, 3]));
        assert_eq!(i.loc_multi(&[50, 40, 30, 20]), Some(vec![4, 3, 2, 1]));
        assert_eq!(i.loc_multi(&[99]), None);
        assert_eq!(i.loc_multi(&[99, 20, 30, 40, 50]), None);
        assert_eq!(i.loc_multi(&[10, 20, 30, 40, 99]), None);
    }

    #[test]
    fn loc_range() {
        let i = Index::from_iter("jihgfedcba".chars());

        assert_eq!(i.loc_range(&'h'..&'c'), Some(vec![2, 3, 4, 5, 6]));
        assert_eq!(i.loc_range(&'h'..=&'c'), Some(vec![2, 3, 4, 5, 6, 7]));
        assert_eq!(i.loc_range(&'h'..), Some(vec![2, 3, 4, 5, 6, 7, 8, 9]));
        assert_eq!(i.loc_range(..&'c'), Some(vec![0, 1, 2, 3, 4, 5, 6]));
        assert_eq!(i.loc_range(..=&'c'), Some(vec![0, 1, 2, 3, 4, 5, 6, 7]));
        assert_eq!(i.loc_range(..), Some(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]));
        assert_eq!(i.loc_range(&'f'..&'e'), Some(vec![4]));
        assert_eq!(i.loc_range(&'f'..=&'e'), Some(vec![4, 5]));
        assert_eq!(i.loc_range(&'f'..&'f'), Some(vec![]));
        assert_eq!(i.loc_range(&'f'..=&'f'), Some(vec![4]));
        assert_eq!(i.loc_range(&'d'..&'g'), Some(vec![]));
        assert_eq!(i.loc_range(&'d'..=&'g'), Some(vec![]));
        assert_eq!(i.loc_range(&'e'..&'f'), Some(vec![]));
        assert_eq!(i.loc_range(&'e'..=&'f'), Some(vec![]));
        assert_eq!(i.loc_range(&'j'..&'x'), None);
        assert_eq!(i.loc_range(&'j'..=&'x'), None);
        assert_eq!(i.loc_range(&'x'..&'a'), None);
        assert_eq!(i.loc_range(&'x'..=&'a'), None);
        assert_eq!(i.loc_range(..&'x'), None);
        assert_eq!(i.loc_range(..=&'x'), None);
        assert_eq!(i.loc_range(&'x'..), None);

        assert_eq!(i.loc_range(..), i.iloc_range(0..i.len()));
        assert_eq!(i.loc_range(..&'c'), i.iloc_range(..7));
        assert_eq!(i.loc_range(..=&'c'), i.iloc_range(..=7));
        assert_eq!(i.loc_range(&'h'..), i.iloc_range(2..));

        assert_eq!(i.loc_range(..), i.loc_range(&'j'..=&'a'));
        assert_eq!(i.loc_range(..&'c'), i.loc_range(&'j'..&'c'));
        assert_eq!(i.loc_range(..=&'c'), i.loc_range(&'j'..=&'c'));
        assert_eq!(i.loc_range(&'h'..), i.loc_range(&'h'..=&'a'));

        let i: Index<std::string::String> = Index::from_vec(vec![
            String::from("ab"),
            String::from("cd"),
            String::from("ef"),
            String::from("gh"),
            String::from("ij"),
            String::from("kl"),
            String::from("mn"),
            String::from("op"),
            String::from("qr"),
            String::from("st"),
        ]);

        assert_eq!(i.loc_range("ef".."op"), Some(vec![2, 3, 4, 5, 6]));
        assert_eq!(i.loc_range("ef"..="op"), Some(vec![2, 3, 4, 5, 6, 7]));
        assert_eq!(i.loc_range("ef"..), Some(vec![2, 3, 4, 5, 6, 7, 8, 9]));
        assert_eq!(i.loc_range(.."op"), Some(vec![0, 1, 2, 3, 4, 5, 6]));
        assert_eq!(i.loc_range(..="op"), Some(vec![0, 1, 2, 3, 4, 5, 6, 7]));
        assert_eq!(i.loc_range(..), Some(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]));
        assert_eq!(i.loc_range("ij".."kl"), Some(vec![4]));
        assert_eq!(i.loc_range("ij"..="kl"), Some(vec![4, 5]));
        assert_eq!(i.loc_range("ij".."ij"), Some(vec![]));
        assert_eq!(i.loc_range("ij"..="ij"), Some(vec![4]));
        assert_eq!(i.loc_range("mn".."gh"), Some(vec![]));
        assert_eq!(i.loc_range("mn"..="gh"), Some(vec![]));
        assert_eq!(i.loc_range("kl".."ij"), Some(vec![]));
        assert_eq!(i.loc_range("kl"..="ij"), Some(vec![]));
        assert_eq!(i.loc_range("ab".."???"), None);
        assert_eq!(i.loc_range("ab"..="???"), None);
        assert_eq!(i.loc_range("???".."ab"), None);
        assert_eq!(i.loc_range("???"..="ab"), None);
        assert_eq!(i.loc_range(.."???"), None);
        assert_eq!(i.loc_range(..="???"), None);
        assert_eq!(i.loc_range("???"..), None);

        assert_eq!(i.loc_range(..), i.iloc_range(0..i.len()));
        assert_eq!(i.loc_range(.."op"), i.iloc_range(..7));
        assert_eq!(i.loc_range(..="op"), i.iloc_range(..=7));
        assert_eq!(i.loc_range("ef"..), i.iloc_range(2..));

        assert_eq!(i.loc_range(..), i.loc_range("ab"..="st"));
        assert_eq!(i.loc_range(.."op"), i.loc_range("ab".."op"));
        assert_eq!(i.loc_range(..="op"), i.loc_range("ab"..="op"));
        assert_eq!(i.loc_range("ef"..), i.loc_range("ef"..="st"));
    }
}
