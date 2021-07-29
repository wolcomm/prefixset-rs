use num::Zero;

use crate::tests::{assert_none, assert_some, TestResult};
use crate::{IpPrefix, IpPrefixRange, Ipv4Prefix, Ipv6Prefix};

use super::{GlueMap, Node};

fn subtree_size<P: IpPrefix>(root: Box<Node<P>>) -> usize {
    let mut i: usize = 0;
    root.walk(&mut |_| i += 1);
    i
}

fn is_glue<P: IpPrefix>(node: &Box<Node<P>>) -> bool {
    node.gluemap.is_zero()
}

mod subtree_of_three_prefixes {
    use super::*;

    fn setup() -> Box<Node<Ipv4Prefix>> {
        let n1 = Box::new(Node::new_singleton("10.1.0.0/16".parse().unwrap()));
        let n2 = Box::new(Node::new_singleton("10.2.0.0/16".parse().unwrap()));
        let n3 = Box::new(Node::new_singleton("10.3.0.0/16".parse().unwrap()));
        n1.add(n2).add(n3)
    }

    #[test]
    fn contains_singletons_and_glue() -> TestResult {
        let n = setup();
        assert!(is_glue(&n));
        let (l, r) = (n.left.unwrap(), n.right.unwrap());
        assert_eq!(l.gluemap, GlueMap::singleton(16));
        assert_none(l.left)?;
        assert_none(l.right)?;
        assert!(is_glue(&r));
        assert_eq!(r.left.unwrap().gluemap, GlueMap::singleton(16));
        assert_eq!(r.right.unwrap().gluemap, GlueMap::singleton(16));
        Ok(())
    }

    mod after_aggregation {
        use super::*;

        fn setup() -> Box<Node<Ipv4Prefix>> {
            super::setup().aggregate()
        }

        #[test]
        fn equal_length_siblings_aggregate() -> TestResult {
            let n = setup();
            assert!(is_glue(&n));
            let (l, r) = (n.left.unwrap(), n.right.unwrap());
            assert_eq!(l.gluemap, GlueMap::singleton(16));
            assert_none(l.left)?;
            assert_none(l.right)?;
            assert_eq!(r.gluemap, GlueMap::singleton(16));
            assert!(is_glue(&r.left.unwrap()));
            assert!(is_glue(&r.right.unwrap()));
            Ok(())
        }

        mod after_compression {
            use super::*;

            fn setup() -> Box<Node<Ipv4Prefix>> {
                super::setup().compress().unwrap()
            }

            #[test]
            fn leaf_glue_nodes_are_removed() -> TestResult {
                let n = setup();
                assert!(is_glue(&n));
                let (l, r) = (n.left.unwrap(), n.right.unwrap());
                assert_eq!(l.gluemap, GlueMap::singleton(16));
                assert_none(l.left)?;
                assert_none(l.right)?;
                assert_eq!(r.gluemap, GlueMap::singleton(16));
                assert_none(r.left)?;
                assert_none(r.right)?;
                Ok(())
            }
        }
    }
}

mod new_ipv4_singleton {
    use super::*;

    fn setup() -> Box<Node<Ipv4Prefix>> {
        let p = "192.0.2.0/24".parse().unwrap();
        Box::new(Node::new_singleton(p))
    }

    #[test]
    fn has_none_children() -> TestResult {
        let n = setup();
        assert_none(n.left)?;
        assert_none(n.right)?;
        Ok(())
    }

    #[test]
    fn has_singleton_gluemap() -> TestResult {
        let n = setup();
        assert_eq!(n.gluemap.count_ones(), 1);
        Ok(())
    }

    #[test]
    fn becomes_glue_after_removal() -> TestResult {
        let n = setup();
        assert!(is_glue(&n.remove(&mut Node::new_singleton(
            "192.0.2.0/24".parse().unwrap()
        ))));
        Ok(())
    }

    mod added_with_self {
        use super::*;

        fn setup() -> Box<Node<Ipv4Prefix>> {
            let n = super::setup();
            let m = super::setup();
            n.add(m)
        }

