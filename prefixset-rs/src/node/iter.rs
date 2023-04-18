use ip::{
    concrete::{PrefixLength, PrefixRange},
    traits::PrefixLength as _,
    Afi,
};

use super::Node;

#[derive(Debug)]
pub struct Children<'a, A: Afi> {
    this: Option<&'a Node<A>>,
    parent: Option<Box<Children<'a, A>>>,
    children: Vec<Option<&'a Node<A>>>,
}

impl<A: Afi> Default for Children<'_, A> {
    fn default() -> Self {
        Self {
            this: None,
            parent: None,
            children: Vec::default(),
        }
    }
}

impl<'a, A: Afi> From<&'a Node<A>> for Children<'a, A> {
    fn from(node: &'a Node<A>) -> Self {
        Self {
            this: Some(node),
            parent: None,
            children: [node.left.as_deref(), node.right.as_deref()].into(),
        }
    }
}

impl<'a, A: Afi> Iterator for Children<'a, A> {
    type Item = &'a Node<A>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(this) = self.this.take() {
            return Some(this);
        }
        while let Some(maybe_child) = self.children.pop() {
            if let Some(child) = maybe_child {
                // construct new Children iterator from `child` replacing self,
                // with current self as parent, and recurse over it
                let mut child_iter = child.children();
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
pub struct Ranges<'a, A: Afi> {
    this: &'a Node<A>,
    next_length: Option<PrefixLength<A>>,
}

impl<'a, A: Afi> From<&'a Node<A>> for Ranges<'a, A> {
    fn from(node: &'a Node<A>) -> Self {
        Self {
            this: node,
            next_length: Some(PrefixLength::MIN),
        }
    }
}

impl<'a, A: Afi> Iterator for Ranges<'a, A> {
    type Item = PrefixRange<A>;

    fn next(&mut self) -> Option<Self::Item> {
        let range = self.this.gluemap.next_range(self.next_length?)?;
        self.next_length = range.end().increment().ok();
        // unwrap is safe here as long as self.map doesn't have any
        // bits set lower than self.this.prefix.length()
        Some(PrefixRange::new(self.this.prefix, range).unwrap())
    }
}
