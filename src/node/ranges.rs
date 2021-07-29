use std::convert::TryInto;

use num::Zero;

use crate::prefix::{IpPrefix, IpPrefixRange};

use super::{GlueMap, Node};

#[derive(Debug)]
pub struct NodeRangesIter<'a, P: IpPrefix> {
    this: &'a Node<P>,
    map: GlueMap<P>,
    last: u8,
}

impl<'a, P: IpPrefix> From<&'a Node<P>> for NodeRangesIter<'a, P> {
    fn from(node: &'a Node<P>) -> Self {
        Self {
            this: node,
            map: node.gluemap.to_owned(),
            last: 0,
        }
    }
}

impl<'a, P: IpPrefix> Iterator for NodeRangesIter<'a, P> {
    type Item = IpPrefixRange<P>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.map != GlueMap::zero() {
            // unwrap is safe here as long as our address family
            // is 255 bits long or less
            let right_zeros: u8 = self.map.trailing_zeros().try_into().unwrap();
            let lower = self.last + right_zeros;
            self.map >>= right_zeros;
            // unwrap is safe here as long as our address family
            // is 255 bits long or less
            let right_ones: u8 = (!self.map).trailing_zeros().try_into().unwrap();
            let upper = lower + right_ones - 1;
            self.map >>= right_ones;
            self.last = upper + 1;
            // unwrap is safe here as long as self.map doesn't have any
            // bits set lower than self.this.prefix.length()
            Some(IpPrefixRange::new(self.this.prefix.to_owned(), lower, upper).unwrap())
        } else {
            None
        }
    }
}
