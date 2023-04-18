use ip::{Afi, PrefixRange};

use crate::node;

use super::PrefixSet;

/// Non-consuming iterator returned by [`PrefixSet<A>::ranges()`].
#[derive(Debug)]
pub struct Ranges<'a, A: Afi> {
    tree_iter: Option<node::Children<'a, A>>,
    ranges_iter: Option<node::Ranges<'a, A>>,
}

impl<'a, A: Afi> From<&'a PrefixSet<A>> for Ranges<'a, A> {
    fn from(s: &'a PrefixSet<A>) -> Self {
        Self {
            tree_iter: s.root.as_ref().map(|root| root.children()),
            ranges_iter: None,
        }
    }
}

impl<'a, A: Afi> Iterator for Ranges<'a, A> {
    type Item = <node::Ranges<'a, A> as Iterator>::Item;

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

/// Non-consuming iterator returned by [`PrefixSet<A>::prefixes()`].
#[derive(Debug)]
pub struct Prefixes<'a, A: Afi> {
    ranges_iter: Ranges<'a, A>,
    prefix_range_iter: Option<<PrefixRange<A> as IntoIterator>::IntoIter>,
}

impl<'a, A: Afi> From<&'a PrefixSet<A>> for Prefixes<'a, A> {
    fn from(s: &'a PrefixSet<A>) -> Self {
        Self {
            ranges_iter: s.into(),
            prefix_range_iter: None,
        }
    }
}

impl<'a, A: Afi> Iterator for Prefixes<'a, A> {
    type Item = <PrefixRange<A> as IntoIterator>::Item;

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
