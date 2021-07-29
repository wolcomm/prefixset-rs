use crate::node::{NodeRangesIter, NodeTreeIter};
use crate::prefix::{IpPrefix, IpPrefixRangeIntoIter};

use super::PrefixSet;

#[derive(Debug)]
pub struct PrefixRangeIter<'a, P: IpPrefix> {
    tree_iter: Option<NodeTreeIter<'a, P>>,
    ranges_iter: Option<NodeRangesIter<'a, P>>,
}

impl<'a, P: IpPrefix> From<&'a PrefixSet<P>> for PrefixRangeIter<'a, P> {
    fn from(s: &'a PrefixSet<P>) -> Self {
        Self {
            tree_iter: match s.root {
                Some(ref root) => Some(root.iter_subtree()),
                None => None,
            },
            ranges_iter: None,
        }
    }
}

impl<'a, P: IpPrefix> Iterator for PrefixRangeIter<'a, P> {
    type Item = <NodeRangesIter<'a, P> as Iterator>::Item;

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
                    Some(node) => self.ranges_iter = Some(node.iter_ranges()),
                    None => return None,
                }
            } else {
                return None;
            }
        }
    }
}

#[derive(Debug)]
pub struct PrefixIter<'a, P: IpPrefix> {
    ranges_iter: PrefixRangeIter<'a, P>,
    prefix_range_iter: Option<IpPrefixRangeIntoIter<P>>,
}

impl<'a, P: IpPrefix> From<&'a PrefixSet<P>> for PrefixIter<'a, P> {
    fn from(s: &'a PrefixSet<P>) -> Self {
        Self {
            ranges_iter: s.into(),
            prefix_range_iter: None,
        }
    }
}

impl<'a, P: IpPrefix> Iterator for PrefixIter<'a, P> {
    type Item = <IpPrefixRangeIntoIter<P> as Iterator>::Item;

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
