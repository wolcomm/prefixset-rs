use std::convert::TryFrom;
use std::ops::BitOrAssign;
use std::str::FromStr;

use ipnet::{PrefixLenError, AddrParseError, IpNet, Ipv4Net, Ipv6Net};
use num::PrimInt;

pub enum IpPrefix {
    IPv4(Ipv4Prefix),
    IPv6(Ipv6Prefix),
}

impl FromStr for IpPrefix {
    type Err = AddrParseError;

    fn from_str(s: &str) -> Result<Self, AddrParseError> {
        let net = IpNet::from_str(s)?;
        match net {
            IpNet::V4(net) => Ok(Self::IPv4(net)),
            IpNet::V6(net) => Ok(Self::IPv6(net)),
        }
    }
}

pub trait IpPrefixAggregate: TryFrom<IpPrefix> + std::fmt::Debug + Copy + FromStr {
    type AggrMap:
        PrimInt +
        BitOrAssign<Self::AggrMap> +
        std::fmt::Debug;

    const MAX_LENGTH: u8;

    fn bits(&self) -> Self::AggrMap;

    fn length(&self) -> u8;

    fn new_from(&self, lenth: u8) -> Result<Self, PrefixLenError>;
}

pub type Ipv4Prefix = Ipv4Net;

impl TryFrom<IpPrefix> for Ipv4Prefix {
    type Error = &'static str;
    fn try_from(p: IpPrefix) -> Result<Self, Self::Error> {
        match p {
            IpPrefix::IPv4(p) => Ok(p.trunc()),
            _ => Err("address family mismatch: expected IPv4 prefix")
        }
    }
}

impl IpPrefixAggregate for Ipv4Prefix {
    type AggrMap = u32;
    const MAX_LENGTH: u8 = 32;

    fn bits(&self) -> Self::AggrMap {
        self.network().into()
    }

    fn length(&self) -> u8 {
        self.prefix_len()
    }

    fn new_from(&self, length: u8) -> Result<Self, PrefixLenError> {
        Ok(Self::new(self.network(), length)?.trunc())
    }
}

pub type Ipv6Prefix = Ipv6Net;

impl TryFrom<IpPrefix> for Ipv6Prefix {
    type Error = &'static str;
    fn try_from(p: IpPrefix) -> Result<Self, Self::Error> {
        match p {
            IpPrefix::IPv6(p) => Ok(p.trunc()),
            _ => Err("address family mismatch: expected IPv6 prefix")
        }
    }
}

impl IpPrefixAggregate for Ipv6Prefix {
    type AggrMap = u128;
    const MAX_LENGTH: u8 = 128;

    fn bits(&self) -> Self::AggrMap {
        self.network().into()
    }

    fn length(&self) -> u8 {
        self.prefix_len()
    }

    fn new_from(&self, length: u8) -> Result<Self, PrefixLenError> {
        Ok(Self::new(self.network(), length)?.trunc())
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryInto;
    use std::str::FromStr;

    use crate::tests::TestResult;
    use super::{IpPrefix, IpPrefixAggregate, Ipv4Prefix, Ipv6Prefix};

    mod ipv4_prefix_from_str {
        use super::*;

        fn setup() -> Ipv4Prefix {
            IpPrefix::from_str("10.0.0.0/8")
                .unwrap()
                .try_into()
                .unwrap()
        }

        #[test]
        fn has_correct_bits() -> TestResult {
            let p = setup();
            assert_eq!(p.bits(), 0x0a000000);
            Ok(())
        }

        #[test]
        fn has_correct_length() -> TestResult {
            let p = setup();
            assert_eq!(p.length(), 8);
            Ok(())
        }

        mod to_superprefix {
            use super::*;

            fn setup() -> Ipv4Prefix {
                let p = super::setup();
                p.new_from(12).unwrap()
            }

            #[test]
            fn has_common_prefix() -> TestResult {
                let p = super::setup();
                let q = setup();
                assert!((p.bits() ^ q.bits()).leading_zeros() >= 12);
                Ok(())
            }

            #[test]
            fn has_correct_length() -> TestResult {
                let p = setup();
                assert_eq!(p.length(), 12);
                Ok(())
            }
        }
    }

    mod ipv6_prefix_from_str {
        use super::*;

        fn setup() -> Ipv6Prefix {
            IpPrefix::from_str("2001:db8::/32")
                .unwrap()
                .try_into()
                .unwrap()
        }

        #[test]
        fn has_correct_bits() -> TestResult {
            let p = setup();
            assert_eq!(p.bits(), 0x20010db8000000000000000000000000);
            Ok(())
        }

        #[test]
        fn has_correct_length() -> TestResult {
            let p = setup();
            assert_eq!(p.length(), 32);
            Ok(())
        }

        mod to_superprefix {
            use super::*;

            fn setup() -> Ipv6Prefix {
                let p = super::setup();
                p.new_from(36).unwrap()
            }

            #[test]
            fn has_common_prefix() -> TestResult {
                let p = super::setup();
                let q = setup();
                assert!((p.bits() ^ q.bits()).leading_zeros() >= 36);
                Ok(())
            }

            #[test]
            fn has_correct_length() -> TestResult {
                let p = setup();
                assert_eq!(p.length(), 36);
                Ok(())
            }
        }
    }
}
