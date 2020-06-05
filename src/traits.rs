
use std::borrow::Borrow;
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::RangeBounds;

use crate::types::DType;

/// The main trait the defines what types are able to be stored in a `Series`.
pub trait Storable: Debug + Clone + Send + Sized {
    /// Returns the [`DType`] of this type.
    fn dtype() -> DType;
}

/// Trait that defines what is needed for a label in an `Index`.
pub trait Label: Storable + PartialEq + Eq + Hash + PartialOrd + Ord {}

impl<T: Storable + Eq + Hash + Ord> Label for T {}

/// Defines types that support lookup via a single iloc.
pub trait ILocable {
    type Item;

    fn iloc(&self, index: &usize) -> Option<Self::Item>;
}

/// Defines types that support lookup via an iterable of ilocs.
/// The returned iterator yields `Option<Self::Item>`, as it is possible for the
/// input iterable to have invalid values, and these invalid values can usually
/// only be observed during iteration.
pub trait ILocableMulti<'a, I>
where
    I: IntoIterator<Item = &'a usize>,
{
    type Item;
    type Iter: Iterator<Item = Option<Self::Item>>;

    fn iloc_multi(&self, indices: I) -> Self::Iter;
}

/// Defines types that support lookup via a range of ilocs.
/// Since typically it is possible to check in advance if a range is valid, this
/// returns `Option<Self::Iter>`, but the iterator (if it exists) simply yields
/// `Self::Item`.
pub trait ILocableRange<'a, R>
where
    R: RangeBounds<&'a usize>,
{
    type Item;
    type Iter: Iterator<Item = Self::Item>;

    fn iloc_range(&self, range: R) -> Option<Self::Iter>;
}

/// Defines types that support lookup via a single loc.
pub trait Locable<L, Q>
where
    L: Label + Borrow<Q>,
    Q: Hash + Eq + ?Sized,
{
    type Item;

    fn loc(&self, label: &Q) -> Option<Self::Item>;
}

/// Defines types that support lookup via an iterable of locs.
/// The returned iterator yields `Option<Self::Item>`, as it is possible for the
/// input iterable to have invalid values, and these invalid values can usually
/// only be observed during iteration.
pub trait LocableMulti<'a, I, L, Q>
where
    I: IntoIterator<Item = &'a Q>,
    L: Label + Borrow<Q>,
    Q: 'a + Hash + Eq + ?Sized,
{
    type Item;
    type Iter: Iterator<Item = Option<Self::Item>>;

    fn loc_multi(&self, labels: I) -> Self::Iter;
}

/// Defines types that support lookup via a range of locs.
/// Since typically it is possible to check in advance if a range is valid, this
/// returns `Option<Self::Iter>`, but the iterator (if it exists) simply yields
/// `Self::Item`.
pub trait LocableRange<'a, R, L, Q>
where
    R: RangeBounds<&'a Q>,
    L: Label + Borrow<Q>,
    Q: 'a + Hash + Eq + ?Sized,
{
    type Item;
    type Iter: Iterator<Item = Self::Item>;

    fn loc_range(&self, range: R) -> Option<Self::Iter>;
}

/// Defines types that support lookup via a fixed sequence of booleans.
/// The input sequence of booleans shoud have the same dimensions as the
/// implementing type. If not, this should return `None`.
pub trait BLocable<A>
where
    A: AsRef<[bool]>,
{
    type Item;
    type Iter: Iterator<Item = Self::Item>;

    fn bloc(&self, selectors: &A) -> Option<Self::Iter>;
}
