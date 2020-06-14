
use std::iter::FromIterator;

use proptest::prelude::*;

use crate::index::Index;
use crate::traits::Label;
use crate::testing::label::LabelGen;

pub type IndexPair<L> = (Index<L>, Index<L>);

pub struct IndexGen;

impl IndexGen {
    pub fn index<L: Label + Arbitrary>() -> impl Strategy<Value = Index<L>> {
        LabelGen::ordered().prop_map(|v| Index::from(v))
    }

    pub fn disjoint_pair<L: Label + Arbitrary>() -> impl Strategy<Value = IndexPair<L>> {
        LabelGen::disjoint_pair().prop_map(|(a, b)| {
            (Index::from_iter(a), Index::from_iter(b))
        })
    }

    pub fn partial_overlap_pair<L: Label + Arbitrary>() -> impl Strategy<Value = IndexPair<L>> {
        LabelGen::partial_overlap_pair().prop_map(|(a, b)| {
            (Index::from_iter(a), Index::from_iter(b))
        })
    }

    pub fn strict_subset_pair<L: Label + Arbitrary>() -> impl Strategy<Value = IndexPair<L>> {
        LabelGen::strict_subset_pair().prop_map(|(a, b)| {
            (Index::from_iter(a), Index::from_iter(b))
        })
    }
}
