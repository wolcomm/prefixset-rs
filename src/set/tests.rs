use crate::tests::{assert_none, TestResult};
use crate::{IpPrefixRange, Ipv4Prefix};

use super::PrefixSet;

mod new_ipv4_prefix_set {
    use super::*;

    fn setup() -> PrefixSet<Ipv4Prefix> {
        PrefixSet::new()
    }

    #[test]
    fn is_emtpy() -> TestResult {
        let s = setup();
        assert_none(s.root)
    }

    #[test]
    fn contains_no_prefixes() -> TestResult {
        let s = setup();
        assert_eq!(s.iter_prefixes().count(), 0);
        assert!(!s.contains("192.0.2.0/24".parse()?));
        assert!(!s.contains("192.0.0.0/22".parse()?));
        assert!(!s.contains("192.0.2.128/25".parse()?));
        assert!(!s.contains("192.0.4.0/24".parse()?));
        Ok(())
    }

    #[test]
    fn iter_over_prefixes_is_empty() -> TestResult {
        let s = setup();
        let c: Vec<_> = s.iter_prefixes().collect();
        assert_eq!(Vec::<Ipv4Prefix>::new(), c);
        Ok(())
    }

    mod with_a_prefix_added {
        use super::*;

        fn setup() -> PrefixSet<Ipv4Prefix> {
            let mut s = super::setup();
            let p = "192.0.2.0/24".parse().unwrap();
            s.add_prefix(p).to_owned()
        }

        #[test]
        fn contains_one_prefix() -> TestResult {
            let s = setup();
            println!("{:#?}", s);
            assert_eq!(s.iter_prefixes().count(), 1);
            Ok(())
        }

        #[test]
        fn contains_that_prefix() -> TestResult {
            let s = setup();
            assert!(s.contains("192.0.2.0/24".parse()?));
            Ok(())
        }

        #[test]
        fn does_not_contain_others() -> TestResult {
            let s = setup();
            assert!(!s.contains("192.0.0.0/22".parse()?));
            assert!(!s.contains("192.0.2.128/25".parse()?));
            assert!(!s.contains("192.0.4.0/24".parse()?));
            Ok(())
        }

        #[test]
        fn iter_over_prefixes_is_singleton() -> TestResult {
            let s = setup();
            let c: Vec<_> = s.iter_prefixes().collect();
            assert_eq!(vec!["192.0.2.0/24".parse::<Ipv4Prefix>()?], c);
            Ok(())
        }

        mod and_removed {
            use super::*;

            fn setup() -> PrefixSet<Ipv4Prefix> {
                let mut s = super::setup();
                let p = "192.0.2.0/24".parse().unwrap();
                s.remove_prefix(p).to_owned()
            }

            #[test]
            fn is_emtpy() -> TestResult {
                let s = setup();
                assert_none(s.root)
            }
        }

        mod with_another_prefix_added {
            use super::*;

            fn setup() -> PrefixSet<Ipv4Prefix> {
                let mut s = super::setup();
                let p = "192.0.0.0/22".parse().unwrap();
                s.add_prefix(p).to_owned()
            }

            #[test]
            fn contains_two_prefixes() -> TestResult {
                let s = setup();
                assert_eq!(s.iter_prefixes().count(), 2);
                Ok(())
            }

            #[test]
            fn contains_both_prefixes() -> TestResult {
                let s = setup();
                assert!(s.contains("192.0.2.0/24".parse()?));
                assert!(s.contains("192.0.0.0/22".parse()?));
                Ok(())
            }

            #[test]
            fn iter_over_prefixes_is_len_two() -> TestResult {
                let s = setup();
                let c: Vec<_> = s.iter_prefixes().collect();
                assert_eq!(
                    vec![
                        "192.0.0.0/22".parse::<Ipv4Prefix>()?,
                        "192.0.2.0/24".parse::<Ipv4Prefix>()?,
                    ],
                    c
                );
                Ok(())
            }

            mod and_a_range_removed {
                use super::*;

                fn setup() -> PrefixSet<Ipv4Prefix> {
                    let mut s = super::setup();
                    let p = "192.0.0.0/16".parse().unwrap();
                    let r = IpPrefixRange::new(p, 24, 24).unwrap();
                    s.remove_prefix_range(r).to_owned()
                }

                #[test]
                fn contains_one_prefix() -> TestResult {
                    let s = setup();
                    println!("{:#?}", s);
                    assert_eq!(s.iter_prefixes().count(), 1);
                    Ok(())
                }

                #[test]
                fn contains_the_remaining_prefix() -> TestResult {
                    let s = setup();
                    assert!(s.contains("192.0.0.0/22".parse()?));
                    Ok(())
                }
            }

            mod with_a_third_prefix_added {
                use super::*;

                fn setup() -> PrefixSet<Ipv4Prefix> {
                    let mut s = super::setup();
                    let p = "192.0.3.0/24".parse().unwrap();
                    s.add_prefix(p).to_owned()
                }

                #[test]
                fn contains_three_prefixes() -> TestResult {
                    let s = setup();
                    assert_eq!(s.iter_prefixes().count(), 3);
                    Ok(())
                }

                #[test]
                fn contains_two_prefix_ranges() -> TestResult {
                    let s = setup();
                    assert_eq!(s.iter_prefix_ranges().count(), 2);
                    Ok(())
                }

                #[test]
                fn contains_all_prefixes() -> TestResult {
                    let s = setup();
                    println!("{:#?}", s);
                    assert!(s.contains("192.0.2.0/24".parse()?));
                    assert!(s.contains("192.0.3.0/24".parse()?));
                    assert!(s.contains("192.0.0.0/22".parse()?));
                    Ok(())
                }

                #[test]
                fn iter_over_prefixes_is_len_three() -> TestResult {
                    let s = setup();
                    let c: Vec<_> = s.iter_prefixes().collect();
                    assert_eq!(
                        vec![
                            "192.0.0.0/22".parse::<Ipv4Prefix>()?,
                            "192.0.2.0/24".parse::<Ipv4Prefix>()?,
                            "192.0.3.0/24".parse::<Ipv4Prefix>()?,
                        ],
                        c
                    );
                    Ok(())
                }

                mod and_a_range_removed {
                    use super::*;

                    fn setup() -> PrefixSet<Ipv4Prefix> {
                        let mut s = super::setup();
                        let p = "192.0.2.0/23".parse().unwrap();
                        let r = IpPrefixRange::new(p, 24, 24).unwrap();
                        s.remove_prefix_range(r).to_owned()
                    }

                    #[test]
                    fn contains_one_prefix() -> TestResult {
                        let s = setup();
                        println!("{:#?}", s);
                        assert_eq!(s.iter_prefixes().count(), 1);
                        Ok(())
                    }

                    #[test]
                    fn contains_the_remaining_prefix() -> TestResult {
                        let s = setup();
                        assert!(s.contains("192.0.0.0/22".parse()?));
                        Ok(())
                    }
                }
            }
        }
    }
}
