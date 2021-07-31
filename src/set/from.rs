use std::iter::FromIterator;

use crate::{node::Node, prefix::IpPrefix};

use super::PrefixSet;

impl<P: IpPrefix, S, T> From<T> for PrefixSet<P>
where
    Self: FromIterator<S>,
    T: IntoIterator<Item = S>,
{
    fn from(t: T) -> Self {
        t.into_iter().collect()
    }
}

impl<T, P> FromIterator<T> for PrefixSet<P>
where
    P: IpPrefix,
    T: Into<Box<Node<P>>>,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        PrefixSet::new().insert_from(iter).to_owned()
    }
}
