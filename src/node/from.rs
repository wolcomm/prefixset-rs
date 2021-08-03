use std::str::FromStr;

use crate::error::Result;
use crate::prefix::{IpPrefix, IpPrefixRange};

use super::{GlueMap, Node};

impl<P: IpPrefix> From<P> for Node<P> {
    fn from(prefix: P) -> Self {
        Self::new(prefix, GlueMap::singleton(prefix.length()))
    }
}

impl<P: IpPrefix> From<P> for Box<Node<P>> {
    fn from(prefix: P) -> Self {
        Box::new(prefix.into())
    }
}

impl<P: IpPrefix> From<&P> for Node<P> {
    fn from(prefix: &P) -> Self {
        (*prefix).into()
    }
}

impl<P: IpPrefix> From<&P> for Box<Node<P>> {
    fn from(prefix: &P) -> Self {
        Box::new(prefix.into())
    }
}

impl<P: IpPrefix> From<IpPrefixRange<P>> for Node<P> {
    fn from(prefix_range: IpPrefixRange<P>) -> Self {
        Node::new(*prefix_range.base(), (prefix_range).into())
    }
}

impl<P: IpPrefix> From<IpPrefixRange<P>> for Box<Node<P>> {
    fn from(prefix_range: IpPrefixRange<P>) -> Self {
        Box::new(prefix_range.into())
    }
}

impl<P: IpPrefix> From<&IpPrefixRange<P>> for Node<P> {
    fn from(prefix_range: &IpPrefixRange<P>) -> Self {
        (*prefix_range).into()
    }
}

impl<P: IpPrefix> From<&IpPrefixRange<P>> for Box<Node<P>> {
    fn from(prefix_range: &IpPrefixRange<P>) -> Self {
        Box::new(prefix_range.into())
    }
}

impl<P: IpPrefix> FromStr for Node<P> {
    type Err = <P as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self> {
        let prefix = &s.parse::<P>()?;
        Ok(prefix.into())
    }
}

impl<P: IpPrefix> FromStr for Box<Node<P>> {
    type Err = <P as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self> {
        s.parse().map(Box::new)
    }
}
