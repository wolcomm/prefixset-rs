use std::cmp::min;
use std::convert::TryInto;
use std::error::Error;

use num::{PrimInt, One, Zero};

use crate::prefix::IpPrefixAggregate;

enum Comparison {
    Equal,
    ChildOf(u8),
    ParentOf(u8),
    Divergent(u8),
}

enum Direction {
    Left,
    Right,
}

#[derive(Debug)]
pub struct Node<P: IpPrefixAggregate> {
    prefix: P,
    left: Option<Box<Node<P>>>,
    right: Option<Box<Node<P>>>,
    gluemap: P::AggrMap,
}

impl<P: IpPrefixAggregate> Node<P> {
    fn new(prefix: P, gluemap: P::AggrMap) -> Self {
        Node{
            prefix,
            gluemap,
            left: None,
            right: None,
        }
    }

    pub fn new_singleton(prefix: P) -> Self {
        let gluemap = P::AggrMap::one() << prefix.length().into();
        Self::new(prefix, gluemap)
    }

    fn new_glue(prefix: P) -> Self {
        Self::new(prefix, P::AggrMap::zero())
    }

    pub fn prefix(&self) -> &P {
        &self.prefix
    }

    pub fn add(mut self, mut other: Self) -> Result<Self, Box<dyn Error>> {
        match self.compare_with(&other)? {
            Comparison::Equal => {
                self.add_glue_from(&other);
                Ok(self)
            },
            Comparison::ChildOf(common) => {
                match other.branch_direction(common) {
                    Direction::Left => {
                        if let Some(child) = self.left {
                            let new_child = child.add(other)?;
                            self.left = Some(Box::new(new_child));
                        } else {
                            self.left = Some(Box::new(other));
                        }
                    },
                    Direction::Right => {
                        if let Some(child) = self.right {
                            let new_child = child.add(other)?;
                            self.right = Some(Box::new(new_child));
                        } else {
                            self.right = Some(Box::new(other));
                        }
                    },
                };
                Ok(self)
            },
            Comparison::ParentOf(common) => {
                match self.branch_direction(common) {
                    Direction::Left => {
                        other.left = Some(Box::new(self));
                    },
                    Direction::Right => {
                        other.right = Some(Box::new(self));
                    }
                };
                Ok(other)
            }
            Comparison::Divergent(common) => {
                let glue_prefix = self.prefix.new_from(common)?;
                let mut glue = Self::new_glue(glue_prefix);
                match self.branch_direction(common) {
                    Direction::Left => {
                        glue.left = Some(Box::new(self));
                        glue.right = Some(Box::new(other));
                    },
                    Direction::Right => {
                        glue.left = Some(Box::new(other));
                        glue.right = Some(Box::new(self));
                    }
                };
                Ok(glue)
            },
        }
    }

    pub fn search(&self, qnode: &Self) -> Result<Option<&Self>, Box<dyn Error>> {
        match self.compare_with(qnode)? {
            Comparison::Equal => Ok(Some(self)),
            Comparison::ChildOf(common) => {
                match qnode.branch_direction(common) {
                    Direction::Left => {
                        if let Some(child) = &self.left {
                            child.search(qnode)
                        } else {
                            Ok(None)
                        }
                    },
                    Direction::Right => {
                        if let Some(child) = &self.right {
                            child.search(qnode)
                        } else {
                            Ok(None)
                        }
                    },
                }
            },
            _ => Ok(None)
        }
    }

    pub fn walk(&self, f: &mut impl FnMut(&Self)) {
        f(self);
        if let Some(child) = &self.left {
            child.walk(f);
        }
        if let Some(child) = &self.right {
            child.walk(f);
        }
    }

    fn add_glue_from(&mut self, other: &Self) {
        self.gluemap |= other.gluemap
    }

    pub fn is_glue(&self) -> bool {
        self.gluemap == P::AggrMap::zero()
    }

    fn branch_direction(&self, bit_index: u8) -> Direction {
        let next_index = bit_index + 1;
        let mask = P::AggrMap::one() << (P::MAX_LENGTH - next_index).into();
        if self.prefix.bits() & mask == P::AggrMap::zero() {
            Direction::Left
        } else {
            Direction::Right
        }
    }

