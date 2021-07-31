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
    Self::BitMap: PrimInt
        + CheckedShl
        + CheckedShr
        + Binary
        + AddAssign
        + BitAndAssign
        + BitOrAssign
        + std::fmt::Debug
        + Shl<u8, Output = Self::BitMap>
        + ShlAssign
        + Shr<u8, Output = Self::BitMap>
        + ShrAssign<u8>
        + Sum,
{
    type BitMap;

    const MAX_LENGTH: u8;

    fn new(addr: Self::BitMap, length: u8) -> Result<Self, Box<dyn Error>>;
    fn bits(&self) -> Self::BitMap;
    fn length(&self) -> u8;

    fn new_from(&self, length: u8) -> Result<Self, Box<dyn Error>> {
        let mask = match (!Self::BitMap::zero()).checked_shl((Self::MAX_LENGTH - length).into()) {
            Some(m) => m,
            None => Self::BitMap::zero(),
        };
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
    max_index: P::BitMap,
    step: P::BitMap,
    next_index: P::BitMap,
}

impl<P, Q> SubPrefixesIntoIter<P, Q>
where
    P: IpPrefix,
    Q: Borrow<P>,
{
    fn new(base: Q, length: u8) -> Self {
        let max_index = match (!P::BitMap::zero())
            .checked_shr((P::MAX_LENGTH - length + base.borrow().length()).into())
        {
            Some(i) => i,
            None => P::BitMap::zero(),
        };
        let step = match P::BitMap::one().checked_shl((P::MAX_LENGTH - length).into()) {
            Some(s) => s,
            None => P::BitMap::zero(),
        };
        Self {
            base,
            length,
            max_index,
            step,
            next_index: P::BitMap::zero(),
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
            self.next_index += P::BitMap::one();
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
