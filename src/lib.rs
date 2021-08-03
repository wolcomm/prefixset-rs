//! This crate provides [`PrefixSet`], a set-like container for IP prefixes
//! (*not* IP addresses).
//!
//! Sets of prefixes are stored in a binary radix tree structure that provides:
//!
//! - Fast insertion of contigious prefix ranges in a single traversal,
//! - Iteration over either prefixes or ranges of prefixes, and
//! - Self aggregation on each operation.
//!
//! This is a Rust implementation derived in large part from the internal
//! data-structure used in the widely used [`bgpq3`] tool by Alexandre Snarskii,
//! packaged as a library, and with the set-theorectic operations added.
//!
//! # Quickstart
//!
//! ``` rust
//! extern crate prefixset;
//! use prefixset::{Error, Ipv6Prefix, IpPrefixRange, PrefixSet};
//!
//! fn main() -> Result<(), Error> {
//!     
//!     // create a set by parsing a Vec<&str>
//!     let set = vec![
//!             "2001:db8::/37",
//!             "2001:db8:f00::/37",
//!         ]
//!         .iter()
//!         .map(|s| s.parse::<Ipv6Prefix>())
//!         .collect::<Result<PrefixSet<_>, _>>()?;
//!
//!     // create a range by parsing a &str and providing the lower
//!     // and upper prefix lenth bounds
//!     let range = IpPrefixRange::new("2001:db8::/36".parse()?, 37, 37)?;
//!
//!     assert_eq!(set.ranges().collect::<Vec<_>>(), vec![range]);
//!     Ok(())
//! }
//! ```
//!
//! [`bgpq3`]: https://github.com/snar/bgpq3
//!
#![doc(html_root_url = "https://docs.rs/prefixset/0.1.0")]
#![warn(missing_docs)]

extern crate ipnet;
extern crate num;

pub mod prefix;
pub mod set;

mod error;
mod node;

#[cfg(test)]
mod tests;

pub use crate::error::Error;
#[doc(inline)]
pub use crate::prefix::{IpPrefix, IpPrefixRange, Ipv4Prefix, Ipv6Prefix};
#[doc(inline)]
pub use crate::set::PrefixSet;
