use std::iter::IntoIterator;
use std::mem;

use crate::node::Node;
use crate::prefix::IpPrefix;

mod from;
mod iter;
mod ops;

use self::iter::{Prefixes, Ranges};

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
            self.root = root.aggregate(None)
        }
        self
    }

    pub fn contains(&self, prefix: &P) -> bool {
        match &self.root {
            Some(root) => root.search(&prefix.into()).is_some(),
            None => false,
        }
    }

    pub fn len(&self) -> usize {
        self.prefixes().count()
    }

    pub fn is_empty(&self) -> bool {
        self.ranges().count() == 0
    }

    pub fn clear(&mut self) {
        self.root = None
    }

    pub fn ranges(&self) -> Ranges<P> {
        self.into()
    }

    pub fn prefixes(&self) -> Prefixes<P> {
        self.into()
    }
}

impl<P: IpPrefix> Default for PrefixSet<P> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, P: IpPrefix> IntoIterator for &'a PrefixSet<P> {
    type Item = P;
    type IntoIter = Prefixes<'a, P>;

    fn into_iter(self) -> Self::IntoIter {
        self.prefixes()
    }
}

impl<P, A> Extend<A> for PrefixSet<P>
where
    P: IpPrefix,
    A: Into<Box<Node<P>>>,
{
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = A>,
    {
        self.insert_from(iter);
    }
}

#[cfg(test)]
mod tests;
