use std::iter::FromIterator;

use crate::prefix::{IpPrefix, IpPrefixRange};

use super::PrefixSet;

impl<P: IpPrefix, S, T> From<T> for PrefixSet<P>
where
    Self: FromIterator<S>,
    T: IntoIterator<Item=S>,
{
    fn from(t: T) -> Self {
        t.into_iter().collect()
    }
}

impl<P: IpPrefix> FromIterator<P> for PrefixSet<P> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item=P>,
    {
        PrefixSet::new().add_prefixes_from(iter).to_owned()
    }
}

impl<'a, P: IpPrefix> FromIterator<&'a P> for PrefixSet<P> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item=&'a P>,
    {
        iter.into_iter()
            .map(|p| {p.to_owned()})
            .collect()
    }
}

impl<P: IpPrefix> FromIterator<IpPrefixRange<P>> for PrefixSet<P> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item=IpPrefixRange<P>>,
    {
        PrefixSet::new().add_prefix_ranges_from(iter).to_owned()
    }
}

impl<'a, P: IpPrefix> FromIterator<&'a IpPrefixRange<P>> for PrefixSet<P> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item=&'a IpPrefixRange<P>>,
    {
        iter.into_iter()
            .map(|r| {r.to_owned()})
            .collect()
    }
}

impl<P: IpPrefix> FromIterator<&'static str> for PrefixSet<P> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item=&'static str>,
    {
        let (ranges, remaining): (Vec<_>, Vec<_>) = iter.into_iter()
            .map(|s| {
                s.parse::<IpPrefixRange<P>>()
                    .map_err(|_| s)
            })
            .partition(|res| res.is_ok());
        let (prefixes, errors): (Vec<_>, Vec<_>) = remaining.into_iter()
            .map(|s| {
                s.unwrap_err()
                    .parse::<P>()
            })
            .partition(|res| res.is_ok());
        assert!(errors.is_empty());
        Self::new()
            .add_prefix_ranges_from(
                ranges.into_iter().map(|r| r.unwrap())
            )
            .add_prefixes_from(
                prefixes.into_iter().map(|p| p.unwrap())
            )
            .to_owned()
    }
}
