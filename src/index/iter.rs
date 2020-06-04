
//! Iterators for use with `Index`.

use std::borrow::Borrow;
use std::hash::Hash;
use std::iter::Chain;
use std::ops::Range;

use crate::traits::Label;

use super::Index;

pub struct Iter<'a, L: Label>(pub(crate) indexmap::set::Iter<'a, L>);

impl<'a, L: Label> Iterator for Iter<'a, L> {
    type Item = &'a L;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a, L: Label> DoubleEndedIterator for Iter<'a, L> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}

pub struct IntoIter<L: Label>(pub(crate) indexmap::set::IntoIter<L>);

impl<L: Label> Iterator for IntoIter<L> {
    type Item = L;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<L: Label> DoubleEndedIterator for IntoIter<L> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}

/// A lazy iterator producing elements in the difference of `Index`s.
pub struct Diff<'a, L: Label>(Iter<'a, L>, &'a Index<L>);

impl<'a, L: Label> Diff<'a, L> {
    pub(crate) fn new(index_a: &'a Index<L>, index_b: &'a Index<L>) -> Self {
        Self(index_a.iter(), index_b)
    }
}

impl<'a, L: Label> Iterator for Diff<'a, L> {
    type Item = &'a L;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(label) = self.0.next() {
            if !self.1.contains(label) {
                return Some(label);
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, self.0.size_hint().1)
    }
}

impl<'a, L: Label> DoubleEndedIterator for Diff<'a, L> {
    fn next_back(&mut self) -> Option<Self::Item> {
        while let Some(label) = self.0.next_back() {
            if !self.1.contains(label) {
                return Some(label);
            }
        }
        None
    }
}

/// A lazy iterator producing elements in the symmetric difference of `Index`s.
pub struct SymDiff<'a, L: Label>(Chain<Diff<'a, L>, Diff<'a, L>>);

impl<'a, L: Label> SymDiff<'a, L> {
    pub(crate) fn new(index_a: &'a Index<L>, index_b: &'a Index<L>) -> Self {
        Self(Diff::new(index_a, index_b).chain(Diff::new(index_b, index_a)))
    }
}

impl<'a, L: Label> Iterator for SymDiff<'a, L> {
    type Item = &'a L;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a, L: Label> DoubleEndedIterator for SymDiff<'a, L> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}

/// A lazy iterator producing elements in the intersection of `Index`s.
pub struct Inter<'a, L: Label>(Iter<'a, L>, &'a Index<L>);

impl<'a, L: Label> Inter<'a, L> {
    pub(crate) fn new(index_a: &'a Index<L>, index_b: &'a Index<L>) -> Self {
        Self(index_a.iter(), index_b)
    }
}

impl<'a, L: Label> Iterator for Inter<'a, L> {
    type Item = &'a L;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(label) = self.0.next() {
            if self.1.contains(label) {
                return Some(label);
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, self.0.size_hint().1)
    }
}

impl<'a, L: Label> DoubleEndedIterator for Inter<'a, L> {
    fn next_back(&mut self) -> Option<Self::Item> {
        while let Some(label) = self.0.next_back() {
            if self.1.contains(label) {
                return Some(label);
            }
        }
        None
    }
}

/// A lazy iterator producing elements in the union of `Index`s.
pub struct Union<'a, L: Label>(Chain<Iter<'a, L>, Diff<'a, L>>);

impl<'a, L: Label> Union<'a, L> {
    pub(crate) fn new(index_a: &'a Index<L>, index_b: &'a Index<L>) -> Self {
        Self(index_a.iter().chain(Diff::new(index_b, index_a)))
    }
}

impl<'a, L: Label> Iterator for Union<'a, L> {
    type Item = &'a L;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a, L: Label> DoubleEndedIterator for Union<'a, L> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}

/// A lazy iterator that yields indices from multiple iloc indices.
pub struct ILocMulti<'a, I, L>(I, &'a Index<L>)
where
    I: Iterator<Item = &'a usize>,
    L: Label,
;

impl<'a, I, L> ILocMulti<'a, I, L>
where
    I: Iterator<Item = &'a usize>,
    L: Label,
{
    // NOTE: Want to intentionally keep a ref to the index, not just the size,
    //       otherwise cohesion is lost.
    pub(crate) fn new(iter: I, index: &'a Index<L>) -> Self {
        Self(iter, index)
    }
}