        #[test]
        fn is_unchanged() -> TestResult {
            let n = super::setup();
            let m = setup();
            assert_eq!(n, m);
            Ok(())
        }

        #[test]
        fn has_subtree_size_one() -> TestResult {
            let n = setup();
            assert_eq!(subtree_size(n), 1);
            Ok(())
        }
    }

    mod added_with_host_subprefix {
        use super::*;

        fn setup() -> Box<Node<Ipv4Prefix>> {
            let n = super::setup();
            let q = "192.0.2.192/32".parse().unwrap();
            let m = Box::new(Node::new_singleton(q));
            n.add(m)
        }

        #[test]
        fn returns_same_root() -> TestResult {
            let n = super::setup();
            let m = setup();
            assert_eq!(n, m);
            Ok(())
        }

        #[test]
        fn has_some_right_child() -> TestResult {
            let n = setup();
            assert_some(n.right)
        }

        #[test]
        fn has_none_left_child() -> TestResult {
            let n = setup();
            assert_none(n.left)
        }

        #[test]
        fn has_subtree_size_two() -> TestResult {
            let n = setup();
            assert_eq!(subtree_size(n), 2);
            Ok(())
        }

        #[test]
        fn becomes_glue_after_removal() -> TestResult {
            let n = setup();
            assert!(is_glue(&n.remove(&mut Node::new_singleton(
                "192.0.2.0/24".parse().unwrap()
            ))));
            Ok(())
        }
    }

    mod added_with_subprefix {
        use super::*;

        fn setup() -> Box<Node<Ipv4Prefix>> {
            let n = super::setup();
            let q = "192.0.2.192/26".parse().unwrap();
            let m = Box::new(Node::new_singleton(q));
            n.add(m)
        }

        #[test]
        fn returns_same_root() -> TestResult {
            let n = super::setup();
            let m = setup();
            assert_eq!(n, m);
            Ok(())
        }

        #[test]
        fn has_some_right_child() -> TestResult {
            let n = setup();
            assert_some(n.right)
        }

        #[test]
        fn has_none_left_child() -> TestResult {
            let n = setup();
            assert_none(n.left)
        }

        #[test]
        fn has_subtree_size_two() -> TestResult {
            let n = setup();
            assert_eq!(subtree_size(n), 2);
            Ok(())
        }
    }

    mod added_with_superprefix {
        use super::*;

        fn setup() -> Box<Node<Ipv4Prefix>> {
            let n = super::setup();
            let q = "192.0.0.0/16".parse().unwrap();
            let m = Box::new(Node::new_singleton(q));
            n.add(m)
        }

        #[test]
        fn returns_new_root() -> TestResult {
            let n = super::setup();
            let m = setup();
            assert_ne!(n, m);
            Ok(())
        }

        #[test]
        fn has_some_left_child() -> TestResult {
            let n = setup();
            assert_some(n.left)
        }

        #[test]
        fn has_none_right_child() -> TestResult {
            let n = setup();
            assert_none(n.right)
        }

        #[test]
        fn has_subtree_size_two() -> TestResult {
            let n = setup();
            assert_eq!(subtree_size(n), 2);
            Ok(())
        }

        #[test]
        fn is_unchanged_after_subprefix_removal() -> TestResult {
            let n = setup();
            let m = n
                .clone()
                .remove(&mut Node::new_singleton("192.0.2.0/24".parse().unwrap()));
            println!("{:#?}", m);
            assert_eq!(m, n);
            Ok(())
        }
    }

    mod added_with_sibling {
        use super::*;

        fn setup() -> Box<Node<Ipv4Prefix>> {
            let n = super::setup();
            let q = "192.0.3.0/24".parse().unwrap();
            let m = Box::new(Node::new_singleton(q));
            n.add(m)
        }

        #[test]
        fn returns_new_root() -> TestResult {
            let n = super::setup();
            let m = setup();
            assert_ne!(n, m);
            Ok(())
        }

        #[test]
        fn has_some_left_child() -> TestResult {
            let n = setup();
            assert_some(n.left)
        }

        #[test]
        fn has_some_right_child() -> TestResult {
            let n = setup();
            assert_some(n.right)
        }

