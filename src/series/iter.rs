
use std::iter::Zip;
use std::slice::Iter as SliceIter;

use crate::traits::Storable;
use crate::traits::Label;
use crate::series::Series;
use crate::index::iter::Iter as IndexIter;

pub struct Iter<'a, K, V>(Zip<IndexIter<'a, K>, SliceIter<'a, V>>)
where
    K: Label,
    V: Storable,
;

impl<'a, K, V> Iter<'a, K, V>
where
    K: Label,
    V: Storable,
{
    pub(crate) fn new(series: &'a Series<'a, K, V>) -> Self {
        Self(series.index().iter().zip(series.values().iter()))
    }
}

impl<'a, K, V> Iterator for Iter<'a, K, V>
where
    K: Label,
    V: Storable,
{
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a, K, V> DoubleEndedIterator for Iter<'a, K, V>
where
    K: Label,
    V: Storable,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}

impl<'a, K, V> ExactSizeIterator for Iter<'a, K, V>
where
    K: Label,
    V: Storable,
{
    fn len(&self) -> usize {
        self.0.len()
    }
}
