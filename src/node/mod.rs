use std::cmp::min;
use std::convert::TryInto;

use num::{One, PrimInt, Zero};

use crate::prefix::IpPrefix;

use self::gluemap::GlueMap;
pub use self::iter::{NodeRangesIter, NodeTreeIter};

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

#[derive(Clone, Debug)]
pub struct Node<P: IpPrefix> {
    prefix: P,
    gluemap: GlueMap<P>,
    left: Option<Box<Node<P>>>,
    right: Option<Box<Node<P>>>,
}

impl<P: IpPrefix> Node<P> {
    fn new(prefix: P, gluemap: GlueMap<P>) -> Self {
        Node {
            prefix,
            gluemap,
            left: None,
            right: None,
        }
    }

    pub fn prefix(&self) -> &P {
        &self.prefix
    }

    pub fn add(mut self: Box<Self>, mut other: Box<Self>) -> Box<Self> {
        match self.compare_with(&other) {
            Comparison::Equal => {
                self.gluemap |= other.gluemap;
                if let Some(child) = other.left {
                    self = self.add(child);
                }
                if let Some(child) = other.right {
                    self = self.add(child);
                }
                self
            }
            Comparison::ChildOf(common) => {
                // mask glue map for prefix lengths already present
                other.gluemap &= !self.gluemap;
                match other.branch_direction(common) {
                    Direction::Left => {
                        if let Some(child) = self.left {
                            let new_child = child.add(other);
                            self.left = Some(new_child);
                        } else {
                            self.left = Some(other);
                        }
                    }
                    Direction::Right => {
                        if let Some(child) = self.right {
                            let new_child = child.add(other);
                            self.right = Some(new_child);
                        } else {
                            self.right = Some(other);
                        }
                    }
                };
                self
            }
            Comparison::ParentOf(common) => {
                self.gluemap &= !other.gluemap;
                match self.branch_direction(common) {
                    Direction::Left => {
                        if let Some(child) = other.left {
                            let new_child = child.add(self);
                            other.left = Some(new_child);
                        } else {
                            other.left = Some(self);
                        }
                    }
                    Direction::Right => {
                        if let Some(child) = other.right {
                            let new_child = child.add(self);
                            other.right = Some(new_child);
                        } else {
                            other.right = Some(self);
                        }
                    }
                };
                other
            }
            Comparison::Divergent(common) => {
                // unwrap is safe here because common < P::MAX_LENGTH
                let glue_prefix = self.prefix.new_from(common).unwrap();
                let mut glue = Box::new(Self::new(glue_prefix, GlueMap::zero()));
                match self.branch_direction(common) {
                    Direction::Left => {
                        glue.left = Some(self);
                        glue.right = Some(other);
                    }
                    Direction::Right => {
                        glue.left = Some(other);
                        glue.right = Some(self);
                    }
                };
                glue
            }
        }
    }

    pub fn remove(mut self: Box<Self>, other: &mut Self) -> Box<Self> {
        if let Some(mut child) = other.left.take() {
            self = self.remove(&mut child);
        }
        if let Some(mut child) = other.right.take() {
            self = self.remove(&mut child);
        }
        match self.compare_with(other) {
            Comparison::ParentOf(_) | Comparison::Equal => {
                // clear gluemap bits and recurse down
                self.gluemap &= !other.gluemap;
                if let Some(child) = self.left.take() {
                    self.left = Some(child.remove(other));
                };
                if let Some(child) = self.right.take() {
                    self.right = Some(child.remove(other));
                };
            }
            Comparison::ChildOf(common) => {
                let deaggr_mask = self.gluemap & other.gluemap;
                if deaggr_mask != GlueMap::zero() {
                    // deaggregate matching subprefixes before recursing
                    self.gluemap &= !deaggr_mask;
                    self = self
                        .prefix
                        .into_iter_subprefixes(other.prefix.length())
                        .map(|p| Box::new(Self::new(p, deaggr_mask)))
                        .fold(self, |this, n| this.add(n));
                }
                match other.branch_direction(common) {
                    Direction::Left => {
                        if let Some(child) = self.left.take() {
                            self.left = Some(child.remove(other));
                        };
                    }
                    Direction::Right => {
                        if let Some(child) = self.right.take() {
                            self.right = Some(child.remove(other));
                        };
                    }
                }
            }
            _ => (),
        };
        self
    }

