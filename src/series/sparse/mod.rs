
use std::borrow::Cow;

use crate::traits::Storable;
use crate::traits::Label;
use crate::index::Index;

#[derive(Debug)]
pub struct SeriesSparse<'a, L: Label, V: Storable>(
    pub(crate) Cow<'a, Index<L>>,
    pub(crate) Cow<'a, [Option<V>]>,
);
