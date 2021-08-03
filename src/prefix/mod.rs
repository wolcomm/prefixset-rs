//! Traits and types defining IP prefix objects that can be contained in a
//! [`PrefixSet<P>`](crate::PrefixSet).
use std::cmp::min;
use std::convert::TryInto;
use std::fmt;
use std::hash::Hash;
use std::iter::Sum;
use std::ops::{AddAssign, BitAndAssign, BitOrAssign, Shl, ShlAssign, Shr, ShrAssign};
use std::str::FromStr;

use num::{
    traits::{CheckedShl, CheckedShr},
    One, PrimInt, Zero,
};

use crate::error::{Error, Result};

/// The result of comparing prefixes with [`IpPrefix::compare_with()`].
///
/// The contained values are the number of bits shared in common between the
/// two prefixes.
pub enum Comparison {
    /// `self` and `other` are equal.
    Equal,
    /// `other` is a sub-prefix of `self`.
    Subprefix(u8),
    /// `other` is a super-prefix of `self`.
    Superprefix(u8),
    /// `other` is neither sub- nor super-prefix of `self`.
    Divergent(u8),
}

/// Provides bounds and methods for types `T` over which a
/// [`PrefixSet<T>`](crate::PrefixSet) can be constructed.
///
/// Default implementations are provided for [IPv4](crate::Ipv4Prefix) and
/// [IPv6](crate::Ipv6Prefix) prefixes.
///
/// Users are unlikely to need to implement this directly. It is provided in the
/// public API mainly to allow writing code that is generic over address family.
pub trait IpPrefix
where
    Self: fmt::Debug
        + fmt::Display
        + Clone
        + Copy
        + FromStr<Err = Error>
        + PartialEq
        + Eq
        + Hash
        + PartialOrd,
    Self::Bits: PrimInt
        + Default
        + CheckedShl
        + CheckedShr
        + fmt::Binary
        + AddAssign
        + BitAndAssign
        + BitOrAssign
        + std::fmt::Debug
        + Shl<u8, Output = Self::Bits>
        + ShlAssign<u8>
        + Shr<u8, Output = Self::Bits>
        + ShrAssign<u8>
        + Sum,
{
    /// The type used to represent the IP prefix as a bit string.
    type Bits;

    /// The maximum length in bits of an IP prefix.
    const MAX_LENGTH: u8;

    /// Construct a new `IpPrefix`.
    fn new(addr: Self::Bits, length: u8) -> Result<Self>;
    /// Get the bit string representation of the IP prefix.
    fn bits(&self) -> Self::Bits;
    /// Get the length of the IP prefix in bits.
    fn length(&self) -> u8;

    /// Construct a new `IpPrefix`, with the upper `length` bits equal to
    /// `self`, and the given prefix length.
    fn new_from(&self, length: u8) -> Result<Self> {
        let mask = (!Self::Bits::zero())
            .checked_shl((Self::MAX_LENGTH - length).into())
            .unwrap_or_default();
        Self::new(self.bits() & mask, length)
    }

    /// Iterator over the sub-prefixes of `self` having prefix length `length`.
    /// Consumes `self`.
    fn into_subprefixes(self, length: u8) -> IntoSubprefixes<Self> {
        IntoSubprefixes::new(self, length)
    }

    /// Iterator over the sub-prefixes of `self` having prefix length `length`.
    /// Doesn't consume `self`.
    fn subprefixes(&self, length: u8) -> Subprefixes<Self> {
        Subprefixes::new(self, length)
    }

    /// Compare `self` with another prefix to determine whether one is a
    /// super-prefix of the other.
    fn compare_with(&self, other: &Self) -> Comparison {
        let min_lens = min(self.length(), other.length());
        let diff_map = self.bits() ^ other.bits();
        let common = min(min_lens, diff_map.leading_zeros().try_into().unwrap());
        if common == self.length() && common == other.length() {
            Comparison::Equal
        } else if common == self.length() && common < other.length() {
            Comparison::Subprefix(common)
        } else if common < self.length() && common == other.length() {
            Comparison::Superprefix(common)
        } else if common < self.length() && common < other.length() {
            Comparison::Divergent(common)
        } else {
            unreachable!("Common cannot be larger than either prefix length")
        }
    }
}

