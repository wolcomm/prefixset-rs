//! [`IpPrefixRange<P>`] and related types.
use std::cmp::Ordering;
use std::fmt;
use std::ops::RangeInclusive;
use std::str::FromStr;

use ipnet::PrefixLenError;

use crate::error::{Error, Result};

use super::{IntoSubprefixes, IpPrefix, Subprefixes};

/// An object representing a contigious range of [`IpPrefix`]s, covered by a
/// common super-prefix.
///
/// ``` rust
/// # use prefixset::{Error, Ipv4Prefix, IpPrefixRange};
/// # fn main() -> Result<(), Error> {
/// let range = IpPrefixRange::new(
///     "192.0.2.0/24".parse::<Ipv4Prefix>()?,  // covering super-prefix
///     26,                                     // prefix-length lower bound (inclusive)
///     28,                                     // prefix-length upper bound (inclusive)
/// )?;
/// assert_eq!(range.iter().count(), 28);
/// # Ok(())
/// # }
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct IpPrefixRange<P: IpPrefix> {
    base: P,
    lower: u8,
    upper: u8,
}

impl<P: IpPrefix> IpPrefixRange<P> {
    /// Construct a new [`IpPrefixRange<P>`] from a base [`IpPrefix`]
    /// along with lower and upper prefix-length bounds.
    pub fn new(base: P, lower: u8, upper: u8) -> Result<Self> {
        if base.length() > lower || lower > upper || upper > P::MAX_LENGTH {
            println!("base: {:?}, lower: {}, upper: {}", base, lower, upper);
            return Err(Error::PrefixLen(PrefixLenError));
        }
        Ok(Self { base, lower, upper })
    }

    /// Get the covering super-prefix of `self`.
    pub fn base(&self) -> &P {
        &self.base
    }

    /// Get the range of prefix-lengths included in `self`.
    pub fn range(&self) -> RangeInclusive<u8> {
        self.lower..=self.upper
    }

    /// Get an iterator over the `IpPrefix`s included in `self`.
    ///
    /// ``` rust
    /// # use prefixset::{Error, Ipv4Prefix, IpPrefixRange};
    /// # fn main() -> Result<(), Error> {
    /// let range = IpPrefixRange::new("192.0.2.0/24".parse::<Ipv4Prefix>()?, 25, 25)?;
    /// let mut prefixes = range.iter();
    /// assert_eq!(prefixes.next(), Some("192.0.2.0/25".parse()?));
    /// assert_eq!(prefixes.next(), Some("192.0.2.128/25".parse()?));
    /// assert_eq!(prefixes.next(), None);
    /// # Ok(())
    /// # }
    pub fn iter(&self) -> Iter<P> {
        self.into_iter()
    }
}

impl<P: IpPrefix> From<P> for IpPrefixRange<P> {
    fn from(prefix: P) -> Self {
        Self::new(prefix, prefix.length(), prefix.length()).unwrap()
    }
}

impl<P: IpPrefix> FromStr for IpPrefixRange<P> {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let (prefix, lower, upper) = match s.split_once(',') {
            Some((prefix_str, range_str)) => {
                let prefix = prefix_str.parse::<P>()?;
                let (lower, upper) = match range_str.split_once(',') {
                    Some((l, u)) => (l.parse()?, u.parse()?),
                    None => return Err(Error::RangeParse { source: None }),
                };
                (prefix, lower, upper)
            }
            None => return Err(Error::RangeParse { source: None }),
        };
        Self::new(prefix, lower, upper)
    }
}

impl<P: IpPrefix> fmt::Display for IpPrefixRange<P> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!("{},{},{}", self.base, self.lower, self.upper))
    }
}

impl<P: IpPrefix> PartialOrd for IpPrefixRange<P> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.base.partial_cmp(&other.base) {
            Some(Ordering::Equal) if self.range() == other.range() => Some(Ordering::Equal),
            Some(Ordering::Less | Ordering::Equal)
                if other.lower <= self.lower && self.upper <= other.upper =>
            {
                Some(Ordering::Less)
            }
            Some(Ordering::Greater | Ordering::Equal)
                if self.lower <= other.lower && other.upper <= self.upper =>
            {
                Some(Ordering::Greater)
            }
            _ => None,
        }
    }
}

impl<P: IpPrefix> IntoIterator for IpPrefixRange<P> {
    type Item = P;
    type IntoIter = IntoIter<P>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            base: self.base,
            lower: self.lower,
            upper: self.upper,
            current: None,
        }
    }
}

/// Consuming iterator returned by [`IpPrefixRange<P>::into_iter()`].
#[derive(Debug)]
pub struct IntoIter<P: IpPrefix> {
    base: P,
    lower: u8,
    upper: u8,
    current: Option<IntoSubprefixes<P>>,
}

