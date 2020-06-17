
use std::borrow::Cow;

use crate::traits::Storable;

pub enum ValueStore<'a, V: Storable> {
    Dense(Cow<'a, [V]>),
    Sparse(Cow<'a, [Option<V>]>),
}
