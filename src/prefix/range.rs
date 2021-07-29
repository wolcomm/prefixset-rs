use std::error::Error;
use std::fmt;
use std::ops::RangeInclusive;
use std::str::FromStr;

use ipnet::PrefixLenError;

use super::{IpPrefix, SubPrefixesIntoIter, SubPrefixesIter};

#[derive(Clone, Debug)]
pub struct IpPrefixRange<P: IpPrefix> {
    base: P,
    lower: u8,
    upper: u8,
}

impl<P: IpPrefix> IpPrefixRange<P> {
    pub fn new(base: P, lower: u8, upper: u8) -> Result<Self, Box<dyn Error>> {
        if base.length() > lower || lower > upper || upper > P::MAX_LENGTH {
            println!("base: {:?}, lower: {}, upper: {}", base, lower, upper);
            return Err(Box::new(PrefixLenError));
        }
        Ok(Self { base, lower, upper })
    }

    pub fn base(&self) -> &P {
        &self.base
    }

    pub fn range(&self) -> RangeInclusive<u8> {
        self.lower..=self.upper
    }
}

impl<P: IpPrefix> FromStr for IpPrefixRange<P> {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Box<dyn Error>> {
        let (prefix, lower, upper) = match s.split_once(',') {
            Some((prefix_str, range_str)) => {
                let prefix = prefix_str.parse::<P>()?;
                let (lower, upper) = match range_str.split_once(',') {
                    Some((l, u)) => (l.parse()?, u.parse()?),
                    None => return Err(PrefixRangeParseErr(()).into()),
                };
                (prefix, lower, upper)
            }
            None => return Err(PrefixRangeParseErr(()).into()),
        };
        Self::new(prefix, lower, upper)
    }
}

impl<P: IpPrefix> PartialEq for IpPrefixRange<P> {
    fn eq(&self, other: &Self) -> bool {
        self.base == other.base && self.lower == other.lower && self.upper == other.upper
    }
}

impl<P: IpPrefix> IntoIterator for IpPrefixRange<P> {
    type Item = P;
    type IntoIter = IpPrefixRangeIntoIter<P>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            base: self.base,
            lower: self.lower,
            upper: self.upper,
            current: None,
        }
    }
}

#[derive(Debug)]
pub struct IpPrefixRangeIntoIter<P: IpPrefix> {
    base: P,
    lower: u8,
    upper: u8,
    current: Option<SubPrefixesIntoIter<P>>,
}

impl<P: IpPrefix> Iterator for IpPrefixRangeIntoIter<P> {
    type Item = P;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.current {
            Some(current) => match current.next() {
                Some(item) => Some(item),
                None => {
                    let next_length = current.length + 1;
                    if next_length <= self.upper {
                        self.current = Some(self.base.into_iter_subprefixes(next_length));
                        self.current.as_mut()?.next()
                    } else {
                        None
                    }
                }
            },
            None => {
                self.current = Some(self.base.into_iter_subprefixes(self.lower));
                self.current.as_mut()?.next()
            }
        }
    }
}

impl<'a, P: IpPrefix> IntoIterator for &'a IpPrefixRange<P> {
    type Item = P;
    type IntoIter = IpPrefixRangeIter<'a, P>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            base: &self.base,
            lower: self.lower,
            upper: self.upper,
            current: None,
        }
    }
}

#[derive(Debug)]
pub struct IpPrefixRangeIter<'a, P: IpPrefix> {
    base: &'a P,
    lower: u8,
    upper: u8,
    current: Option<SubPrefixesIter<'a, P>>,
}

impl<'a, P: IpPrefix> Iterator for IpPrefixRangeIter<'a, P> {
    type Item = P;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.current {
            Some(current) => match current.next() {
                Some(item) => Some(item),
                None => {
                    let next_length = current.length + 1;
                    if next_length <= self.upper {
                        self.current = Some(self.base.iter_subprefixes(next_length));
                        self.current.as_mut()?.next()
                    } else {
                        None
                    }
                }
            },
            None => {
                self.current = Some(self.base.iter_subprefixes(self.lower));
                self.current.as_mut()?.next()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::IpPrefixRange;
    use crate::tests::{assert_none, TestResult};
    use crate::Ipv4Prefix;

    #[test]
    fn invalid_lower() -> TestResult {
        let p: Ipv4Prefix = "192.0.2.0/24".parse()?;
        match IpPrefixRange::new(p, 23, 24) {
            Err(_) => Ok(()),
            Ok(_) => Err("Expected 'PrefixLenError'".into()),
        }
    }

    #[test]
    fn invalid_upper() -> TestResult {
        let p: Ipv4Prefix = "192.0.2.0/24".parse()?;
        match IpPrefixRange::new(p, 23, 24) {
            Err(_) => Ok(()),
            Ok(_) => Err("Expected 'PrefixLenError'".into()),
        }
    }

    #[test]
    fn invalid_range() -> TestResult {
        let p: Ipv4Prefix = "192.0.2.0/24".parse()?;
        match IpPrefixRange::new(p, 23, 24) {
            Err(_) => Ok(()),
            Ok(_) => Err("Expected 'PrefixLenError'".into()),
        }
    }

    #[test]
    fn singleton_range() -> TestResult {
        let p: Ipv4Prefix = "192.0.2.0/24".parse()?;
        let r = IpPrefixRange::new(p, 24, 24)?;
        let mut iter = r.into_iter();
        assert_eq!(iter.next().unwrap(), p);
        assert_none(iter.next())?;
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
        assert_none(iter.next())?;
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrefixRangeParseErr(());

impl Error for PrefixRangeParseErr {}

impl fmt::Display for PrefixRangeParseErr {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("invalid IP prefix range syntax")
    }
}
