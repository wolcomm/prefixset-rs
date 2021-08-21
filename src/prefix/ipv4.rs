use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;

use ipnet::{IpNet, Ipv4Net, PrefixLenError};

use crate::error::{Error, Result};

use super::IpPrefix;

/// An implementation of [`IpPrefix`] for the IPv4 address family.
///
/// ``` rust
/// # use prefixset::{Error, IpPrefix, Ipv4Prefix};
/// # fn main() -> Result<(), Error> {
/// let p: Ipv4Prefix = "192.0.2.0/24".parse()?;
/// assert_eq!(p.length(), 24);
/// # Ok(())
/// # }
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

impl TryFrom<IpNet> for Ipv4Prefix {
    type Error = Error;

    fn try_from(n: IpNet) -> Result<Self> {
        if let IpNet::V4(ipnet) = n {
            Ok(ipnet.into())
        } else {
            Err(Error::AddressFamiltMismatch)
        }
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
