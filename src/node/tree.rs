use crate::prefix::IpPrefix;

use super::Node;

#[derive(Debug)]
pub struct NodeTreeIter<'a, P: IpPrefix> {
    this: Option<&'a Box<Node<P>>>,
    parent: Option<Box<NodeTreeIter<'a, P>>>,
    children: Vec<Option<&'a Box<Node<P>>>>,
}

impl<P: IpPrefix> Default for NodeTreeIter<'_, P> {
    fn default() -> Self {
        Self {
            this: None,
            parent: None,
            children: Vec::default(),
        }
    }
}

impl<'a, P: IpPrefix> From<&'a Box<Node<P>>> for NodeTreeIter<'a, P> {
    fn from(node: &'a Box<Node<P>>) -> Self {
        Self {
            this: Some(node),
            parent: None,
            children: [node.left.as_ref(), node.right.as_ref()].into(),
        }
    }
}

impl<'a, P: IpPrefix> Iterator for NodeTreeIter<'a, P> {
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
