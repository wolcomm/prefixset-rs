//! [`PrefixSet<P>`] and related types.
use std::iter::IntoIterator;
use std::mem;

use crate::node::Node;
use crate::prefix::IpPrefix;

mod from;
mod iter;
mod ops;

pub use self::iter::{Prefixes, Ranges};

/// A collection of IP prefixes, providing fast insertion and iteration,
/// and set-theorectic arithmetic.
///
/// Most mutating methods return `&mut Self` for easy chaining, e.g.:
///
/// ``` rust
/// # use prefixset::{Error, Ipv4Prefix, PrefixSet};
/// # fn main() -> Result<(), Error> {
/// let set = PrefixSet::new()
///     .insert("192.0.2.0/24".parse::<Ipv4Prefix>()?)
///     .to_owned();
/// assert_eq!(set.len(), 1);
/// #     Ok(())
/// # }
/// ```
#[derive(Clone, Debug)]
pub struct PrefixSet<P: IpPrefix> {
    root: Option<Box<Node<P>>>,
}

impl<P: IpPrefix> PrefixSet<P> {
    /// Construct a new, empty [`PrefixSet<P>`].
    pub fn new() -> Self {
        PrefixSet { root: None }
    }

    fn new_with_root(root: Option<Box<Node<P>>>) -> Self {
        PrefixSet { root }
    }

    fn insert_node(&mut self, new: Box<Node<P>>) -> &mut Self {
        match mem::take(&mut self.root) {
            Some(root) => {
                self.root = Some(root.add(new));
            }
            None => {
                self.root = Some(new);
            }
        };
        self
    }

    /// Insert a new `item` into `self`.
    ///
    /// `T` can be either `P` or [`IpPrefixRange<P>`](crate::IpPrefixRange).
    ///
    /// ``` rust
    /// # use prefixset::{Error, Ipv6Prefix, IpPrefixRange, PrefixSet};
    /// # fn main() -> Result<(), Error> {
    /// let range = IpPrefixRange::new(
    ///     "2001:db8:f00::/48".parse::<Ipv6Prefix>()?, 64, 64,
    /// )?;
    /// let set = PrefixSet::new()
    ///     .insert(range)
    ///     .to_owned();
    /// assert_eq!(set.len(), 1 << 16);
    /// #     Ok(())
    /// # }
    /// ```
    pub fn insert<T>(&mut self, item: T) -> &mut Self
    where
        T: Into<Box<Node<P>>>,
    {
        self.insert_node(item.into()).aggregate()
    }

    /// Insert a items into `self` from an iterator yielding either `P` or
    /// [`IpPrefixRange<P>`](crate::IpPrefixRange).
    ///
    /// Aggregation occurs after all items are inserted, making this far more
    /// efficient than calling [`PrefixSet::insert()`] repeatedly.
    ///
    /// ``` rust
    /// # use prefixset::{Error, Ipv4Prefix, PrefixSet};
    /// # fn main() -> Result<(), Error> {
    /// let prefixes: Vec<_> = vec!["192.0.2.0/26", "192.0.2.64/26"]
    ///     .into_iter()
    ///     .map(|s| s.parse::<Ipv4Prefix>())
    ///     .collect::<Result<_, _>>()?;
    /// let set = PrefixSet::new()
    ///     .insert_from(prefixes)
    ///     .to_owned();
    /// assert_eq!(set.len(), 2);
    /// #     Ok(())
    /// # }
    /// ```
    pub fn insert_from<I, T>(&mut self, iter: I) -> &mut Self
    where
        I: IntoIterator<Item = T>,
        T: Into<Box<Node<P>>>,
    {
        iter.into_iter()
            .fold(self, |set, item| set.insert_node(item.into()))
            .aggregate()
    }

    fn remove_node(&mut self, mut old: Box<Node<P>>) -> &mut Self {
        if let Some(root) = mem::take(&mut self.root) {
            self.root = Some(root.remove(&mut old));
        };
        self
    }

    /// Remove an `item` from `self`.
    ///
    /// `T` can be either `P` or [`IpPrefixRange<P>`](crate::IpPrefixRange).
    ///
    /// ``` rust
    /// # use prefixset::{Error, Ipv6Prefix, IpPrefixRange, PrefixSet};
    /// # fn main() -> Result<(), Error> {
    /// let set = [
    ///         "2001:db8:f00::/48",
    ///         "2001:db8:baa::/48",
    ///     ]
    ///     .into_iter()
    ///     .map(|s| s.parse::<Ipv6Prefix>())
    ///     .collect::<Result<PrefixSet<_>, _>>()?
    ///     .remove("2001:db8:f00::/48".parse::<Ipv6Prefix>()?)
    ///     .to_owned();
    /// assert_eq!(set.len(), 1);
    /// #     Ok(())
    /// # }
    /// ```
    pub fn remove<T>(&mut self, item: T) -> &mut Self
    where
        T: Into<Box<Node<P>>>,
    {
        self.remove_node(item.into()).aggregate()
    }