    pub fn deduplicate(mut self: Box<Self>, mut mask: Option<GlueMap<P>>) -> Box<Self> {
        if mask.is_none() {
            mask = Some(GlueMap::zero());
        }
        self.gluemap &= !mask.unwrap();
        *mask.as_mut().unwrap() |= self.gluemap;
        if let Some(child) = self.left.take() {
            self.left = Some(child.deduplicate(mask));
        }
        if let Some(child) = self.right.take() {
            self.right = Some(child.deduplicate(mask));
        }
        self
    }

    pub fn aggregate(mut self: Box<Self>) -> Box<Self> {
        if let Some(child) = self.left.take() {
            self.left = Some(child.aggregate());
        }
        if let Some(child) = self.right.take() {
            self.right = Some(child.aggregate());
        };
        let aggr_length = self.prefix().length() + 1;
        if let (Some(l), Some(r)) = (&mut self.left, &mut self.right) {
            if l.prefix().length() == aggr_length && r.prefix().length() == aggr_length {
                let aggr_bits = l.gluemap & r.gluemap;
                l.gluemap &= !aggr_bits;
                r.gluemap &= !aggr_bits;
                self.gluemap |= aggr_bits;
            }
        }
        self
    }

    pub fn compress(mut self: Box<Self>) -> Option<Box<Self>> {
        if let Some(child) = self.left.take() {
            self.left = child.compress();
        }
        if let Some(child) = self.right.take() {
            self.right = child.compress();
        }
        if self.gluemap == GlueMap::zero() {
            match (&self.left, &self.right) {
                (None, None) => None,
                (Some(_), None) => Some(self.left.unwrap()),
                (None, Some(_)) => Some(self.right.unwrap()),
                _ => Some(self),
            }
        } else {
            Some(self)
        }
    }

    pub fn search(&self, qnode: &Self) -> Option<&Self> {
        match self.compare_with(qnode) {
            Comparison::Equal | Comparison::ChildOf(_)
                if self.gluemap & qnode.gluemap == qnode.gluemap =>
            {
                Some(self)
            }
            Comparison::ChildOf(common) => match qnode.branch_direction(common) {
                Direction::Left => {
                    if let Some(child) = &self.left {
                        child.search(qnode)
                    } else {
                        None
                    }
                }
                Direction::Right => {
                    if let Some(child) = &self.right {
                        child.search(qnode)
                    } else {
                        None
                    }
                }
            },
            _ => None,
        }
    }

    fn intersect_nodes(&self, qnode: &Self) -> Option<Box<Self>> {
        match self.compare_with(qnode) {
            Comparison::Divergent(_) => None,
            cmp => {
                let prefix = if let Comparison::ChildOf(_) = cmp {
                    qnode.prefix().to_owned()
                } else {
                    self.prefix().to_owned()
                };
                let mut new = Box::new(Node::new(prefix, self.gluemap & qnode.gluemap));
                if let Some(child) = &self.left {
                    if let Some(intersect_child) = child.intersect_nodes(qnode) {
                        new = new.add(intersect_child);
                    };
                };
                if let Some(child) = &self.right {
                    if let Some(intersect_child) = child.intersect_nodes(qnode) {
                        new = new.add(intersect_child);
                    };
                };
                Some(new)
            }
        }
    }

    fn branch_direction(&self, bit_index: u8) -> Direction {
        let next_index = bit_index + 1;
        let mask = P::BitMap::one() << (P::MAX_LENGTH - next_index);
        if self.prefix.bits() & mask == P::BitMap::zero() {
            Direction::Left
        } else {
            Direction::Right
        }
    }

    fn compare_with(&self, other: &Self) -> Comparison {
        let min_lens = min(self.prefix.length(), other.prefix.length());
        let diff_map = self.prefix.bits() ^ other.prefix.bits();
        let common = min(min_lens, diff_map.leading_zeros().try_into().unwrap());
        if common == self.prefix.length() && common == other.prefix.length() {
            Comparison::Equal
        } else if common == self.prefix.length() && common < other.prefix.length() {
            Comparison::ChildOf(common)
        } else if common < self.prefix.length() && common == other.prefix.length() {
            Comparison::ParentOf(common)
        } else if common < self.prefix.length() && common < other.prefix.length() {
            Comparison::Divergent(common)
        } else {
            unreachable!("Common cannot be larger than either prefix length")
        }
    }

    pub fn iter_ranges(&self) -> NodeRangesIter<P> {
        self.into()
    }

    #[allow(clippy::needless_lifetimes, clippy::borrowed_box)]
    pub fn iter_subtree<'a>(self: &'a Box<Self>) -> NodeTreeIter<'a, P> {
        self.into()
    }
}

mod from;
mod gluemap;
mod iter;
mod ops;

#[cfg(test)]
mod tests;
