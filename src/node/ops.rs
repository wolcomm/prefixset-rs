use std::ops::{BitAnd, BitOr, Sub};

use crate::prefix::IpPrefix;

use super::Node;

impl<P: IpPrefix> PartialEq for Node<P> {
    fn eq(&self, other: &Self) -> bool {
        self.prefix == other.prefix && self.gluemap == other.gluemap
    }
}

impl<P: IpPrefix> BitAnd for Box<Node<P>> {
    type Output = Option<Self>;

    fn bitand(self, rhs: Self) -> Self::Output {
        self.iter_subtree()
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

impl<P: IpPrefix> BitOr for Box<Node<P>> {
    type Output = Option<Self>;

    fn bitor(self, rhs: Self) -> Self::Output {
        Some(self.add(rhs))
    }
}

impl<P: IpPrefix> Sub for Box<Node<P>> {
    type Output = Option<Self>;

    fn sub(self, mut rhs: Self) -> Self::Output {
        Some(self.remove(&mut rhs))
    }
}
