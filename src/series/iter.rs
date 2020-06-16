
use std::iter::Zip;
use std::slice::Iter as SliceIter;
use std::slice::IterMut as SliceIterMut;

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
        Self(series.0.iter().zip(series.1.iter()))
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

pub struct IterMut<'a, K, V>(Zip<IndexIter<'a, K>, SliceIterMut<'a, V>>)
where
    K: Label,
    V: Storable,
;

impl<'a, K, V> IterMut<'a, K, V>
where
    K: Label,
    V: Storable,
{
    pub(crate) fn new(series: &'a mut Series<'a, K, V>) -> Self {
        Self(series.0.iter().zip(series.1.iter_mut()))
    }
}

impl<'a, K, V> Iterator for IterMut<'a, K, V>
where
    K: Label,
    V: Storable,
{
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a, K, V> DoubleEndedIterator for IterMut<'a, K, V>
where
    K: Label,
    V: Storable,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}

impl<'a, K, V> ExactSizeIterator for IterMut<'a, K, V>
where
    K: Label,
    V: Storable,
{
    fn len(&self) -> usize {
        self.0.len()
    }
}
