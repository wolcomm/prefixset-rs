use std::convert::TryInto;

use num::Zero;

use crate::prefix::{IpPrefix, IpPrefixRange};

use super::{Node, GlueMap};

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
            last: 0
        }
    }
}

impl<'a, P: IpPrefix> Iterator for NodeRangesIter<'a, P> {
    type Item = IpPrefixRange<P>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.map != GlueMap::zero() {
            // unwrap is safe here as long as our address family
            // is 255 bits long or less
            // println!("gluemap: {:#?}", self.map);
            let right_zeros: u8 = self.map.trailing_zeros().try_into().unwrap();
            // println!("right zeros: {}", right_zeros);
            let lower = self.last + right_zeros;
            self.map >>= right_zeros;
            // println!("gluemap: {:#?}", self.map);
            // unwrap is safe here as long as our address family
            // is 255 bits long or less
            let right_ones: u8 = (!self.map).trailing_zeros().try_into().unwrap();
            // println!("right ones: {}", right_ones);
            let upper = lower + right_ones - 1;
            self.map >>= right_ones;
            // println!("gluemap: {:#?}", self.map);
            self.last = upper + 1;
            // println!("lower: {}, upper: {}", lower, upper);
            // unwrap is safe here as long as self.map doesn't have any
            // bits set lower than self.this.prefix.length()
            Some(IpPrefixRange::new(self.this.prefix.to_owned(), lower, upper).unwrap())
        } else {
            None
        }
    }
}
