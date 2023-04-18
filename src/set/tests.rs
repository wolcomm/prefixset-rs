use ip::{Ipv4, Prefix, PrefixRange};

use crate::tests::TestResult;
// use crate::{IpPrefixRange, Ipv4Prefix};

use super::PrefixSet;

fn assert_send<T: Send>(_: T) -> TestResult {
    Ok(())
}
fn assert_sync<T: Sync>(_: T) -> TestResult {
    Ok(())
}

mod new_ipv4_prefix_set {
    use super::*;

    fn setup() -> PrefixSet<Ipv4> {
        PrefixSet::new()
    }

    #[test]
    fn is_send() -> TestResult {
        let s = setup();
        assert_send(s)
    }

    #[test]
    fn is_sync() -> TestResult {
        let s = setup();
        assert_sync(s)
    }

    #[test]
    fn is_emtpy() -> TestResult {
        let s = setup();
        assert!(s.root.is_none());
        Ok(())
    }

    #[test]
    fn contains_no_prefixes() -> TestResult {
        let s = setup();
        assert_eq!(s.prefixes().count(), 0);
        assert!(!s.contains("192.0.2.0/24".parse()?));
        assert!(!s.contains("192.0.0.0/22".parse()?));
        assert!(!s.contains("192.0.2.128/25".parse()?));
        assert!(!s.contains("192.0.4.0/24".parse()?));
        Ok(())
    }

    #[test]
    fn iter_over_prefixes_is_empty() -> TestResult {
        let s = setup();
        let c: Vec<_> = s.prefixes().collect();
        assert_eq!(Vec::<Prefix<Ipv4>>::new(), c);
        Ok(())
    }

    mod with_a_prefix_added {
        use super::*;

        fn setup() -> PrefixSet<Ipv4> {
            let mut s = super::setup();
            let p = "192.0.2.0/24".parse::<Prefix<Ipv4>>().unwrap();
            s.insert(p).to_owned()
        }

        #[test]
        fn contains_one_prefix() -> TestResult {
            let s = setup();
            println!("{:#?}", s);
            assert_eq!(s.prefixes().count(), 1);
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
            let c: Vec<Prefix<Ipv4>> = s.prefixes().collect();
            assert_eq!(vec!["192.0.2.0/24".parse::<Prefix<Ipv4>>()?], c);
            Ok(())
        }

        mod and_removed {
            use super::*;

            fn setup() -> PrefixSet<Ipv4> {
                let mut s = super::setup();
                let p = "192.0.2.0/24".parse::<Prefix<Ipv4>>().unwrap();
                s.remove(p).to_owned()
            }

            #[test]
            fn is_emtpy() -> TestResult {
                let s = setup();
                assert!(s.root.is_none());
                Ok(())
            }
        }

        mod with_another_prefix_added {
            use super::*;

            fn setup() -> PrefixSet<Ipv4> {
                let mut s = super::setup();
                let p = "192.0.0.0/22".parse::<Prefix<Ipv4>>().unwrap();
                s.insert(p).to_owned()
            }

            #[test]
            fn contains_two_prefixes() -> TestResult {
                let s = setup();
                assert_eq!(s.prefixes().count(), 2);
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
                let c: Vec<_> = s.prefixes().collect();
                assert_eq!(
                    vec![
                        "192.0.0.0/22".parse::<Prefix<Ipv4>>()?,
                        "192.0.2.0/24".parse::<Prefix<Ipv4>>()?,
                    ],
                    c
                );
                Ok(())
            }

            mod and_a_range_removed {
                use super::*;

                fn setup() -> PrefixSet<Ipv4> {
                    let mut s = super::setup();
                    let r: PrefixRange<Ipv4> = "192.0.0.0/16,24,24".parse().unwrap();
                    s.remove(r).to_owned()
                }

                #[test]
                fn contains_one_prefix() -> TestResult {
                    let s = setup();
                    dbg!(&s);
                    assert_eq!(s.prefixes().count(), 1);
                    Ok(())
                }

                #[test]
                fn contains_the_remaining_prefix() -> TestResult {
                    let s = setup();
                    assert!(&s.contains("192.0.0.0/22".parse()?));
                    Ok(())
                }
            }

            mod with_a_third_prefix_added {
                use super::*;

                fn setup() -> PrefixSet<Ipv4> {
                    let mut s = super::setup();
                    let p: Prefix<Ipv4> = "192.0.3.0/24".parse().unwrap();
                    s.insert(p).to_owned()
                }

                #[test]
                fn contains_three_prefixes() -> TestResult {
                    let s = setup();
                    assert_eq!(s.prefixes().count(), 3);
                    Ok(())
                }

                #[test]
                fn contains_two_prefix_ranges() -> TestResult {
                    let s = setup();
                    assert_eq!(s.ranges().count(), 2);
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
                    let c: Vec<_> = s.prefixes().collect();
                    assert_eq!(
                        vec![
                            "192.0.0.0/22".parse::<Prefix<Ipv4>>()?,
                            "192.0.2.0/24".parse::<Prefix<Ipv4>>()?,
                            "192.0.3.0/24".parse::<Prefix<Ipv4>>()?,
                        ],
                        c
                    );
                    Ok(())
                }

                mod and_a_range_removed {
                    use super::*;

                    fn setup() -> PrefixSet<Ipv4> {
                        let mut s = super::setup();
                        let r: PrefixRange<Ipv4> = "192.0.2.0/23,24,24".parse().unwrap();
                        s.remove(r).to_owned()
                    }

                    #[test]
                    fn contains_one_prefix() -> TestResult {
                        let s = setup();
                        println!("{:#?}", s);
                        assert_eq!(s.prefixes().count(), 1);
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