        #[test]
        fn is_glue() -> TestResult {
            let n = setup();
            assert!(n.gluemap.is_zero());
            Ok(())
        }

        #[test]
        fn has_subtree_size_three() -> TestResult {
            let n = setup();
            assert_eq!(subtree_size(n), 3);
            Ok(())
        }

        #[test]
        fn can_iter() -> TestResult {
            let n = setup();
            assert_eq!(n.into_iter().count(), subtree_size(n));
            Ok(())
        }

        mod after_aggregation {
            use super::*;

            fn setup() -> Box<Node<Ipv4Prefix>> {
                super::setup().aggregate()
            }

            #[test]
            fn is_aggregate() -> TestResult {
                let n = setup();
                assert_eq!(n.gluemap, GlueMap::singleton(24));
                Ok(())
            }

            #[test]
            fn is_glue_after_subprefix_removal() -> TestResult {
                let mut n = setup();
                let mut r = Node::new_range(
                    IpPrefixRange::new("192.0.2.0/23".parse().unwrap(), 24, 24).unwrap(),
                );
                n = n.remove(&mut r);
                println!("{:#?}", n);
                assert!(is_glue(&n));
                Ok(())
            }
        }
    }

    mod added_with_divergent {
        use super::*;

        fn setup() -> Box<Node<Ipv4Prefix>> {
            let n = super::setup();
            let q = "192.168.0.0/16".parse().unwrap();
            let m = Box::new(Node::new_singleton(q));
            n.add(m)
        }

        #[test]
        fn returns_new_root() -> TestResult {
            let n = super::setup();
            let m = setup();
            assert_ne!(n, m);
            Ok(())
        }

        #[test]
        fn has_some_left_child() -> TestResult {
            let n = setup();
            assert_some(n.left)
        }

        #[test]
        fn has_some_right_child() -> TestResult {
            let n = setup();
            assert_some(n.right)
        }

        #[test]
        fn is_glue() -> TestResult {
            let n = setup();
            assert!(n.gluemap.is_zero());
            Ok(())
        }

        #[test]
        fn has_subtree_size_three() -> TestResult {
            let n = setup();
            assert_eq!(subtree_size(n), 3);
            Ok(())
        }

        #[test]
        fn can_iter() -> TestResult {
            let n = setup();
            assert_eq!(n.into_iter().count(), subtree_size(n));
            Ok(())
        }

        mod after_aggregation {
            use super::*;

            fn setup() -> Box<Node<Ipv4Prefix>> {
                super::setup().aggregate()
            }

            #[test]
            fn is_glue() -> TestResult {
                let n = setup();
                assert!(n.gluemap.is_zero());
                Ok(())
            }
        }
    }
}

mod new_ipv6_singleton {
    use super::*;

    fn setup() -> Box<Node<Ipv6Prefix>> {
        let p = "2001:db8:f00::/48".parse().unwrap();
        Box::new(Node::new_singleton(p))
    }

    #[test]
    fn has_none_children() -> TestResult {
        let n = setup();
        assert_none(n.left)?;
        assert_none(n.right)?;
        Ok(())
    }

    #[test]
    fn has_singleton_gluemap() -> TestResult {
        let n = setup();
        assert_eq!(n.gluemap.count_ones(), 1);
        Ok(())
    }

    mod added_with_self {
        use super::*;

        fn setup() -> Box<Node<Ipv6Prefix>> {
            let n = super::setup();
            let m = super::setup();
            n.add(m)
        }

        #[test]
        fn is_unchanged() -> TestResult {
            let n = super::setup();
            let m = setup();
            assert_eq!(n, m);
            Ok(())
        }

        #[test]
        fn has_subtree_size_one() -> TestResult {
            let n = setup();
            assert_eq!(subtree_size(n), 1);
            Ok(())
        }

        #[test]
        fn can_iter() -> TestResult {
            let n = setup();
            assert_eq!(n.into_iter().count(), subtree_size(n));
            Ok(())
        }
    }

    mod added_with_host_subprefix {
        use super::*;

