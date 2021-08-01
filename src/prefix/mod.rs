use std::borrow::Borrow;
use std::error::Error;
use std::fmt::Binary;
use std::hash::Hash;
use std::iter::Sum;
use std::ops::{AddAssign, BitAndAssign, BitOrAssign, Shl, ShlAssign, Shr, ShrAssign};
use std::str::FromStr;

use ipnet::AddrParseError;

use num::{
    traits::{CheckedShl, CheckedShr},
    One, PrimInt, Zero,
};

pub trait IpPrefix
where
    Self: std::fmt::Debug
        + std::fmt::Display
        + Copy
        + FromStr<Err = AddrParseError>
        + PartialEq
        + Eq
        + Hash,
    Self::Bits: PrimInt
        + Default
        + CheckedShl
        + CheckedShr
        + Binary
        + AddAssign
        + BitAndAssign
        + BitOrAssign
        + std::fmt::Debug
        + Shl<u8, Output = Self::Bits>
        + ShlAssign
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

    fn iter_subprefixes(&self, length: u8) -> SubPrefixesIntoIter<Self, &Self> {
        SubPrefixesIntoIter::new(self, length)
    }

    fn into_iter_subprefixes(self, length: u8) -> SubPrefixesIntoIter<Self, Self> {
        SubPrefixesIntoIter::new(self, length)
    }
}

#[derive(Debug)]
pub struct SubPrefixesIntoIter<P, Q>
where
    P: IpPrefix,
    Q: Borrow<P>,
{
    base: Q,
    length: u8,
    max_index: P::Bits,
    step: P::Bits,
    next_index: P::Bits,
}

impl<P, Q> SubPrefixesIntoIter<P, Q>
where
    P: IpPrefix,
    Q: Borrow<P>,
{
    fn new(base: Q, length: u8) -> Self {
        let max_index = (!P::Bits::zero())
            .checked_shr((P::MAX_LENGTH - length + base.borrow().length()).into())
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

impl<P, Q> Iterator for SubPrefixesIntoIter<P, Q>
where
    P: IpPrefix,
    Q: Borrow<P>,
{
    type Item = P;

    fn next(&mut self) -> Option<Self::Item> {
        if !(self.base.borrow().length() <= self.length && self.length <= P::MAX_LENGTH) {
            return None;
        }
        if self.next_index <= self.max_index {
            let addr = self.base.borrow().bits() + (self.next_index * self.step);
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
pub use range::{IpPrefixRange, IpPrefixRangeIntoIter, IpPrefixRangeIter};

mod ipv4;
mod ipv6;
mod range;

#[cfg(test)]
mod tests;
