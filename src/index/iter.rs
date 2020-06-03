
//! Iterators for use with `Index`.

use std::iter::Chain;

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
