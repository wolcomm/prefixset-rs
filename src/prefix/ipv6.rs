use std::error::Error;
use std::fmt;
use std::str::FromStr;

use ipnet::{AddrParseError, Ipv6Net};

use super::IpPrefix;

#[derive(Clone, Copy, Debug, Eq, Hash)]
pub struct Ipv6Prefix {
    ipnet: Ipv6Net,
    bits: u128,
    length: u8,
}

impl From<Ipv6Net> for Ipv6Prefix {
    fn from(ipnet: Ipv6Net) -> Self {
        Self {
            ipnet,
            bits: ipnet.network().into(),
            length: ipnet.prefix_len(),
        }
    }
}

impl FromStr for Ipv6Prefix {
    type Err = AddrParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(s.parse::<Ipv6Net>()?.into())
    }
}

impl PartialEq for Ipv6Prefix {
    fn eq(&self, other: &Self) -> bool {
        self.bits() == other.bits() && self.length() == other.length()
    }
}

impl fmt::Display for Ipv6Prefix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.ipnet)
    }
}

impl IpPrefix for Ipv6Prefix {
    type BitMap = u128;
    const MAX_LENGTH: u8 = 128;

    fn new(addr: Self::BitMap, length: u8) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            ipnet: Ipv6Net::new(addr.into(), length)?,
            bits: addr,
            length,
        })
    }

    fn bits(&self) -> Self::BitMap {
        self.bits
    }

    fn length(&self) -> u8 {
        self.length
    }

    fn new_from(&self, length: u8) -> Result<Self, Box<dyn Error>> {
        Ok(Ipv6Net::new(self.ipnet.network(), length)?.trunc().into())
    }
}
