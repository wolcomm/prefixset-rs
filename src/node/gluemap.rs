use std::fmt;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not, RangeInclusive};

use ip::{
    concrete::{PrefixLength, PrefixRange},
    traits::{
        primitive::{Address, LengthMap as _},
        PrefixLength as _,
    },
    Afi,
};

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct GlueMap<A: Afi> {
    inner: <A::Primitive as Address<A>>::LengthMap,
}

impl<A: Afi> BitAnd for GlueMap<A> {
    type Output = Self;

    fn bitand(self, other: Self) -> Self::Output {
        let inner = self.inner.bitand(other.inner);
        Self { inner }
    }
}

impl<A: Afi> BitOr for GlueMap<A> {
    type Output = Self;

    fn bitor(self, other: Self) -> Self::Output {
        let inner = self.inner.bitor(other.inner);
        Self { inner }
    }
}

impl<A: Afi> Not for GlueMap<A> {
    type Output = Self;

    fn not(self) -> Self::Output {
        let inner = self.inner.not();
        Self { inner }
    }
}

impl<A: Afi> BitAndAssign for GlueMap<A> {
    fn bitand_assign(&mut self, other: Self) {
        *self = *self & other
    }
}

impl<A: Afi> BitOrAssign for GlueMap<A> {
    fn bitor_assign(&mut self, other: Self) {
        *self = *self | other
    }
}

impl<A: Afi> Default for GlueMap<A> {
    fn default() -> Self {
        Self::ZERO
    }
}

impl<A: Afi> GlueMap<A> {
    pub const ZERO: Self = Self {
        inner: <A::Primitive as Address<A>>::LengthMap::ZERO,
    };

    const MAX: <A::Primitive as Address<A>>::Length = <A::Primitive as Address<A>>::MAX_LENGTH;

    pub fn singleton(length: PrefixLength<A>) -> Self {
        let mut inner: <A::Primitive as Address<A>>::LengthMap = Default::default();
        inner.set(length.into_primitive().into(), true);
        Self { inner }
    }

    pub fn count_ones(&self) -> usize {
        self.inner.count_ones()
    }

    pub fn next_range(&self, from: PrefixLength<A>) -> Option<RangeInclusive<PrefixLength<A>>> {
        let start = from.into_primitive().into();
        let first = start + self.inner[start..].first_one()?;
        let last = match self.inner[first..].first_zero() {
            Some(len) => first + len - 1,
            None => Self::MAX.into(),
        };
        // Ok to unwrap because indices of Self are within the bounds
        // of `PrefixLength<A>`
        let lower = first.try_into().unwrap();
        let upper = last.try_into().unwrap();
        Some(lower..=upper)
    }
}

impl<A: Afi> From<PrefixRange<A>> for GlueMap<A> {
    fn from(range: PrefixRange<A>) -> Self {
        let mut map = Self::ZERO;
        let mut length = range.lower();
        loop {
            map |= Self::singleton(length);
            match length.increment() {
                Ok(next) if next <= range.upper() => length = next,
                _ => break map,
            }
        }
    }
}

impl<A: Afi> fmt::Debug for GlueMap<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("GlueMap")
            .field(&format_args!(
                "{:#0w$b}",
                &self.inner,
                w = A::Primitive::MAX_LENGTH.into() as usize + 2
            ))
            .finish()
    }
}
