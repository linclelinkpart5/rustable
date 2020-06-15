
use std::collections::HashSet;
use std::iter::FromIterator;

use proptest::prelude::*;
use proptest::collection::hash_set;

use crate::traits::Label;

pub const MAX_LABELS: usize = 2000;

pub struct LabelGen;

impl LabelGen {
    pub fn unordered<L>() -> impl Strategy<Value = HashSet<L>>
    where
        L: Label + Arbitrary,
    {
        hash_set(any::<L>(), 0..=MAX_LABELS)
    }

    pub fn ordered<L>() -> impl Strategy<Value = Vec<L>>
    where
        L: Label + Arbitrary,
    {
        Self::unordered().prop_map(|m| Vec::from_iter(m))
    }
}