impl<P: IpPrefix> Iterator for IntoIter<P> {
    type Item = P;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.current {
            Some(current) => match current.next() {
                Some(item) => Some(item),
                None => {
                    let next_length = current.length + 1;
                    if next_length <= self.upper {
                        self.current = Some(self.base.into_subprefixes(next_length));
                        self.current.as_mut()?.next()
                    } else {
                        None
                    }
                }
            },
            None => {
                self.current = Some(self.base.into_subprefixes(self.lower));
                self.current.as_mut()?.next()
            }
        }
    }
}

impl<'a, P: IpPrefix> IntoIterator for &'a IpPrefixRange<P> {
    type Item = P;
    type IntoIter = Iter<'a, P>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            base: &self.base,
            lower: self.lower,
            upper: self.upper,
            current: None,
        }
    }
}

/// Non-consuming iterator returned by [`IpPrefixRange<P>::iter()`].
#[derive(Debug)]
pub struct Iter<'a, P: IpPrefix> {
    base: &'a P,
    lower: u8,
    upper: u8,
    current: Option<Subprefixes<'a, P>>,
}

impl<'a, P: IpPrefix> Iterator for Iter<'a, P> {
    type Item = P;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.current {
            Some(current) => match current.next() {
                Some(item) => Some(item),
                None => {
                    let next_length = current.length + 1;
                    if next_length <= self.upper {
                        self.current = Some(self.base.subprefixes(next_length));
                        self.current.as_mut()?.next()
                    } else {
                        None
                    }
                }
            },
            None => {
                self.current = Some(self.base.subprefixes(self.lower));
                self.current.as_mut()?.next()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Error, IpPrefixRange};
    use crate::tests::TestResult;
    use crate::Ipv4Prefix;

    #[test]
    fn invalid_lower() -> TestResult {
        let p: Ipv4Prefix = "192.0.2.0/24".parse()?;
        match IpPrefixRange::new(p, 23, 24) {
            Err(Error::PrefixLen(_)) => Ok(()),
            _ => Err("Expected 'PrefixLenError'".into()),
        }
    }

    #[test]
    fn invalid_upper() -> TestResult {
        let p: Ipv4Prefix = "192.0.2.0/24".parse()?;
        match IpPrefixRange::new(p, 23, 24) {
            Err(Error::PrefixLen(_)) => Ok(()),
            _ => Err("Expected 'PrefixLenError'".into()),
        }
    }

    #[test]
    fn invalid_range() -> TestResult {
        let p: Ipv4Prefix = "192.0.2.0/24".parse()?;
        match IpPrefixRange::new(p, 25, 24) {
            Err(Error::PrefixLen(_)) => Ok(()),
            _ => Err("Expected 'PrefixLenError'".into()),
        }
    }

    #[test]
    fn singleton_range() -> TestResult {
        let p: Ipv4Prefix = "192.0.2.0/24".parse()?;
        let r = IpPrefixRange::new(p, 24, 24)?;
        let mut iter = r.into_iter();
        assert_eq!(iter.next().unwrap(), p);
        assert!(iter.next().is_none());
        Ok(())
    }

    #[test]
    fn host_length_range() -> TestResult {
        let p: Ipv4Prefix = "192.0.2.0/24".parse()?;
        let r = IpPrefixRange::new(p, 32, 32)?;
        let mut iter = r.into_iter();
        assert_eq!(iter.next().unwrap(), "192.0.2.0/32".parse()?);
        assert_eq!(iter.next().unwrap(), "192.0.2.1/32".parse()?);
        Ok(())
    }

    #[test]
    fn single_length_range() -> TestResult {
        let p: Ipv4Prefix = "192.0.2.0/24".parse()?;
        let r = IpPrefixRange::new(p, 25, 25)?;
        let mut iter = r.into_iter();
        assert_eq!(iter.next().unwrap(), "192.0.2.0/25".parse()?);
        assert_eq!(iter.next().unwrap(), "192.0.2.128/25".parse()?);
        assert!(iter.next().is_none());
        Ok(())
    }

    #[test]
    fn multi_length_range() -> TestResult {
        let p: Ipv4Prefix = "192.0.2.0/24".parse()?;
        let (lower, upper) = (26, 28);
        let r = IpPrefixRange::new(p, lower, upper)?;
        assert_eq!(
            r.into_iter().count(),
            (lower..=upper).map(|l| { 1 << (l - 24) }).sum()
        );
        Ok(())
    }

    #[test]
    fn subprefix_order() -> TestResult {
        let r: IpPrefixRange<Ipv4Prefix> = "192.0.2.0/24,25,25".parse()?;
        let s: IpPrefixRange<Ipv4Prefix> = "192.0.2.128/25,25,25".parse()?;
        assert!(dbg!(r) > dbg!(s));
        Ok(())
    }

    #[test]
    fn range_order() -> TestResult {
        let r: IpPrefixRange<Ipv4Prefix> = "192.0.2.0/24,24,26".parse()?;
        let s: IpPrefixRange<Ipv4Prefix> = "192.0.2.0/24,25,25".parse()?;
        assert!(dbg!(r) > dbg!(s));
        Ok(())
    }
}
