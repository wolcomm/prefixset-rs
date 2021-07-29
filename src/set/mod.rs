use std::iter::IntoIterator;
use std::mem;

use crate::node::Node;
use crate::prefix::{IpPrefix, IpPrefixRange};

mod from;
mod iter;
mod ops;

use self::iter::{PrefixIter, PrefixRangeIter};

#[derive(Clone, Debug)]
pub struct PrefixSet<P: IpPrefix> {
    root: Option<Box<Node<P>>>,
}

impl<P: IpPrefix> PrefixSet<P> {
    pub fn new() -> Self {
        PrefixSet { root: None }
    }

    fn new_with_root(root: Option<Box<Node<P>>>) -> Self {
        PrefixSet { root }
    }

    fn aggregate(&mut self) -> &mut Self {
        if let Some(root) = mem::take(&mut self.root) {
            self.root = root.deduplicate(None).aggregate().compress();
        }
        self
    }

    fn add_node(&mut self, new: Box<Node<P>>) -> &mut Self {
        match mem::take(&mut self.root) {
            Some(root) => {
                let new_root = root.add(new);
                self.root = Some(new_root);
            }
            None => {
                self.root = Some(new);
            }
        };
        self
    }

    fn add_singleton(&mut self, prefix: P) -> &mut Self {
        let new = Box::new(Node::new_singleton(prefix));
        self.add_node(new)
    }

    fn add_range(&mut self, prefix_range: IpPrefixRange<P>) -> &mut Self {
        let new = Box::new(Node::new_range(prefix_range));
        self.add_node(new)
    }

    pub fn add_prefix(&mut self, prefix: P) -> &mut Self {
        self.add_singleton(prefix).aggregate()
    }

    pub fn add_prefixes_from<I>(&mut self, iter: I) -> &mut Self
    where
        I: IntoIterator<Item = P>,
    {
        iter.into_iter()
            .fold(self, |set, p| set.add_singleton(p))
            .aggregate()
    }

    pub fn add_prefix_range(&mut self, prefix_range: IpPrefixRange<P>) -> &mut Self {
        self.add_range(prefix_range).aggregate()
    }

    pub fn add_prefix_ranges_from<I>(&mut self, iter: I) -> &mut Self
    where
        I: IntoIterator<Item = IpPrefixRange<P>>,
    {
        iter.into_iter()
            .fold(self, |set, r| set.add_range(r))
            .aggregate()
    }

    fn remove_node(&mut self, mut old: Box<Node<P>>) -> &mut Self {
        if let Some(root) = mem::take(&mut self.root) {
            self.root = Some(root.remove(&mut old));
        };
        self
    }

    fn remove_singleton(&mut self, prefix: P) -> &mut Self {
        let old = Box::new(Node::new_singleton(prefix));
        self.remove_node(old)
    }

    fn remove_range(&mut self, prefix_range: IpPrefixRange<P>) -> &mut Self {
        let old = Box::new(Node::new_range(prefix_range));
        self.remove_node(old)
    }

    pub fn remove_prefix(&mut self, prefix: P) -> &mut Self {
        self.remove_singleton(prefix).aggregate()
    }

    pub fn remove_prefixes_from<I>(&mut self, iter: I) -> &mut Self
    where
        I: IntoIterator<Item = P>,
    {
        iter.into_iter()
            .fold(self, |set, p| set.remove_singleton(p))
            .aggregate()
    }

    pub fn remove_prefix_range(&mut self, prefix_range: IpPrefixRange<P>) -> &mut Self {
        self.remove_range(prefix_range).aggregate()
    }

    pub fn remove_prefix_ranges_from<I>(&mut self, iter: I) -> &mut Self
    where
        I: IntoIterator<Item = IpPrefixRange<P>>,
    {
        iter.into_iter()
            .fold(self, |set, r| set.remove_range(r))
            .aggregate()
    }

    pub fn contains(&self, prefix: P) -> bool {
        let qnode = Node::new_singleton(prefix);
        match &self.root {
            Some(root) => root.search(&qnode).is_some(),
            None => false,
        }
    }

    pub fn iter_prefix_ranges(&self) -> PrefixRangeIter<P> {
        self.into()
    }

    pub fn iter_prefixes(&self) -> PrefixIter<P> {
        self.into()
    }
}

impl<P: IpPrefix> Default for PrefixSet<P> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, P: IpPrefix> IntoIterator for &'a PrefixSet<P> {
    type Item = IpPrefixRange<P>;
    type IntoIter = PrefixRangeIter<'a, P>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_prefix_ranges()
    }
}

#[cfg(test)]
mod tests;
