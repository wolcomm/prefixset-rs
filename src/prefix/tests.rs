use super::{IpPrefix, Ipv4Prefix, Ipv6Prefix};
use crate::tests::TestResult;

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

    #[test]
    fn has_expected_subprefixes() -> TestResult {
        let p = setup();
        assert_eq!(p.subprefixes(16).count(), 256,);
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

    #[test]
    fn has_expected_subprefixes() -> TestResult {
        let p = setup();
        assert_eq!(p.subprefixes(48).count(), 1 << 16,);
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
