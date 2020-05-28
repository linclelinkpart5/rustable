
//! Iterators for use with `Index`.

use std::hash::Hash;

use crate::traits::Storable;

pub struct Iter<'a, K>(pub(crate) indexmap::set::Iter<'a, K>)
where
    K: Storable + Eq + Hash
;

impl<'a, K> Iterator for Iter<'a, K>
where
    K: Storable + Eq + Hash,
{
    type Item = &'a K;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

pub struct IntoIter<K>(pub(crate) indexmap::set::IntoIter<K>)
where
    K: Storable + Eq + Hash,
;

impl<K> Iterator for IntoIter<K>
where
    K: Storable + Eq + Hash,
{
    type Item = K;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}
