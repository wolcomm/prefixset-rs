use std::error::Error;

use crate::node::Node;
use crate::prefix::IpPrefixAggregate;

#[derive(Clone, Debug)]
pub struct PrefixSet<P: IpPrefixAggregate> {
    root: Option<Box<Node<P>>>,
}

impl<P: IpPrefixAggregate> PrefixSet<P> {
    pub fn new() -> Self {
        PrefixSet{
            root: None
        }
    }

    pub fn add(mut self, prefix: P) -> Result<Self, Box<dyn Error>> {
        let new = Node::new_singleton(prefix);
        match self.root {
            Some(root) => {
                let new_root = root.add(new)?;
                self.root = Some(Box::new(new_root));
            }
            None => {
                self.root = Some(Box::new(new));
            }
        };
        Ok(self)
    }

    pub fn contains(&self, prefix: P) -> Result<bool, Box<dyn Error>> {
        let qnode = Node::new_singleton(prefix);
        match &self.root {
            Some(root) => match root.search(&qnode)? {
                Some(node) => {
                    Ok(!node.is_glue())
                },
                None => Ok(false),
            },
            None => Ok(false),
        }
    }
}

impl<P: IpPrefixAggregate> IntoIterator for PrefixSet<P> {
    type Item = P;
    type IntoIter = std::vec::IntoIter<P>;
    fn into_iter(self) -> Self::IntoIter {
        let mut items: Vec<P> = Vec::new();
        if let Some(root) = &self.root {
            root.walk(&mut |node: &Node<P>| {
                if ! node.is_glue() {
                    items.push(node.prefix().to_owned())
                }
            });
        }
        items.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryInto;
    use std::str::FromStr;

    use crate::{IpPrefix, Ipv4Prefix};
    use crate::tests::{assert_none, TestResult};

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
            assert!(!s.contains("192.0.2.0/24".parse()?)?);
            assert!(!s.contains("192.0.0.0/22".parse()?)?);
            assert!(!s.contains("192.0.2.128/25".parse()?)?);
            assert!(!s.contains("192.0.4.0/24".parse()?)?);
            Ok(())
        }

        mod with_a_prefix_added {
            use super::*;

            fn setup() -> PrefixSet<Ipv4Prefix> {
                let s = super::setup();
                let p = IpPrefix::from_str("192.0.2.0/24")
                    .unwrap()
                    .try_into()
                    .unwrap();
                s.add(p).unwrap()
            }

            #[test]
            fn contains_that_prefix() -> TestResult {
                let s = setup();
                assert!(s.contains("192.0.2.0/24".parse()?)?);
                Ok(())
            }

            #[test]
            fn does_not_contain_others() -> TestResult {
                let s = setup();
                assert!(!s.contains("192.0.0.0/22".parse()?)?);
                assert!(!s.contains("192.0.2.128/25".parse()?)?);
                assert!(!s.contains("192.0.4.0/24".parse()?)?);
                Ok(())
            }

            mod with_another_prefix_added {
                use super::*;

                fn setup() -> PrefixSet<Ipv4Prefix> {
                    let s = super::setup();
                    let p = IpPrefix::from_str("192.0.0.0/22")
                        .unwrap()
                        .try_into()
                        .unwrap();
                    s.add(p).unwrap()
                }

                #[test]
                fn contains_both_prefixes() -> TestResult {
                    let s = setup();
                    assert!(s.contains("192.0.2.0/24".parse()?)?);
                    assert!(s.contains("192.0.0.0/22".parse()?)?);
                    Ok(())
                }
            }
        }
    }
}
