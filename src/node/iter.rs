use std::convert::TryInto;

use num::Zero;

use crate::prefix::{IpPrefix, IpPrefixRange};

use super::{GlueMap, Node};

impl<'a, P: IpPrefix> IntoIterator for &'a Box<Node<P>> {
    type Item = &'a Box<Node<P>>;
    type IntoIter = Children<'a, P>;

    fn into_iter(self) -> Self::IntoIter {
        self.into()
    }
}

#[derive(Debug)]
#[allow(clippy::borrowed_box)]
pub struct Children<'a, P: IpPrefix> {
    this: Option<&'a Box<Node<P>>>,
    parent: Option<Box<Children<'a, P>>>,
    children: Vec<Option<&'a Box<Node<P>>>>,
}

impl<P: IpPrefix> Default for Children<'_, P> {
    fn default() -> Self {
        Self {
            this: None,
            parent: None,
            children: Vec::default(),
        }
    }
}

impl<'a, P: IpPrefix> From<&'a Box<Node<P>>> for Children<'a, P> {
    fn from(node: &'a Box<Node<P>>) -> Self {
        Self {
            this: Some(node),
            parent: None,
            children: [node.left.as_ref(), node.right.as_ref()].into(),
        }
    }
}

impl<'a, P: IpPrefix> Iterator for Children<'a, P> {
    type Item = &'a Box<Node<P>>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(this) = self.this.take() {
            return Some(this);
        }
        while let Some(maybe_child) = self.children.pop() {
            if let Some(child) = maybe_child {
                // construct new NodeTreeIter as self, with current self as
                // parent, and recurse child
                let mut child_iter = child.into_iter();
                child_iter.parent = Some(Box::new(std::mem::take(self)));
                *self = child_iter;
                return self.next();
            }
        }
        // No children left, recurse over parent
        if let Some(parent) = self.parent.take() {
            *self = *parent;
            self.next()
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct Ranges<'a, P: IpPrefix> {
    this: &'a Node<P>,
    map: GlueMap<P>,
    last: u8,
}

impl<'a, P: IpPrefix> From<&'a Node<P>> for Ranges<'a, P> {
    fn from(node: &'a Node<P>) -> Self {
        Self {
            this: node,
            map: node.gluemap.to_owned(),
            last: 0,
        }
    }
}

impl<'a, P: IpPrefix> Iterator for Ranges<'a, P> {
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
