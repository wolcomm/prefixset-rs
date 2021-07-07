use std::ops::BitOrAssign;
use std::str::FromStr;

use ipnet::{PrefixLenError, Ipv4Net, Ipv6Net};
use num::PrimInt;

pub trait IpPrefixAggregate: std::fmt::Debug + Copy + FromStr {
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
