use std::cmp::min;
use std::convert::TryInto;
use std::error::Error;
use std::fmt;
use std::hash::Hash;
use std::iter::Sum;
use std::ops::{AddAssign, BitAndAssign, BitOrAssign, Shl, ShlAssign, Shr, ShrAssign};
use std::str::FromStr;

use ipnet::AddrParseError;

use num::{
    traits::{CheckedShl, CheckedShr},
    One, PrimInt, Zero,
};

pub enum Comparison {
    Equal,
    Subprefix(u8),
    Superprefix(u8),
    Divergent(u8),
}

pub trait IpPrefix
where
    Self: fmt::Debug
        + fmt::Display
        + Clone
        + Copy
        + FromStr<Err = AddrParseError>
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
    type Bits;

    const MAX_LENGTH: u8;

    fn new(addr: Self::Bits, length: u8) -> Result<Self, Box<dyn Error>>;
    fn bits(&self) -> Self::Bits;
    fn length(&self) -> u8;

    fn new_from(&self, length: u8) -> Result<Self, Box<dyn Error>> {
        let mask = (!Self::Bits::zero())
            .checked_shl((Self::MAX_LENGTH - length).into())
            .unwrap_or_default();
        Self::new(self.bits() & mask, length)
    }

    fn into_subprefixes(self, length: u8) -> IntoSubprefixes<Self> {
        IntoSubprefixes::new(self, length)
    }

    fn subprefixes(&self, length: u8) -> Subprefixes<Self> {
        Subprefixes::new(self, length)
    }

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
