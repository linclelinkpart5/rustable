
use std::collections::HashSet;
use std::iter::FromIterator;

use proptest::prelude::*;
use proptest::collection::hash_set;

use crate::traits::Label;

pub const MAX_LABELS: usize = 2000;

pub type SetPair<L> = (HashSet<L>, HashSet<L>);

pub struct ArbLabel;

impl ArbLabel {
    fn unordered<L: Label + Arbitrary>() -> impl Strategy<Value = HashSet<L>> {
        hash_set(any::<L>(), 0..=MAX_LABELS)
    }

    fn ordered<L: Label + Arbitrary>() -> impl Strategy<Value = Vec<L>> {
        Self::unordered().prop_map(|m| Vec::from_iter(m))
    }

    fn disjoint_pair<L: Label + Arbitrary>() -> impl Strategy<Value = SetPair<L>> {
        (0..=MAX_LABELS).prop_flat_map(|len| {
            let tot_len = 2 * len;

            hash_set(any::<L>(), tot_len).prop_map(move |m| {
                let mut it = m.into_iter();

                let a = it.by_ref().take(tot_len / 2).collect();
                let b = it.collect();

                (a, b)
            })
        })
    }

    fn partial_overlap_pair<L: Label + Arbitrary>() -> impl Strategy<Value = SetPair<L>> {
        // Generate the desired size of each set as well as the number of
        // labels to duplicate between them.
        // Always want at least one label in common, and at least one label that
        // is in one set and not the other, so each set has a minimum size of 2.
        (2..=MAX_LABELS).prop_flat_map(|len| {
            (Just(len), 1..len)
        })
        // Calculate the total number of unique labels that are needed, and
        // generate two sets.
        .prop_flat_map(|(len, inn_len)| {
            let out_len = len - inn_len;

            let total = (2 * out_len) + inn_len;

            hash_set(any::<L>(), total).prop_map(move |comb_map| {
                let mut iter = comb_map.into_iter();

                let mut a: HashSet<_> = iter.by_ref().take(inn_len).collect();
                let mut b = a.clone();

                a.extend(iter.by_ref().take(out_len));
                b.extend(iter);

                (a, b)
            })
        })
    }
}
