use std::ops::{BitAnd, BitOr, Sub};

use ip::Afi;

use super::Node;

impl<A: Afi> PartialEq for Node<A> {
    fn eq(&self, other: &Self) -> bool {
        self.prefix == other.prefix && self.gluemap == other.gluemap
    }
}

impl<A: Afi> BitAnd for Box<Node<A>> {
    type Output = Option<Self>;

    fn bitand(self, rhs: Self) -> Self::Output {
        self.children()
            .fold(None, |root, node| match rhs.intersect_nodes(node) {
                Some(new) => {
                    if let Some(root) = root {
                        Some(root.add(new))
                    } else {
                        Some(new)
                    }
                }
                None => root,
            })
    }
}

impl<A: Afi> BitOr for Box<Node<A>> {
    type Output = Option<Self>;

    fn bitor(self, rhs: Self) -> Self::Output {
        Some(self.add(rhs))
    }
}

impl<A: Afi> Sub for Box<Node<A>> {
    type Output = Option<Self>;

    fn sub(self, mut rhs: Self) -> Self::Output {
        Some(self.remove(&mut rhs))
    }
}
