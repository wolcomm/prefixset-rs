use std::str::FromStr;

use ip::{
    concrete::{Prefix, PrefixRange},
    Afi,
};

use crate::error::{Error, Result};

use super::{GlueMap, Node};

impl<A: Afi> From<Prefix<A>> for Node<A> {
    fn from(prefix: Prefix<A>) -> Self {
        Self::new(prefix, GlueMap::singleton(prefix.length()))
    }
}

impl<A: Afi> From<PrefixRange<A>> for Node<A> {
    fn from(range: PrefixRange<A>) -> Self {
        Self::new(range.prefix(), range.into())
    }
}

impl<A: Afi> FromStr for Node<A> {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        s.parse::<Prefix<_>>().map(Node::from).map_err(Error::from)
    }
}
