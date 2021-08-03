extern crate ipnet;
extern crate num;

mod error;
mod node;
mod prefix;
mod set;

#[cfg(test)]
mod tests;

pub use crate::prefix::{IpPrefix, IpPrefixRange, Ipv4Prefix, Ipv6Prefix};
pub use crate::set::PrefixSet;
