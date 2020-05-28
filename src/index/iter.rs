
//! Iterators for use with `Index`.

use crate::traits::Label;

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

pub struct IntoIter<L: Label>(pub(crate) indexmap::set::IntoIter<L>);

impl<L: Label> Iterator for IntoIter<L> {
    type Item = L;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}
