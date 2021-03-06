use crate::node;
use crate::prefix::{self, IpPrefix};

use super::PrefixSet;

/// Non-consuming iterator returned by [`PrefixSet<P>::ranges()`].
#[derive(Debug)]
pub struct Ranges<'a, P: IpPrefix> {
    tree_iter: Option<node::Children<'a, P>>,
    ranges_iter: Option<node::Ranges<'a, P>>,
}

impl<'a, P: IpPrefix> From<&'a PrefixSet<P>> for Ranges<'a, P> {
    fn from(s: &'a PrefixSet<P>) -> Self {
        Self {
            tree_iter: s.root.as_ref().map(|root| root.children()),
            ranges_iter: None,
        }
    }
}

impl<'a, P: IpPrefix> Iterator for Ranges<'a, P> {
    type Item = <node::Ranges<'a, P> as Iterator>::Item;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(ref mut ranges_iter) = self.ranges_iter {
                match ranges_iter.next() {
                    range @ Some(_) => return range,
                    None => self.ranges_iter = None,
                }
            }
            if let Some(ref mut tree_iter) = self.tree_iter {
                match tree_iter.next() {
                    Some(node) => self.ranges_iter = Some(node.ranges()),
                    None => return None,
                }
            } else {
                return None;
            }
        }
    }
}

/// Non-consuming iterator returned by [`PrefixSet<P>::prefixes()`].
#[derive(Debug)]
pub struct Prefixes<'a, P: IpPrefix> {
    ranges_iter: Ranges<'a, P>,
    prefix_range_iter: Option<prefix::range::IntoIter<P>>,
}

impl<'a, P: IpPrefix> From<&'a PrefixSet<P>> for Prefixes<'a, P> {
    fn from(s: &'a PrefixSet<P>) -> Self {
        Self {
            ranges_iter: s.into(),
            prefix_range_iter: None,
        }
    }
}

impl<'a, P: IpPrefix> Iterator for Prefixes<'a, P> {
    type Item = <prefix::range::IntoIter<P> as Iterator>::Item;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(ref mut prefix_range_iter) = self.prefix_range_iter {
                match prefix_range_iter.next() {
                    p @ Some(_) => return p,
                    None => self.prefix_range_iter = None,
                }
            }
            match self.ranges_iter.next() {
                Some(range) => self.prefix_range_iter = Some(range.into_iter()),
                None => return None,
            }
        }
    }
}
