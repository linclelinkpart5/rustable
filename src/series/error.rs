
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::error::Error;

use crate::index::Index;
use crate::traits::Storable;
use crate::traits::Label;

#[derive(Debug)]
pub struct LengthMismatch<L: Label, V: Storable> {
    pub index: Index<L>,
    pub values: Vec<V>,
}

impl<L: Label, V: Storable> Display for LengthMismatch<L, V> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let i_len = self.index.len();
        let v_len = self.values.len();
        write!(f, "length mismatch between index and values: {} != {}", i_len, v_len)
    }
}

impl<L: Label, V: Storable> Error for LengthMismatch<L, V> {
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

#[derive(Debug)]
pub struct OverlappingIndex;

impl Display for OverlappingIndex {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "overlapping index")
    }
}

impl Error for OverlappingIndex {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}
