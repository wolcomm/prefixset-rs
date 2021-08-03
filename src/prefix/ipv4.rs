use std::fmt;
use std::str::FromStr;

use ipnet::{Ipv4Net, PrefixLenError};

use crate::error::{Error, Result};

use super::IpPrefix;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Ipv4Prefix {
    bits: u32,
    length: u8,
}

impl Ipv4Prefix {
    fn to_ipnet(self) -> Ipv4Net {
        Ipv4Net::new(self.bits().into(), self.length()).unwrap()
    }
}

impl From<Ipv4Net> for Ipv4Prefix {
    fn from(ipnet: Ipv4Net) -> Self {
        Self {
            bits: ipnet.network().into(),
            length: ipnet.prefix_len(),
        }
    }
}

impl FromStr for Ipv4Prefix {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Ok(s.parse::<Ipv4Net>()?.into())
    }
}

impl fmt::Display for Ipv4Prefix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.to_ipnet().fmt(f)
    }
}

impl IpPrefix for Ipv4Prefix {
    type Bits = u32;
    const MAX_LENGTH: u8 = 32;

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

impl_partial_ord!(Ipv4Prefix);
