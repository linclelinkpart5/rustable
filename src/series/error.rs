
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::error::Error;

use crate::traits::Storable;
use crate::traits::Label;

#[derive(Debug)]
pub struct ValueLenMismatch<V: Storable> {
    pub expected_len: usize,
    pub values: Vec<V>,
}

impl<V: Storable> Display for ValueLenMismatch<V> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "expected number of values {}, got {}", self.expected_len, self.values.len())
    }
}

impl<V: Storable> Error for ValueLenMismatch<V> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

#[derive(Debug)]
pub struct DuplicateIndexLabel<L: Label> {
    pub label: L,
}

impl<L: Label> Display for DuplicateIndexLabel<L> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "found duplicate index label: {:?}", self.label)
    }
}

impl<L: Label> Error for DuplicateIndexLabel<L> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}