        fn setup() -> Box<Node<Ipv6Prefix>> {
            let n = super::setup();
            let q = "2001:db8:f00:baa::/128".parse().unwrap();
            let m = Box::new(Node::new_singleton(q));
            n.add(m)
        }

        #[test]
        fn returns_same_root() -> TestResult {
            let n = super::setup();
            let m = setup();
            assert_eq!(n, m);
            Ok(())
        }

        #[test]
        fn has_some_left_child() -> TestResult {
            let n = setup();
            assert_some(n.left)
        }

        #[test]
        fn has_none_right_child() -> TestResult {
            let n = setup();
            assert_none(n.right)
        }

        #[test]
        fn has_subtree_size_two() -> TestResult {
            let n = setup();
            assert_eq!(subtree_size(n), 2);
            Ok(())
        }

        #[test]
        fn can_iter() -> TestResult {
            let n = setup();
            assert_eq!(n.into_iter().count(), subtree_size(n));
            Ok(())
        }
    }

    mod added_with_subprefix {
        use super::*;

        fn setup() -> Box<Node<Ipv6Prefix>> {
            let n = super::setup();
            let q = "2001:db8:f00:baa::/64".parse().unwrap();
            let m = Box::new(Node::new_singleton(q));
            n.add(m)
        }

        #[test]
        fn returns_same_root() -> TestResult {
            let n = super::setup();
            let m = setup();
            assert_eq!(n, m);
            Ok(())
        }

        #[test]
        fn has_some_left_child() -> TestResult {
            let n = setup();
            assert_some(n.left)
        }

        #[test]
        fn has_none_right_child() -> TestResult {
            let n = setup();
            assert_none(n.right)
        }

        #[test]
        fn has_subtree_size_two() -> TestResult {
            let n = setup();
            assert_eq!(subtree_size(n), 2);
            Ok(())
        }

        #[test]
        fn can_iter() -> TestResult {
            let n = setup();
            assert_eq!(n.into_iter().count(), subtree_size(n));
            Ok(())
        }
    }

    mod added_with_superprefix {
        use super::*;

        fn setup() -> Box<Node<Ipv6Prefix>> {
            let n = super::setup();
            let q = "2001:db8::/36".parse().unwrap();
            let m = Box::new(Node::new_singleton(q));
            n.add(m)
        }

        #[test]
        fn returns_new_root() -> TestResult {
            let n = super::setup();
            let m = setup();
            assert_ne!(n, m);
            Ok(())
        }

        #[test]
        fn has_some_right_child() -> TestResult {
            let n = setup();
            assert_some(n.right)
        }

        #[test]
        fn has_none_left_child() -> TestResult {
            let n = dbg!(setup());
            assert_none(n.left)
        }

        #[test]
        fn has_subtree_size_two() -> TestResult {
            let n = setup();
            assert_eq!(subtree_size(n), 2);
            Ok(())
        }

        #[test]
        fn can_iter() -> TestResult {
            let n = setup();
            assert_eq!(n.into_iter().count(), subtree_size(n));
            Ok(())
        }
    }

    mod added_with_sibling {
        use super::*;

        fn setup() -> Box<Node<Ipv6Prefix>> {
            let n = super::setup();
            let q = "2001:db8:baa::/48".parse().unwrap();
            let m = Box::new(Node::new_singleton(q));
            n.add(m)
        }

        #[test]
        fn returns_new_root() -> TestResult {
            let n = super::setup();
            let m = setup();
            assert_ne!(n, m);
            Ok(())
        }

        #[test]
        fn has_some_left_child() -> TestResult {
            let n = setup();
            assert_some(n.left)
        }

        #[test]
        fn has_some_right_child() -> TestResult {
            let n = setup();
            assert_some(n.right)
        }

        #[test]
        fn is_glue() -> TestResult {
            let n = setup();
            assert!(n.gluemap.is_zero());
            Ok(())
        }

        #[test]
        fn has_subtree_size_two() -> TestResult {
            let n = setup();
            assert_eq!(subtree_size(n), 3);
            Ok(())
        }

        #[test]
        fn can_iter() -> TestResult {
            let n = setup();
            assert_eq!(n.into_iter().count(), subtree_size(n));
            Ok(())
        }
    }
}
