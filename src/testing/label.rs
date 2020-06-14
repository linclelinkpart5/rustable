
use std::collections::HashSet;
use std::iter::FromIterator;

use proptest::prelude::*;
use proptest::collection::hash_set;

use crate::traits::Label;

pub const MAX_LABELS: usize = 2000;

pub type LabelSetPair<L> = (HashSet<L>, HashSet<L>);

pub struct LabelGen;

impl LabelGen {
    pub fn unordered<L: Label + Arbitrary>() -> impl Strategy<Value = HashSet<L>> {
        hash_set(any::<L>(), 0..=MAX_LABELS)
    }

    pub fn ordered<L: Label + Arbitrary>() -> impl Strategy<Value = Vec<L>> {
        Self::unordered().prop_map(|m| Vec::from_iter(m))
    }

    pub fn disjoint_pair<L: Label + Arbitrary>() -> impl Strategy<Value = LabelSetPair<L>> {
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

    pub fn non_disjoint_pair<L: Label + Arbitrary>() -> impl Strategy<Value = LabelSetPair<L>> {
        // Generate the desired size of each set, as well as the number of
        // labels to have in common between them.
        // NOTE: The minimum size is 1, since empty sets are always disjoint.
        (1..=MAX_LABELS).prop_flat_map(|len| {
            (Just(len), 1..=len)
        })
        .prop_flat_map(|(len, inner_len)| {
            let outer_len = len - inner_len;
            let total_len = (2 * outer_len) + inner_len;

            hash_set(any::<L>(), total_len).prop_map(move |label_pool| {
                let mut iter = label_pool.into_iter();

                // Take the desired number of common labels, then clone the set.
                let mut a: HashSet<_> = iter.by_ref().take(inner_len).collect();
                let mut b = a.clone();

                // There should be exactly enough labels to add to each set to
                // make each one the target length.
                a.extend(iter.by_ref().take(outer_len));
                b.extend(iter);

                (a, b)
            })
        })
    }

    pub fn strict_subset_pair<L: Label + Arbitrary>() -> impl Strategy<Value = LabelSetPair<L>> {
        // Generate the desired size of the larger set as well as the number of
        // labels to copy into the smaller set.
        // The larger set should always be a strict superset.
        (1..=MAX_LABELS).prop_flat_map(|len| {
            (Just(len), 0..len)
        })
        // Calculate the total number of unique labels that are needed, and
        // generate two sets.
        .prop_flat_map(|(len, sub_len)| {
            hash_set(any::<L>(), len).prop_map(move |comb_map| {
                let mut iter = comb_map.into_iter();

                let a: HashSet<_> = iter.by_ref().take(sub_len).collect();
                let mut b = a.clone();

                b.extend(iter);

                (a, b)
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    proptest! {
        #[test]
        fn verify_disjoint_pair(
            (labels_a, labels_b) in LabelGen::disjoint_pair::<i32>()
        )
        {
            assert!(labels_a.len() <= MAX_LABELS);
            assert!(labels_b.len() <= MAX_LABELS);
            assert_eq!(labels_a.len(), labels_b.len());
            assert!(labels_a.is_disjoint(&labels_b));
        }
    }

    proptest! {
        #[test]
        fn verify_non_disjoint_pair(
            (labels_a, labels_b) in LabelGen::non_disjoint_pair::<i32>()
        )
        {
            assert!(labels_a.len() >= 1);
            assert!(labels_b.len() >= 1);
            assert!(labels_a.len() <= MAX_LABELS);
            assert!(labels_b.len() <= MAX_LABELS);
            assert_eq!(labels_a.len(), labels_b.len());
            assert!(!labels_a.is_disjoint(&labels_b));
        }
    }

    proptest! {
        #[test]
        fn verify_strict_subset_pair(
            (labels_a, labels_b) in LabelGen::strict_subset_pair::<i32>()
        )
        {
            assert!(labels_b.len() <= MAX_LABELS);
            assert!(labels_a.len() < labels_b.len());
            assert!(labels_a.is_subset(&labels_b));
        }
    }
}
