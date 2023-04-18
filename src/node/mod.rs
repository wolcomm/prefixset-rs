use ip::{
    concrete::{Hostmask, Netmask, PrefixOrdering},
    traits::{Address as _, Prefix as _, PrefixLength as _},
    Afi, Prefix,
};

mod from;
mod gluemap;
mod iter;
mod ops;

use self::gluemap::GlueMap;
pub use self::iter::{Children, Ranges};

enum Direction {
    Left,
    Right,
}

#[derive(Clone, Debug)]
pub struct Node<A: Afi> {
    prefix: Prefix<A>,
    gluemap: GlueMap<A>,
    left: Option<Box<Node<A>>>,
    right: Option<Box<Node<A>>>,
}

impl<A: Afi> Node<A> {
    fn new(prefix: Prefix<A>, gluemap: GlueMap<A>) -> Self {
        Node {
            prefix,
            gluemap,
            left: None,
            right: None,
        }
    }

    fn new_glue(prefix: Prefix<A>) -> Self {
        Self::new(prefix, GlueMap::ZERO)
    }

    pub fn boxed(self) -> Box<Self> {
        Box::new(self)
    }

    pub fn prefix(&self) -> &Prefix<A> {
        &self.prefix
    }

    pub fn is_glue(&self) -> bool {
        self.gluemap == GlueMap::ZERO
    }

    pub fn add(mut self: Box<Self>, mut other: Box<Self>) -> Box<Self> {
        match self.prefix().compare(other.prefix()) {
            PrefixOrdering::Equal => {
                self.gluemap |= other.gluemap;
                if let Some(child) = other.left {
                    self = self.add(child);
                }
                if let Some(child) = other.right {
                    self = self.add(child);
                }
                self
            }
            PrefixOrdering::Subprefix(common) => {
                // mask glue map for prefix lengths already present
                other.gluemap &= !self.gluemap;
                match other.branch_direction(&common) {
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
            PrefixOrdering::Superprefix(common) => {
                self.gluemap &= !other.gluemap;
                match self.branch_direction(&common) {
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
            PrefixOrdering::Divergent(common) => {
                // unwrap is safe here because common < P::MAX_LENGTH
                let mut glue = Self::new_glue(common).boxed();
                match self.branch_direction(&common) {
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
        match self.prefix().compare(other.prefix()) {
            PrefixOrdering::Superprefix(_) | PrefixOrdering::Equal => {
                // clear gluemap bits and recurse down
                self.gluemap &= !other.gluemap;
                if let Some(child) = self.left.take() {
                    self.left = Some(child.remove(other));
                };
                if let Some(child) = self.right.take() {
                    self.right = Some(child.remove(other));
                };
            }
            PrefixOrdering::Subprefix(common) => {
                let deaggr_mask = self.gluemap & other.gluemap;
                if deaggr_mask != GlueMap::ZERO {
                    // deaggregate matching subprefixes before recursing
                    self.gluemap &= !deaggr_mask;
                    self = self
                        .prefix
                        .subprefixes(other.prefix.length())
                        .unwrap() // safe because `other` is a subprefix of `self`.
                        .map(|p| Box::new(Self::new(p, deaggr_mask)))
                        .fold(self, |this, n| this.add(n));
                }
                match other.branch_direction(&common) {
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

    pub fn aggregate(mut self: Box<Self>, mut mask: Option<GlueMap<A>>) -> Option<Box<Self>> {
        // set mask to zero if None given
        if mask.is_none() {
            mask = Some(GlueMap::ZERO)
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
        match (
            self.prefix().length().increment(),
            &mut self.right,
            &mut self.left,
        ) {
            (Ok(len), Some(l), Some(r))
                if l.prefix().length() == len && r.prefix().length() == len =>
            {
                // get the bits set in both child gluemaps
                let aggr_bits = l.gluemap & r.gluemap;
                // unset the bits in each child gluemap
                l.gluemap &= !aggr_bits;
                r.gluemap &= !aggr_bits;
                // set them in self.gluemap
                self.gluemap |= aggr_bits;
                // check whether any aggregation occured
                if aggr_bits != GlueMap::ZERO {
                    // left or right may now be unnecessary glue.
                    // also, since some aggregation into self.gluemap occured, self
                    // cannot be a glue node.
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
            _ => self.clean(),
        }
    }

    fn clean(self: Box<Self>) -> Option<Box<Self>> {
        if self.gluemap == GlueMap::ZERO {
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
        match self.prefix().compare(qnode.prefix()) {
            PrefixOrdering::Equal | PrefixOrdering::Subprefix(_)
                if self.gluemap & qnode.gluemap == qnode.gluemap =>
            {
                Some(self)
            }
            PrefixOrdering::Subprefix(common) => match qnode.branch_direction(&common) {
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
        match self.prefix().compare(qnode.prefix()) {
            PrefixOrdering::Divergent(_) => None,
            cmp => {
                let prefix = if let PrefixOrdering::Subprefix(_) = cmp {
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

    fn branch_direction(&self, from: &Prefix<A>) -> Direction {
        let mask = Hostmask::from(from.length())
            & Netmask::from(
                from.length()
                    .increment()
                    // ok to unwrap, because `from.length() < MAX_LENGTH`.
                    .unwrap(),
            );
        if (self.prefix().network() & mask).is_unspecified() {
            Direction::Left
        } else {
            Direction::Right
        }
    }

    pub fn ranges(&self) -> Ranges<A> {
        self.into()
    }

    pub fn children(&self) -> Children<'_, A> {
        self.into()
    }
}

#[cfg(test)]
mod tests;
