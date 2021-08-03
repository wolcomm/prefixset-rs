use std::fmt;
use std::str::FromStr;

use ipnet::{Ipv6Net, PrefixLenError};

use crate::error::{Error, Result};

use super::IpPrefix;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Ipv6Prefix {
    bits: u128,
    length: u8,
}

impl Ipv6Prefix {
    fn to_ipnet(self) -> Ipv6Net {
        Ipv6Net::new(self.bits().into(), self.length()).unwrap()
    }
}

impl From<Ipv6Net> for Ipv6Prefix {
    fn from(ipnet: Ipv6Net) -> Self {
        Self {
            bits: ipnet.network().into(),
            length: ipnet.prefix_len(),
        }
    }
}

impl FromStr for Ipv6Prefix {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Ok(s.parse::<Ipv6Net>()?.into())
    }
}

impl fmt::Display for Ipv6Prefix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.to_ipnet().fmt(f)
    }
}

impl IpPrefix for Ipv6Prefix {
    type Bits = u128;
    const MAX_LENGTH: u8 = 128;

    fn new(addr: Self::Bits, length: u8) -> Result<Self> {
        if length > Self::MAX_LENGTH {
            return Err(Error::PrefixLen(PrefixLenError));
        }
        Ok(Self { bits: addr, length })
    }

    fn bits(&self) -> Self::Bits {
        self.bits
    }

    fn length(&self) -> u8 {
        self.length
    }
}

impl_partial_ord!(Ipv6Prefix);