    /// Remove items from `self` from an iterator yielding either `P` or
    /// [`IpPrefixRange<P>`](crate::IpPrefixRange).
    ///
    /// Aggregation occurs after all items are removed, making this far more
    /// efficient than calling [`PrefixSet::remove()`] repeatedly.
    ///
    /// ``` rust
    /// # use prefixset::{Error, Ipv4Prefix, IpPrefixRange, PrefixSet};
    /// # fn main() -> Result<(), Error> {
    /// let prefixes: Vec<_> = vec!["192.0.2.0/26", "192.0.2.64/26"]
    ///     .into_iter()
    ///     .map(|s| s.parse::<Ipv4Prefix>())
    ///     .collect::<Result<_, _>>()?;
    /// let mut set = PrefixSet::new()
    ///     .insert("192.0.2.0/24,26,26".parse::<IpPrefixRange<Ipv4Prefix>>()?)
    ///     .to_owned();
    /// assert_eq!(set.remove_from(prefixes).len(), 2);
    /// #     Ok(())
    /// # }
    /// ```
    pub fn remove_from<I, T>(&mut self, iter: I) -> &mut Self
    where
        I: IntoIterator<Item = T>,
        T: Into<Box<Node<P>>>,
    {
        iter.into_iter()
            .fold(self, |set, item| set.remove_node(item.into()))
            .aggregate()
    }

    fn aggregate(&mut self) -> &mut Self {
        if let Some(root) = mem::take(&mut self.root) {
            self.root = root.aggregate(None)
        }
        self
    }

    /// Test whether `prefix` is contained in `self`.
    ///
    /// ``` rust
    /// # use prefixset::{Error, Ipv4Prefix, IpPrefixRange, PrefixSet};
    /// # fn main() -> Result<(), Error> {
    /// let set = PrefixSet::new()
    ///     .insert("192.0.2.0/24,26,26".parse::<IpPrefixRange<Ipv4Prefix>>()?)
    ///     .to_owned();
    /// assert!(set.contains(&"192.0.2.128/26".parse()?));
    /// #     Ok(())
    /// # }
    /// ```
    pub fn contains(&self, prefix: &P) -> bool {
        match &self.root {
            Some(root) => root.search(&prefix.into()).is_some(),
            None => false,
        }
    }

    /// Get the number of prefixes in `self`.
    ///
    /// ``` rust
    /// # use prefixset::{Error, Ipv4Prefix, IpPrefixRange, PrefixSet};
    /// # fn main() -> Result<(), Error> {
    /// let set = PrefixSet::new()
    ///     .insert("192.0.2.0/24,26,26".parse::<IpPrefixRange<Ipv4Prefix>>()?)
    ///     .to_owned();
    /// assert_eq!(set.len(), 4);
    /// #     Ok(())
    /// # }
    /// ```
    pub fn len(&self) -> usize {
        self.prefixes().count()
    }

    /// Test whether `self` is empty.
    ///
    /// ``` rust
    /// # use prefixset::{Error, Ipv4Prefix, PrefixSet};
    /// # fn main() -> Result<(), Error> {
    /// assert!(PrefixSet::<Ipv4Prefix>::new().is_empty());
    /// #     Ok(())
    /// # }
    /// ```
    pub fn is_empty(&self) -> bool {
        self.ranges().count() == 0
    }

    /// Clear the contents of `self`
    ///
    /// ``` rust
    /// # use prefixset::{Error, Ipv6Prefix, PrefixSet};
    /// # fn main() -> Result<(), Error> {
    /// let mut set = PrefixSet::new()
    ///     .insert("2001:db8::/32".parse::<Ipv6Prefix>()?)
    ///     .to_owned();
    /// set.clear();
    /// assert!(set.is_empty());
    /// #     Ok(())
    /// # }
    /// ```
    pub fn clear(&mut self) {
        self.root = None
    }

    /// Get an iterator over the [`IpPrefixRange<P>`](crate::IpPrefixRange)s
    /// contained in `self`.
    ///
    /// ``` rust
    /// # use prefixset::{Error, Ipv4Prefix, PrefixSet};
    /// # fn main() -> Result<(), Error> {
    /// let set = PrefixSet::new()
    ///     .insert("192.0.2.0/25".parse::<Ipv4Prefix>()?)
    ///     .insert("192.0.2.128/25".parse::<Ipv4Prefix>()?)
    ///     .to_owned();
    /// let mut ranges = set.ranges();
    /// assert_eq!(ranges.next(), Some("192.0.2.0/24,25,25".parse()?));
    /// assert_eq!(ranges.next(), None);
    /// #     Ok(())
    /// # }
    /// ```
    pub fn ranges(&self) -> Ranges<P> {
        self.into()
    }

    /// Get an iterator over the prefixes contained in `self`.
    ///
    /// ``` rust
    /// # use prefixset::{Error, Ipv4Prefix, PrefixSet};
    /// # fn main() -> Result<(), Error> {
    /// let set = PrefixSet::new()
    ///     .insert("192.0.2.0/25".parse::<Ipv4Prefix>()?)
    ///     .insert("192.0.2.128/25".parse::<Ipv4Prefix>()?)
    ///     .to_owned();
    /// let mut prefixes = set.prefixes();
    /// assert_eq!(prefixes.next(), Some("192.0.2.0/25".parse()?));
    /// assert_eq!(prefixes.next(), Some("192.0.2.128/25".parse()?));
    /// assert_eq!(prefixes.next(), None);
    /// #     Ok(())
    /// # }
    /// ```
    pub fn prefixes(&self) -> Prefixes<P> {
        self.into()
    }
}

impl<P: IpPrefix> Default for PrefixSet<P> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, P: IpPrefix> IntoIterator for &'a PrefixSet<P> {
    type Item = P;
    type IntoIter = Prefixes<'a, P>;

    fn into_iter(self) -> Self::IntoIter {
        self.prefixes()
    }
}

impl<P, A> Extend<A> for PrefixSet<P>
where
    P: IpPrefix,
    A: Into<Box<Node<P>>>,
{
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = A>,
    {
        self.insert_from(iter);
    }
}

#[cfg(test)]
mod tests;
