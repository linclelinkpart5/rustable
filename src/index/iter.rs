
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

impl<'a, L: Label> ExactSizeIterator for Iter<'a, L> {
    fn len(&self) -> usize {
        self.0.len()
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

impl<L: Label> ExactSizeIterator for IntoIter<L> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

/// A lazy iterator producing elements in the difference of `Index`s.
pub struct Difference<'a, L: Label>(Iter<'a, L>, &'a Index<L>);

impl<'a, L: Label> Difference<'a, L> {
    pub(crate) fn new(index_a: &'a Index<L>, index_b: &'a Index<L>) -> Self {
        Self(index_a.iter(), index_b)
    }
}

impl<'a, L: Label> Iterator for Difference<'a, L> {
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

impl<'a, L: Label> DoubleEndedIterator for Difference<'a, L> {
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
pub struct SymmetricDifference<'a, L: Label>(Chain<Difference<'a, L>, Difference<'a, L>>);

impl<'a, L: Label> SymmetricDifference<'a, L> {
    pub(crate) fn new(index_a: &'a Index<L>, index_b: &'a Index<L>) -> Self {
        Self(Difference::new(index_a, index_b).chain(Difference::new(index_b, index_a)))
    }
}

impl<'a, L: Label> Iterator for SymmetricDifference<'a, L> {
    type Item = &'a L;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a, L: Label> DoubleEndedIterator for SymmetricDifference<'a, L> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}

/// A lazy iterator producing elements in the intersection of `Index`s.
pub struct Intersection<'a, L: Label>(Iter<'a, L>, &'a Index<L>);

impl<'a, L: Label> Intersection<'a, L> {
    pub(crate) fn new(index_a: &'a Index<L>, index_b: &'a Index<L>) -> Self {
        Self(index_a.iter(), index_b)
    }
}

impl<'a, L: Label> Iterator for Intersection<'a, L> {
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

impl<'a, L: Label> DoubleEndedIterator for Intersection<'a, L> {
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
pub struct Union<'a, L: Label>(Chain<Iter<'a, L>, Difference<'a, L>>);

impl<'a, L: Label> Union<'a, L> {
    pub(crate) fn new(index_a: &'a Index<L>, index_b: &'a Index<L>) -> Self {
        Self(index_a.iter().chain(Difference::new(index_b, index_a)))
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
