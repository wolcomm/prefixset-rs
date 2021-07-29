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

    fn new_from(&self, length: u8) -> Result<Self, Box<dyn Error>>;

    fn iter_subprefixes(&self, length: u8) -> SubPrefixesIter<Self> {
        SubPrefixesIter {
            base: self,
            length,
            next_index: Self::BitMap::zero(),
        }
    }

    fn into_iter_subprefixes(self, length: u8) -> SubPrefixesIntoIter<Self> {
        SubPrefixesIntoIter {
            base: self,
            length,
            next_index: Self::BitMap::zero(),
            max_index: match (!Self::BitMap::zero())
                .checked_shr((Self::MAX_LENGTH - length + self.length()).into())
            {
                Some(i) => i,
                None => Self::BitMap::zero(),
            },
            step: match Self::BitMap::one().checked_shl((Self::MAX_LENGTH - length).into()) {
                Some(s) => s,
                None => Self::BitMap::zero(),
            },
        }
    }
}

#[derive(Debug)]
pub struct SubPrefixesIntoIter<P: IpPrefix> {
    base: P,
    length: u8,
    next_index: P::BitMap,
    max_index: P::BitMap,
    step: P::BitMap,
}

impl<P: IpPrefix> Iterator for SubPrefixesIntoIter<P> {
    type Item = P;

    fn next(&mut self) -> Option<Self::Item> {
        if !(self.base.length() <= self.length && self.length <= P::MAX_LENGTH) {
            return None;
        }
        // dbg!(&self);
        // let max_index = P::BitMap::one() << (self.length - self.base.length());
        // let max_index = (!P::BitMap::zero()).checked_shr(P::MAX_LENGTH + self.base.length() - self.length);
        if self.next_index <= self.max_index {
            // let addr = if self.next_index.is_zero() {
            //     self.base.bits()
            // } else {
            //     let step = P::BitMap::one() << (P::MAX_LENGTH - self.length);
            //     self.base.bits() + (self.next_index * step)
            // };
            let addr = self.base.bits() + (self.next_index * self.step);
            self.next_index += P::BitMap::one();
            // safe to unwrap here, since we checked length above
            Some(P::new(addr, self.length).unwrap())
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct SubPrefixesIter<'a, P: IpPrefix> {
    base: &'a P,
    length: u8,
    next_index: P::BitMap,
}

impl<'a, P: IpPrefix> Iterator for SubPrefixesIter<'a, P> {
    type Item = P;

    fn next(&mut self) -> Option<Self::Item> {
        if !(self.base.length() <= self.length && self.length <= P::MAX_LENGTH) {
            return None;
        }
        let max_index = P::BitMap::one() << (self.length - self.base.length());
        if self.next_index < max_index {
            let step = P::BitMap::one() << (P::MAX_LENGTH - self.length);
            let addr = self.base.bits() + (self.next_index * step);
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
