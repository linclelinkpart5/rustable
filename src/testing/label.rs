
use std::collections::HashSet;
use std::iter::FromIterator;

use proptest::prelude::Arbitrary;
use proptest::prelude::Strategy;

use crate::traits::Label;

pub const MAX_LABELS: usize = 2000;

pub struct ArbLabel;

impl ArbLabel {
    fn unordered<L: Label + Arbitrary>() -> impl Strategy<Value = HashSet<L>> {
        proptest::collection::hash_set(proptest::arbitrary::any::<L>(), 0..=MAX_LABELS)
    }

    fn ordered<L: Label + Arbitrary>() -> impl Strategy<Value = Vec<L>> {
        Self::unordered().prop_map(|m| Vec::from_iter(m))
    }
}
