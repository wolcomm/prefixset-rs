use std::fmt;
use std::iter::Sum;
use std::ops::{
    BitAnd,
    BitAndAssign,
    BitOr,
    BitOrAssign,
    Not,
    Add,
    Mul,
    Shl,
    ShlAssign,
    Shr,
    ShrAssign,
};

use num::{
    PrimInt,
    One,
    Zero,
    traits::{
        CheckedShl,
        CheckedShr,
    },
};

use crate::prefix::{IpPrefix, IpPrefixRange};

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct GlueMap<P: IpPrefix> {
    bitmap: P::BitMap,
    hostbit: bool,
}

impl<P: IpPrefix> Zero for GlueMap<P> {
    fn zero() -> Self {
        Self {
            bitmap: P::BitMap::zero(),
            hostbit: false,
        }
    }

    fn is_zero(&self) -> bool {
        self.bitmap == P::BitMap::zero() && !self.hostbit
    }
}

impl<P: IpPrefix> One for GlueMap<P> {
    fn one() -> Self {
        Self {
            bitmap: !P::BitMap::zero(),
            hostbit: true,
        }
    }
}

impl<P: IpPrefix> BitAnd for GlueMap<P> {
    type Output = Self;

    fn bitand(self, other: Self) -> Self::Output {
        Self {
            bitmap: self.bitmap & other.bitmap,
            hostbit: self.hostbit && other.hostbit,
        }
    }
}

impl<P: IpPrefix> BitOr for GlueMap<P> {
    type Output = Self;

    fn bitor(self, other: Self) -> Self::Output {
        Self {
            bitmap: self.bitmap | other.bitmap,
            hostbit: self.hostbit || other.hostbit,
        }
    }
}

impl<P: IpPrefix> Not for GlueMap<P> {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self {
            bitmap: !self.bitmap,
            hostbit: !self.hostbit,
        }
    }
}

impl<P: IpPrefix> BitAndAssign for GlueMap<P> {
    fn bitand_assign(&mut self, other: Self) {
        *self = *self & other
    }
}

impl<P: IpPrefix> BitOrAssign for GlueMap<P> {
    fn bitor_assign(&mut self, other: Self) {
        *self = *self | other
    }
}

impl<P: IpPrefix> Add for GlueMap<P> {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        self | other
    }
}

impl<P: IpPrefix> Mul for GlueMap<P> {
    type Output = Self;

    fn mul(self, other: Self) -> Self::Output {
        self & other
    }
}

impl<P: IpPrefix> Shl<u8> for GlueMap<P> {
    type Output = Self;

    fn shl(self, rhs: u8) -> Self::Output {
        match self.bitmap.checked_shl(rhs.into()) {
            Some(result) => Self {
                bitmap: result,
                hostbit: self.hostbit.to_owned(),
            },
            None => Self {
                bitmap: P::BitMap::zero(),
                hostbit: true,
            }
        }
    }
}

impl<P: IpPrefix> ShlAssign<u8> for GlueMap<P> {
    fn shl_assign(&mut self, rhs: u8) {
        *self = *self << rhs
    }
}

impl<P: IpPrefix> Shr<u8> for GlueMap<P> {
    type Output = Self;

    fn shr(self, rhs: u8) -> Self::Output {
        let (hostbit, shifted_hostbit) = if self.hostbit && rhs > 0 {
            (false, if rhs > P::MAX_LENGTH {
                P::BitMap::zero()
            } else {
                P::BitMap::one() << P::MAX_LENGTH - rhs
            })
        } else {
            (self.hostbit, P::BitMap::zero())
        };
        let shifted_bitmap = match self.bitmap.checked_shr(rhs.into()) {
            Some(result) => result,
            None => P::BitMap::zero()
        };
        Self {
            bitmap: shifted_bitmap + shifted_hostbit,
            hostbit,
        }
    }
}

impl<P: IpPrefix> ShrAssign<u8> for GlueMap<P> {
    fn shr_assign(&mut self, rhs: u8) {
        *self = *self >> rhs
    }
}

impl<P: IpPrefix> Sum for GlueMap<P> {
    fn sum<I>(iter: I) -> Self
    where
        I: Iterator<Item=Self>
    {
        iter.fold(Self::zero(), |acc, item| {acc + item})
    }
}

impl<P: IpPrefix> GlueMap<P> {
    pub fn singleton(length: u8) -> Self {
        Self {
            bitmap: P::BitMap::one(),
            hostbit: false,
        } << length
    }

    pub fn trailing_zeros(self) -> u32 {
        let zeros = self.bitmap.trailing_zeros();
        if zeros == P::MAX_LENGTH.into() && !self.hostbit {
            zeros + 1
        } else {
            zeros
        }
    }

    pub fn count_ones(self) -> u32 {
        let ones = self.bitmap.count_ones();
        if self.hostbit {
            ones + 1
        } else {
            ones
        }
    }
}

impl<P: IpPrefix> From<IpPrefixRange<P>> for GlueMap<P> {
    fn from(r: IpPrefixRange<P>) -> Self {
        r.range()
            .map(|l| { Self::singleton(l) })
            .sum()
    }
}

impl<P: IpPrefix> fmt::Debug for GlueMap<P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GlueMap")
            .field(
                "bitmap",
                &format_args!(
                    "{:#0w$b}",
                    &self.bitmap,
                    w = (P::MAX_LENGTH + 2).into()
                )
            )
            .field("hostbit", &self.hostbit)
            .finish()
    }
}
