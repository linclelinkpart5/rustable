
use std::iter::Zip;
use std::slice::Iter as SliceIter;
use std::slice::IterMut as SliceIterMut;
use std::vec::IntoIter as VecIntoIter;

use super::Series;

use crate::traits::Storable;
use crate::traits::Label;
use crate::index::iter::Iter as IndexIter;
use crate::index::iter::IntoIter as IndexIntoIter;

pub struct Iter<'a, L: Label, V: Storable>(
    Zip<IndexIter<'a, L>, SliceIter<'a, V>>,
);

impl<'a, L, V> Iter<'a, L, V>
where
    L: Label,
    V: Storable,
{
    pub(crate) fn new(series: &'a Series<L, V>) -> Self {
        Self(series.0.iter().zip(series.1.iter()))
    }
}

impl<'a, L, V> Iterator for Iter<'a, L, V>
where
    L: Label,
    V: Storable,
{
    type Item = (&'a L, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a, L, V> DoubleEndedIterator for Iter<'a, L, V>
where
    L: Label,
    V: Storable,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}

impl<'a, L, V> ExactSizeIterator for Iter<'a, L, V>
where
    L: Label,
    V: Storable,
{
    fn len(&self) -> usize {
        self.0.len()
    }
}

pub struct IterMut<'a, L: Label, V: Storable>(
    Zip<IndexIter<'a, L>, SliceIterMut<'a, V>>,
);

impl<'a, L, V> IterMut<'a, L, V>
where
    L: Label,
    V: Storable,
{
    pub(crate) fn new(series: &'a mut Series<L, V>) -> Self {
        Self(series.0.iter().zip(series.1.iter_mut()))
    }
}

impl<'a, L, V> Iterator for IterMut<'a, L, V>
where
    L: Label,
    V: Storable,
{
    type Item = (&'a L, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a, L, V> DoubleEndedIterator for IterMut<'a, L, V>
where
    L: Label,
    V: Storable,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}

impl<'a, L, V> ExactSizeIterator for IterMut<'a, L, V>
where
    L: Label,
    V: Storable,
{
    fn len(&self) -> usize {
        self.0.len()
    }
}

pub struct IntoIter<L: Label, V: Storable>(
    Zip<IndexIntoIter<L>, VecIntoIter<V>>,
);

impl<L, V> IntoIter<L, V>
where
    L: Label,
    V: Storable,
{
    pub(crate) fn new(series: Series<L, V>) -> Self {
        let (index, values) = series.into_index_values();
        let index_ii = index.into_iter();
        let values_ii = values.into_iter();

        Self(index_ii.zip(values_ii))
    }
}

impl<L, V> Iterator for IntoIter<L, V>
where
    L: Label,
    V: Storable,
{
    type Item = (L, V);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<L, V> DoubleEndedIterator for IntoIter<L, V>
where
    L: Label,
    V: Storable,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}

impl<L, V> ExactSizeIterator for IntoIter<L, V>
where
    L: Label,
    V: Storable,
{
    fn len(&self) -> usize {
        self.0.len()
    }
}
