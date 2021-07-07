use std::ops::BitOrAssign;
use std::str::FromStr;

use ipnet::{AddrParseError, PrefixLenError, Ipv4Net, Ipv6Net};
use num::PrimInt;

pub trait IpPrefixAggregate
where
    Self: std::fmt::Debug + Copy + FromStr,
    Self::AggrMap: PrimInt + BitOrAssign<Self::AggrMap> + std::fmt::Debug,
{
    type AggrMap;

    const MAX_LENGTH: u8;

    fn bits(&self) -> Self::AggrMap;

    fn length(&self) -> u8;

    fn new_from(&self, lenth: u8) -> Result<Self, PrefixLenError>;
}

#[derive(Clone, Copy, Debug)]
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

impl IpPrefixAggregate for Ipv4Prefix {
    type AggrMap = u32;
    const MAX_LENGTH: u8 = 32;

    fn bits(&self) -> Self::AggrMap {
        self.bits
    }

    fn length(&self) -> u8 {
        self.length
    }

    fn new_from(&self, length: u8) -> Result<Self, PrefixLenError> {
        Ok(Ipv4Net::new(self.ipnet.network(), length)?.trunc().into())
    }
}

#[derive(Clone, Copy, Debug)]
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

impl IpPrefixAggregate for Ipv6Prefix {
    type AggrMap = u128;
    const MAX_LENGTH: u8 = 128;

    fn bits(&self) -> Self::AggrMap {
        self.bits
    }

    fn length(&self) -> u8 {
        self.length
    }

    fn new_from(&self, length: u8) -> Result<Self, PrefixLenError> {
        Ok(Ipv6Net::new(self.ipnet.network(), length)?.trunc().into())
    }
}

#[cfg(test)]
mod tests {

    use crate::tests::TestResult;
    use super::{IpPrefixAggregate, Ipv4Prefix, Ipv6Prefix};

    mod ipv4_prefix_from_str {
        use super::*;

        fn setup() -> Ipv4Prefix {
            "10.0.0.0/8".parse().unwrap()
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
            "2001:db8::/32".parse().unwrap()
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
