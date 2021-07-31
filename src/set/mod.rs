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

    fn insert_node(&mut self, new: Box<Node<P>>) -> &mut Self {
        match mem::take(&mut self.root) {
            Some(root) => {
                self.root = Some(root.add(new));
            }
            None => {
                self.root = Some(new);
            }
        };
        self
    }

    pub fn insert<T>(&mut self, item: T) -> &mut Self
    where
        T: Into<Box<Node<P>>>,
    {
        self.insert_node(item.into()).aggregate()
    }

    pub fn insert_from<I, T>(&mut self, iter: I) -> &mut Self
    where
        I: IntoIterator<Item = T>,
        T: Into<Box<Node<P>>>,
    {
        iter.into_iter()
            .fold(self, |set, item| set.insert_node(item.into()))
            .aggregate()
    }

    fn remove_node(&mut self, mut old: Box<Node<P>>) -> &mut Self {
        if let Some(root) = mem::take(&mut self.root) {
            self.root = Some(root.remove(&mut old));
        };
        self
    }

    pub fn remove<T>(&mut self, item: T) -> &mut Self
    where
        T: Into<Box<Node<P>>>,
    {
        self.remove_node(item.into()).aggregate()
    }

    pub fn remove_from<I, T>(&mut self, iter: I) -> &mut Self
    where
        I: IntoIterator<Item = T>,
        T: Into<Box<Node<P>>>,
    {
        iter.into_iter()
            .fold(self, |set, item| set.remove_node(item.into()))
            .aggregate()
    }

    fn aggregate(&mut self) -> &mut Self {
        if let Some(root) = mem::take(&mut self.root) {
            self.root = root.deduplicate(None).aggregate().compress();
        }
        self
    }

    pub fn contains(&self, prefix: P) -> bool {
        match &self.root {
            Some(root) => root.search(&prefix.into()).is_some(),
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
