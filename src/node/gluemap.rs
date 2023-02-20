use std::fmt;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not, Shl, ShlAssign, Shr, ShrAssign};

use ip::{traits::primitive::Address, Afi};

use num::{
    traits::{CheckedShl, CheckedShr},
    One, PrimInt, Zero,
};

type Length<A> = <<A as Afi>::Primitive as Address<A>>::Length;

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct GlueMap<A: Afi> {
    bitmap: A::Primitive,
    hostbit: bool,
}

// TODO unused?
impl<A: Afi> Zero for GlueMap<A> {
    fn zero() -> Self {
        Self {
            bitmap: A::Primitive::ZERO,
            hostbit: false,
        }
    }

    fn is_zero(&self) -> bool {
        self.bitmap == A::Primitive::ZERO && !self.hostbit
    }
}

// TODO unused?
impl<A: Afi> One for GlueMap<A> {
    fn one() -> Self {
        Self {
            bitmap: A::Primitive::ONES,
            hostbit: true,
        }
    }
}

impl<A: Afi> BitAnd for GlueMap<A> {
    type Output = Self;

    fn bitand(self, other: Self) -> Self::Output {
        Self {
            bitmap: self.bitmap & other.bitmap,
            hostbit: self.hostbit && other.hostbit,
        }
    }
}

impl<A: Afi> BitOr for GlueMap<A> {
    type Output = Self;

    fn bitor(self, other: Self) -> Self::Output {
        Self {
            bitmap: self.bitmap | other.bitmap,
            hostbit: self.hostbit || other.hostbit,
        }
    }
}

impl<A: Afi> Not for GlueMap<A> {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self {
            bitmap: !self.bitmap,
            hostbit: !self.hostbit,
        }
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

impl<A: Afi> Shl<u8> for GlueMap<A>
where
    // TODO
    A::Primitive: CheckedShl,
{
    type Output = Self;

    fn shl(self, rhs: u8) -> Self::Output {
        match self.bitmap.checked_shl(rhs.into()) {
            Some(result) => Self {
                bitmap: result,
                hostbit: self.hostbit.to_owned(),
            },
            None => Self {
                bitmap: A::Primitive::ZERO,
                hostbit: true,
            },
        }
    }
}

impl<A: Afi> ShlAssign<u8> for GlueMap<A>
where
    // TODO
    A::Primitive: CheckedShl,
{
    fn shl_assign(&mut self, rhs: u8) {
        *self = *self << rhs
    }
}

impl<A: Afi> Shr<u8> for GlueMap<A>
where
    // TODO
    A::Primitive: CheckedShr + Address<A, Length = u8>,
{
    type Output = Self;

    fn shr(self, rhs: u8) -> Self::Output {
        let (hostbit, shifted_hostbit) = if self.hostbit && rhs > 0 {
            (
                false,
                if A::Primitive::MAX_LENGTH < rhs {
                    A::Primitive::ZERO
                } else {
                    // TODO: BUG! this should be 0x0001 not 0x1111
                    A::Primitive::ONES << (A::Primitive::MAX_LENGTH - rhs)
                },
            )
        } else {
            (self.hostbit, A::Primitive::ZERO)
        };
        let shifted_bitmap = match self.bitmap.checked_shr(rhs.into()) {
            Some(result) => result,
            None => A::Primitive::ZERO,
        };
        Self {
            bitmap: shifted_bitmap | shifted_hostbit,
            hostbit,
        }
    }
}

impl<A: Afi> ShrAssign<u8> for GlueMap<A>
where
    A::Primitive: CheckedShr + Address<A, Length = u8>,
{
    fn shr_assign(&mut self, rhs: u8) {
        *self = *self >> rhs
    }
}

impl<A: Afi> GlueMap<A> {
    pub(crate) const ZERO: Self = Self {
        bitmap: A::Primitive::ZERO,
        hostbit: false,
    };

    pub fn singleton(length: u8) -> Self
    where
        // TODO
        A::Primitive: CheckedShl,
    {
        Self {
            // TODO: BUG!
            bitmap: A::Primitive::ONES,
            hostbit: false,
        } << length
    }

    pub fn trailing_zeros(self) -> u32
    where
        // TODO
        A::Primitive: PrimInt + Address<A, Length = u8>,
    {
        let zeros = self.bitmap.trailing_zeros();
        if zeros == A::Primitive::MAX_LENGTH as u32 && !self.hostbit {
            zeros + 1
        } else {
            zeros
        }
    }

    pub fn count_ones(self) -> u32
    where
        // TODO
        A::Primitive: PrimInt,
    {
        let ones = self.bitmap.count_ones();
        if self.hostbit {
            ones + 1
        } else {
            ones
        }
    }
}

impl<A: Afi> From<ip::concrete::PrefixRange<A>> for GlueMap<A>
where
    ip::PrefixLength<A>: AsRef<u8>,
    A::Primitive: CheckedShl,
{
    fn from(range: ip::PrefixRange<A>) -> Self {
        (*range.lower().as_ref()..=*range.upper().as_ref())
            .into_iter()
            .map(Self::singleton)
            .fold(Self::ZERO, |acc, item| acc | item)
    }
}

impl<A: Afi> fmt::Debug for GlueMap<A>
where
    // TODO
    A::Primitive: std::fmt::Binary + Address<A, Length = u8>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GlueMap")
            .field(
                "bitmap",
                &format_args!(
                    "{:#0w$b}",
                    &self.bitmap,
                    w = (A::Primitive::MAX_LENGTH + 2).into()
                ),
            )
            .field("hostbit", &self.hostbit)
            .finish()
    }
}