impl<'a, I, L> Iterator for ILocMulti<'a, I, L>
where
    I: Iterator<Item = &'a usize>,
    L: Label,
{
    type Item = Option<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        let idx = self.0.next()?;
        Some(self.1.iloc(idx))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a, I, L> DoubleEndedIterator for ILocMulti<'a, I, L>
where
    I: Iterator<Item = &'a usize> + DoubleEndedIterator,
    L: Label,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let idx = self.0.next_back()?;
        Some(self.1.iloc(idx))
    }
}

impl<'a, I, L> ExactSizeIterator for ILocMulti<'a, I, L>
where
    I: Iterator<Item = &'a usize> + ExactSizeIterator,
    L: Label,
{
    fn len(&self) -> usize {
        self.0.len()
    }
}

/// A lazy iterator that yields indices from a iloc range expression.
pub struct ILocRange<'a, L>(Range<usize>, &'a Index<L>)
where
    L: Label,
;

impl<'a, L> ILocRange<'a, L>
where
    L: Label,
{
    // NOTE: Want to intentionally keep a ref to the index, even though the
    //       bounds are independent, otherwise cohesion is lost.
    pub(crate) fn new(range: Range<usize>, index: &'a Index<L>) -> Option<Self> {
        if range.start > index.len() || range.end > index.len() { None }
        else { Some(Self(range, index)) }
    }
}

impl<'a, L> Iterator for ILocRange<'a, L>
where
    L: Label,
{
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a, L> DoubleEndedIterator for ILocRange<'a, L>
where
    L: Label,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}

impl<'a, L> ExactSizeIterator for ILocRange<'a, L>
where
    L: Label,
{
    fn len(&self) -> usize {
        self.0.len()
    }
}

/// A lazy iterator that yields indices from multiple loc indices.
pub struct LocMulti<'a, I, L, Q>(I, &'a Index<L>)
where
    I: Iterator<Item = &'a Q>,
    L: Label + Borrow<Q>,
    Q: 'a + Hash + Eq;

impl<'a, I, L, Q> LocMulti<'a, I, L, Q>
where
    I: Iterator<Item = &'a Q>,
    L: Label + Borrow<Q>,
    Q: 'a + Hash + Eq,
{
    pub(crate) fn new(iter: I, index: &'a Index<L>) -> Self {
        Self(iter, index)
    }
}

impl<'a, I, L, Q> Iterator for LocMulti<'a, I, L, Q>
where
    I: Iterator<Item = &'a Q>,
    L: Label + Borrow<Q>,
    Q: Hash + Eq,
{
    type Item = Option<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        let lbl = self.0.next()?;
        Some(self.1.loc(lbl))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a, I, L, Q> DoubleEndedIterator for LocMulti<'a, I, L, Q>
where
    I: Iterator<Item = &'a Q> + DoubleEndedIterator,
    L: Label + Borrow<Q>,
    Q: Hash + Eq,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let lbl = self.0.next_back()?;
        Some(self.1.loc(lbl))
    }
}

impl<'a, I, L, Q> ExactSizeIterator for LocMulti<'a, I, L, Q>
where
    I: Iterator<Item = &'a Q> + ExactSizeIterator,
    L: Label + Borrow<Q>,
    Q: Hash + Eq,
{
    fn len(&self) -> usize {
        self.0.len()
    }
}

/// A lazy iterator that yields indices from a loc range expression.
pub struct LocRange<'a, L>(ILocRange<'a, L>)
where
    L: Label,
;

impl<'a, L> LocRange<'a, L>
where
    L: Label,
{
    pub(crate) fn new<Q>(a_lbl: &'a Q, b_lbl: &'a Q, index: &'a Index<L>) -> Option<Self>
    where
        L: Borrow<Q>,
        Q: 'a + Hash + Eq,
    {
        // This returns the range from the first to the second label, inclusive.
        let start_idx = index.loc(a_lbl)?;
        let close_idx = index.loc(b_lbl).and_then(|i| i.checked_add(1))?;
        ILocRange::new(start_idx..close_idx, index).map(Self)
    }
}

impl<'a, L> Iterator for LocRange<'a, L>
where
    L: Label,
{
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a, L> DoubleEndedIterator for LocRange<'a, L>
where
    L: Label,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}

impl<'a, L> ExactSizeIterator for LocRange<'a, L>
where
    L: Label,
{
    fn len(&self) -> usize {
        self.0.len()
    }
}
