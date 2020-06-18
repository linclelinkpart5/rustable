
use std::borrow::Cow;

use crate::traits::Storable;

struct ValueStoreImpl<'a, T: Clone>(Cow<'a, [T]>);

impl<'a, T: Clone> ValueStoreImpl<'a, T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        // If the requested capacity is 0, then no need to allocate.
        if capacity == 0 {
            Self::new()
        } else {
            Self(Cow::Owned(Vec::with_capacity(capacity)))
        }
    }

    pub fn capacity(&self) -> usize {
        match &self.0 {
            // With a `Vec`, just return the actual capacity.
            Cow::Owned(vec) => vec.capacity(),

            // With a slice, there is no real capacity, but there is a pseudo
            // "capacity" equal to the length of the slice.
            Cow::Borrowed(slice) => slice.len(),
        }
    }

    pub fn reserve(&mut self, additional: usize) {
        // This requires making the underlying data store owned.
        // A borrowed data store cannot actually hold new elements.
        self.0.to_mut().reserve(additional)
    }

    pub fn shrink_to_fit(&mut self) {
        match &mut self.0 {
            Cow::Owned(vec) => vec.shrink_to_fit(),

            // With a slice, this is a no-op.
            Cow::Borrowed(..) => {},
        };
    }

    pub fn truncate(&mut self, len: usize) {
        // Only do work if values need to be removed.
        if len < self.len() {
            self.0.to_mut().truncate(len);
        }
    }

    pub fn as_slice(&self) -> &[T] {
        match &self.0 {
            Cow::Owned(vec) => vec.as_slice(),
            Cow::Borrowed(slice) => slice,
        }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        self.0.to_mut().as_mut_slice()
    }

    pub fn swap_remove(&mut self, index: usize) -> Option<T> {
        // If the index is out of bounds, just no-op and return `None`.
        if index >= self.len() {
            None
        } else {
            Some(self.0.to_mut().swap_remove(index))
        }
    }

    pub fn insert(&mut self, index: usize, value: T) -> Option<()> {
        // If the index is out of bounds, just no-op and return `None`.
        if index > self.len() {
            None
        } else {
            self.0.to_mut().insert(index, value);
            Some(())
        }
    }

    pub fn remove(&mut self, index: usize) -> Option<T> {
        // If the index is out of bounds, just no-op and return `None`.
        if index >= self.len() {
            None
        } else {
            Some(self.0.to_mut().remove(index))
        }
    }

    pub fn retain<F>(&mut self, func: F)
    where
        F: FnMut(&T) -> bool,
    {
        // TODO: See if there is a way to avoid having to clone if no changes
        //       are needed.
        if self.len() > 0 {
            self.0.to_mut().retain(func)
        }
    }

    pub fn dedup(&mut self)
    where
        T: PartialEq,
    {
        // TODO: See if there is a way to avoid having to clone if no changes
        //       are needed.
        if self.len() > 1 {
            self.0.to_mut().dedup()
        }
    }

    pub fn dedup_by<F>(&mut self, same_bucket: F)
    where
        F: FnMut(&mut T, &mut T) -> bool,
    {
        // TODO: See if there is a way to avoid having to clone if no changes
        //       are needed.
        if self.len() > 1 {
            self.0.to_mut().dedup_by(same_bucket)
        }
    }

    pub fn dedup_by_key<F, K>(&mut self, key_func: F)
    where
        F: FnMut(&mut T) -> K,
        K: PartialEq<K>,
    {
        // TODO: See if there is a way to avoid having to clone if no changes
        //       are needed.
        if self.len() > 1 {
            self.0.to_mut().dedup_by_key(key_func)
        }
    }

    pub fn push(&mut self, value: T) {
        self.0.to_mut().push(value)
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len() > 0 {
            self.0.to_mut().pop()
        } else {
            None
        }
    }

    // append
    // drain

    pub fn clear(&mut self) {
        if self.len() > 0 {
            self.0.to_mut().clear()
        }
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<'a, T: Clone> Default for ValueStoreImpl<'a, T> {
    fn default() -> Self {
        Self(Cow::Borrowed(&[]))
    }
}

pub struct DenseValueStore<'a, V: Storable>(ValueStoreImpl<'a, V>);
pub struct SparseValueStore<'a, V: Storable>(ValueStoreImpl<'a, Option<V>>);
