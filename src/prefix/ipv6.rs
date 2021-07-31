use std::error::Error;
use std::fmt;
// use std::hash;
use std::str::FromStr;

use ipnet::{AddrParseError, Ipv6Net, PrefixLenError};

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
    type Err = AddrParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(s.parse::<Ipv6Net>()?.into())
    }
}

// impl PartialEq for Ipv6Prefix {
//     fn eq(&self, other: &Self) -> bool {
//         self.bits() == other.bits() && self.length() == other.length()
//     }
// }

// impl hash::Hash for Ipv6Prefix {
//     fn hash<H>(&self, state: &mut H)
//     where
//         H: hash::Hasher,
//     {
//         self.bits().hash(state);
//         self.length().hash(state);
//     }
// }

impl fmt::Display for Ipv6Prefix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.to_ipnet().fmt(f)
    }
}

impl IpPrefix for Ipv6Prefix {
    type BitMap = u128;
    const MAX_LENGTH: u8 = 128;

    fn new(addr: Self::BitMap, length: u8) -> Result<Self, Box<dyn Error>> {
        if length > Self::MAX_LENGTH {
            return Err(Box::new(PrefixLenError));
        }
        Ok(Self { bits: addr, length })
    }

    fn bits(&self) -> Self::BitMap {
        self.bits
    }

    fn length(&self) -> u8 {
        self.length
    }
}
