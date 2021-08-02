use num::{One, Zero};

use crate::prefix::{Comparison, IpPrefix};

use self::gluemap::GlueMap;
pub use self::iter::{Children, Ranges};

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
        match self.prefix().compare_with(other.prefix()) {
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
            Comparison::Subprefix(common) => {
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
            Comparison::Superprefix(common) => {
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
        match self.prefix().compare_with(other.prefix()) {
            Comparison::Superprefix(_) | Comparison::Equal => {
                // clear gluemap bits and recurse down
                self.gluemap &= !other.gluemap;
                if let Some(child) = self.left.take() {
                    self.left = Some(child.remove(other));
                };
                if let Some(child) = self.right.take() {
                    self.right = Some(child.remove(other));
                };
            }
            Comparison::Subprefix(common) => {
                let deaggr_mask = self.gluemap & other.gluemap;
                if deaggr_mask != GlueMap::zero() {
                    // deaggregate matching subprefixes before recursing
                    self.gluemap &= !deaggr_mask;
                    self = self
                        .prefix
                        .into_subprefixes(other.prefix.length())
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

    pub fn aggregate(mut self: Box<Self>, mut mask: Option<GlueMap<P>>) -> Option<Box<Self>> {
        // set mask to zero if None given
        if mask.is_none() {
            mask = Some(GlueMap::zero())
        }
        // mask is the union of gluemaps of all parent nodes.
        // if the intersection of mask and self.gluemap is not zero
        // then self represents one or more deduplicate prefixes.
        //
        // unset mask bits in self.gluemap
        self.gluemap &= !mask.unwrap();
        // set remaining bits of self.gluemap in mask
        *mask.as_mut().unwrap() |= self.gluemap;
        // recurse child nodes
        if let Some(child) = self.left.take() {
            self.left = child.aggregate(mask);
        }
        if let Some(child) = self.right.take() {
            self.right = child.aggregate(mask);
        }
        // if both left and right child nodes exist, and have the same
        // length == self.prefix.length() + 1, then any bits set in both
        // child gluemaps can be aggregated into self.gluemap.
        //
        let aggr_length = self.prefix().length() + 1;
        let did_aggr = if let (Some(l), Some(r)) = (&mut self.left, &mut self.right) {
            if l.prefix().length() == aggr_length && r.prefix().length() == aggr_length {
                // get the bits set in both child gluemaps
                let aggr_bits = l.gluemap & r.gluemap;
                // unset the bits in each child gluemap
                l.gluemap &= !aggr_bits;
                r.gluemap &= !aggr_bits;
                // set them in self.gluemap
                self.gluemap |= aggr_bits;
                // indicate whether any aggregation occured
                aggr_bits != GlueMap::zero()
            } else {
                false
            }
        } else {
            false
        };
        // if aggregation occured, left or right may now be unnecessary glue.
        // also, since some aggregation into self.gluemap occured, self
        // cannot be a glue node.
        if did_aggr {
            if let Some(child) = self.left.take() {
                self.left = child.clean();
            };
            if let Some(child) = self.right.take() {
                self.right = child.clean();
            };
            Some(self)
        } else {
            self.clean()
        }
    }

    fn clean(self: Box<Self>) -> Option<Box<Self>> {
        if self.gluemap == GlueMap::zero() {
            match (&self.left, &self.right) {
                (None, None) => None,
                (Some(_), None) => self.left,
                (None, Some(_)) => self.right,
                _ => Some(self),
            }
        } else {
            Some(self)
        }
    }

    pub fn search(&self, qnode: &Self) -> Option<&Self> {
        match self.prefix().compare_with(qnode.prefix()) {
            Comparison::Equal | Comparison::Subprefix(_)
                if self.gluemap & qnode.gluemap == qnode.gluemap =>
            {
                Some(self)
            }
            Comparison::Subprefix(common) => match qnode.branch_direction(common) {
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
        match self.prefix().compare_with(qnode.prefix()) {
            Comparison::Divergent(_) => None,
            cmp => {
                let prefix = if let Comparison::Subprefix(_) = cmp {
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
        let mask = P::Bits::one() << (P::MAX_LENGTH - next_index);
        if self.prefix.bits() & mask == P::Bits::zero() {
            Direction::Left
        } else {
            Direction::Right
        }
    }

    pub fn ranges(&self) -> Ranges<P> {
        self.into()
    }

    #[allow(clippy::needless_lifetimes, clippy::borrowed_box)]
    pub fn children<'a>(self: &'a Box<Self>) -> Children<'a, P> {
        self.into()
    }
}

mod from;
mod gluemap;
mod iter;
mod ops;

#[cfg(test)]
mod tests;
