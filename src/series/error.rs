
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::error::Error;

use crate::traits::Storable;

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
