//! This crate provides [`PrefixSet`], a set-like container for IP prefixes
//! (*not* IP addresses).
//!
//! Sets of prefixes are stored in a binary radix tree structure that provides:
//!
//! - Fast insertion of contiguous prefix ranges in a single traversal,
//! - Iteration over either prefixes or ranges of prefixes, and
//! - Self aggregation on each operation.
//!
//! This is a Rust implementation derived in large part from the internal
//! data-structure used in the widely used [`bgpq3`] tool by Alexandre Snarskii,
//! packaged as a library, and with set-theoretic operations added.
//!
//! # Quickstart
//!
//! ``` rust
//! use ip::{Ipv6, Prefix, PrefixLength, PrefixRange};
//!
//! use prefixset::{Error, PrefixSet};
//!
//! fn main() -> Result<(), Error> {
//!     
//!     // create a set by parsing a Vec<&str>
//!     let set = vec![
//!             "2001:db8::/37",
//!             "2001:db8:f00::/37",
//!         ]
//!         .iter()
//!         .map(|s| s.parse::<Prefix<Ipv6>>())
//!         .collect::<Result<PrefixSet<_>, _>>()?;
//!
//!     // create a range by parsing a &str and providing the lower
//!     // and upper prefix lenth bounds
//!     let length = PrefixLength::<Ipv6>::from_primitive(37)?;
//!     let range = PrefixRange::<Ipv6>::new("2001:db8::/36".parse()?, length..=length)?;
//!
//!     assert_eq!(set.ranges().collect::<Vec<_>>(), vec![range]);
//!     Ok(())
//! }
//! ```
//!
//! [`bgpq3`]: https://github.com/snar/bgpq3
//!
#![doc(html_root_url = "https://docs.rs/prefixset/0.1.0-rc.2")]
#![warn(missing_docs)]

pub mod set;

mod error;
mod node;

#[cfg(test)]
mod tests;

pub use crate::error::Error;
#[doc(inline)]
pub use crate::set::PrefixSet;