macro_rules! impl_partial_ord {
    ( $type:ty ) => {
        impl PartialOrd for $type
        where
            Self: IpPrefix,
        {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                match self.compare_with(other) {
                    $crate::prefix::Comparison::Equal => Some(std::cmp::Ordering::Equal),
                    $crate::prefix::Comparison::Subprefix(_) => Some(std::cmp::Ordering::Greater),
                    $crate::prefix::Comparison::Superprefix(_) => Some(std::cmp::Ordering::Less),
                    $crate::prefix::Comparison::Divergent(_) => None,
                }
            }
        }
    };
}

/// A consuming iterator over the sub-prefixes of an [`IpPrefix`].
#[derive(Debug)]
pub struct IntoSubprefixes<P: IpPrefix> {
    base: P,
    length: u8,
    max_index: P::Bits,
    step: P::Bits,
    next_index: P::Bits,
}

impl<P: IpPrefix> IntoSubprefixes<P> {
    fn new(base: P, length: u8) -> Self {
        let max_index = (!P::Bits::zero())
            .checked_shr((P::MAX_LENGTH - length + base.length()).into())
            .unwrap_or_default();
        let step = P::Bits::one()
            .checked_shl((P::MAX_LENGTH - length).into())
            .unwrap_or_default();
        Self {
            base,
            length,
            max_index,
            step,
            next_index: P::Bits::zero(),
        }
    }
}

impl<P: IpPrefix> Iterator for IntoSubprefixes<P> {
    type Item = P;

    fn next(&mut self) -> Option<Self::Item> {
        if !(self.base.length() <= self.length && self.length <= P::MAX_LENGTH) {
            return None;
        }
        if self.next_index <= self.max_index {
            let addr = self.base.bits() + (self.next_index * self.step);
            self.next_index += P::Bits::one();
            // safe to unwrap here, since we checked length above
            Some(P::new(addr, self.length).unwrap())
        } else {
            None
        }
    }
}

/// A non-consuming iterator over the sub-prefixes of an [`IpPrefix`].
#[derive(Debug)]
pub struct Subprefixes<'a, P: IpPrefix> {
    base: &'a P,
    length: u8,
    max_index: P::Bits,
    step: P::Bits,
    next_index: P::Bits,
}

impl<'a, P: IpPrefix> Subprefixes<'a, P> {
    fn new(base: &'a P, length: u8) -> Self {
        let max_index = (!P::Bits::zero())
            .checked_shr((P::MAX_LENGTH - length + base.length()).into())
            .unwrap_or_default();
        let step = P::Bits::one()
            .checked_shl((P::MAX_LENGTH - length).into())
            .unwrap_or_default();
        Self {
            base,
            length,
            max_index,
            step,
            next_index: P::Bits::zero(),
        }
    }
}

impl<P: IpPrefix> Iterator for Subprefixes<'_, P> {
    type Item = P;

    fn next(&mut self) -> Option<Self::Item> {
        if !(self.base.length() <= self.length && self.length <= P::MAX_LENGTH) {
            return None;
        }
        if self.next_index <= self.max_index {
            let addr = self.base.bits() + (self.next_index * self.step);
            self.next_index += P::Bits::one();
            // safe to unwrap here, since we checked length above
            Some(P::new(addr, self.length).unwrap())
        } else {
            None
        }
    }
}

pub use ipv4::Ipv4Prefix;
pub use ipv6::Ipv6Prefix;
pub use range::IpPrefixRange;

mod ipv4;
mod ipv6;
pub mod range;

#[cfg(test)]
mod tests;