    fn compare_with(&self, other: &Self) -> Result<Comparison, &'static str> {
        let min_lens = min(self.prefix.length(), other.prefix.length());
        let diff_map = self.prefix.bits() ^ other.prefix.bits();
        let common = min(min_lens, diff_map.leading_zeros().try_into().unwrap());
        if common == self.prefix.length() && common == other.prefix.length() {
            Ok(Comparison::Equal)
        } else if common == self.prefix.length() && common < other.prefix.length() {
            Ok(Comparison::ChildOf(common))
        } else if common < self.prefix.length() && common == other.prefix.length() {
            Ok(Comparison::ParentOf(common))
        } else if common < self.prefix.length() && common < other.prefix.length() {
            Ok(Comparison::Divergent(common))
        } else {
            Err("The maths is wrong!")
        }
    }
}

impl<P: IpPrefixAggregate> PartialEq for Node<P> {
    fn eq(&self, other: &Self) -> bool {
        (self.prefix.bits() == other.prefix.bits()) 
            && (self.prefix.length() == other.prefix.length())
            && (self.gluemap == other.gluemap)
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryInto;
    use std::str::FromStr;

    use crate::{IpPrefix, Ipv4Prefix, Ipv6Prefix};
    use crate::prefix::IpPrefixAggregate;
    use crate::tests::{assert_none, assert_some, TestResult};

    use super::Node;

    fn subtree_size<P: IpPrefixAggregate>(root: Node<P>) -> usize {
        let mut i: usize = 0;
        root.walk(&mut |_| i += 1);
        i
    }


    mod new_ipv4_singleton {
        use super::*;

        fn setup() -> Node<Ipv4Prefix> {
            let p = IpPrefix::from_str("192.0.2.0/24")
                .unwrap()
                .try_into()
                .unwrap();
            Node::new_singleton(p)
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

            fn setup() -> Node<Ipv4Prefix> {
                let n = super::setup();
                let m = super::setup();
                n.add(m).unwrap()
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

        mod added_with_subprefix {
            use super::*;

            fn setup() -> Node<Ipv4Prefix> {
                let n = super::setup();
                let q = IpPrefix::from_str("192.0.2.192/26")
                    .unwrap()
                    .try_into()
                    .unwrap();
                let m = Node::new_singleton(q);
                n.add(m).unwrap()
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

            fn setup() -> Node<Ipv4Prefix> {
                let n = super::setup();
                let q = IpPrefix::from_str("192.0.0.0/16")
                    .unwrap()
                    .try_into()
                    .unwrap();
                let m = Node::new_singleton(q);
                n.add(m).unwrap()
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
        }

        mod added_with_sibling {
            use super::*;

            fn setup() -> Node<Ipv4Prefix> {
                let n = super::setup();
                let q = IpPrefix::from_str("192.168.0.0/16")
                    .unwrap()
                    .try_into()
                    .unwrap();
                let m = Node::new_singleton(q);
                n.add(m).unwrap()
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
                assert_eq!(n.gluemap, 0);
                Ok(())
            }

            #[test]
            fn has_subtree_size_two() -> TestResult {
                let n = setup();
                assert_eq!(subtree_size(n), 3);
                Ok(())
            }
        }
    }

    mod new_ipv6_singleton {
        use super::*;

        fn setup() -> Node<Ipv6Prefix> {
            let p = IpPrefix::from_str("2001:db8:f00::/48")
                .unwrap()
                .try_into()
                .unwrap();
            Node::new_singleton(p)
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

            fn setup() -> Node<Ipv6Prefix> {
                let n = super::setup();
                let m = super::setup();
                n.add(m).unwrap()
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

        mod added_with_subprefix {
            use super::*;

            fn setup() -> Node<Ipv6Prefix> {
                let n = super::setup();
                let q = IpPrefix::from_str("2001:db8:f00:baa::/64")
                    .unwrap()
                    .try_into()
                    .unwrap();
                let m = Node::new_singleton(q);
                n.add(m).unwrap()
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
        }

        mod added_with_superprefix {
            use super::*;

            fn setup() -> Node<Ipv6Prefix> {
                let n = super::setup();
                let q = IpPrefix::from_str("2001:db8::/36")
                    .unwrap()
                    .try_into()
                    .unwrap();
                let m = Node::new_singleton(q);
                n.add(m).unwrap()
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

        mod added_with_sibling {
            use super::*;

            fn setup() -> Node<Ipv6Prefix> {
                let n = super::setup();
                let q = IpPrefix::from_str("2001:db8:baa::/48")
                    .unwrap()
                    .try_into()
                    .unwrap();
                let m = Node::new_singleton(q);
                n.add(m).unwrap()
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
                assert_eq!(n.gluemap, 0);
                Ok(())
            }

            #[test]
            fn has_subtree_size_two() -> TestResult {
                let n = setup();
                assert_eq!(subtree_size(n), 3);
                Ok(())
            }
        }
    }
}
