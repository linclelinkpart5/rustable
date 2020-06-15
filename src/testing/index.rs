
use proptest::prelude::*;

use crate::index::Index;
use crate::traits::Label;
use crate::testing::label::LabelGen;

pub struct IndexGen;

impl IndexGen {
    pub fn index<L: Label + Arbitrary>() -> impl Strategy<Value = Index<L>> {
        LabelGen::ordered().prop_map(|v| Index::from(v))
    }
}
