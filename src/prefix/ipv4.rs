use std::error::Error;
use std::fmt;
use std::hash;
use std::str::FromStr;

use ipnet::{AddrParseError, Ipv4Net};

use super::IpPrefix;

#[derive(Clone, Copy, Debug, Eq)]
pub struct Ipv4Prefix {
    ipnet: Ipv4Net,
    bits: u32,
    length: u8,
}

impl From<Ipv4Net> for Ipv4Prefix {
    fn from(ipnet: Ipv4Net) -> Self {
        Self {
            ipnet,
            bits: ipnet.network().into(),
            length: ipnet.prefix_len(),
        }
    }
}

impl FromStr for Ipv4Prefix {
    type Err = AddrParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(s.parse::<Ipv4Net>()?.into())
    }
}

impl PartialEq for Ipv4Prefix {
    fn eq(&self, other: &Self) -> bool {
        self.bits() == other.bits() && self.length() == other.length()
    }
}

impl hash::Hash for Ipv4Prefix {
    fn hash<H>(&self, state: &mut H)
    where
        H: hash::Hasher,
    {
        self.bits().hash(state);
        self.length().hash(state);
    }
}

impl fmt::Display for Ipv4Prefix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.ipnet)
    }
}

impl IpPrefix for Ipv4Prefix {
    type BitMap = u32;
    const MAX_LENGTH: u8 = 32;

    fn new(addr: Self::BitMap, length: u8) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            ipnet: Ipv4Net::new(addr.into(), length)?,
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
        Ok(Ipv4Net::new(self.ipnet.network(), length)?.trunc().into())
    }
}
