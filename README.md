# prefixset-rs

[![Crates.io](https://img.shields.io/crates/v/prefixset)](https://crates.io/crates/prefixset)
[![ci/cd](https://github.com/wolcomm/prefixset-rs/actions/workflows/cicd.yml/badge.svg?event=push)](https://github.com/wolcomm/prefixset-rs/actions/workflows/cicd.yml)
[![codecov](https://codecov.io/gh/wolcomm/prefixset-rs/branch/master/graph/badge.svg?token=9dktFtdydp)](https://codecov.io/gh/wolcomm/prefixset-rs)
[![docs.rs](https://img.shields.io/docsrs/prefixset)](https://docs.rs/prefixset)

## About

A Rust library crate `prefixset`, providing a set-like container for IP
prefixes (*not* IP addresses).

Sets of prefixes are stored in a binary radix tree structure that provides:

- Fast insertion of contiguous prefix ranges in a single traversal,
- Iteration over either prefixes or ranges of prefixes, and
- Self aggregation on each operation.

## Prior art

This is a Rust implementation derived in large part from the internal
data-structure used in the widely used [`bgpq3`] tool by Alexandre Snarskii,
packaged as a library, and with the set-theoretic operations added.

## Usage

Full documentation can be found [here](https://docs.rs/prefixset/).

``` rust
extern crate prefixset;
use prefixset::{Error, Ipv6Prefix, IpPrefixRange, PrefixSet};

fn main() -> Result<(), Error> {

    // create a set by parsing a Vec<&str>
    let set = vec![
            "2001:db8::/37",
            "2001:db8:f00::/37",
        ]
        .iter()
        .map(|s| s.parse::<Ipv6Prefix>())
        .collect::<Result<PrefixSet<_>, _>>()?;

    // create a range by parsing a &str and providing the lower
    // and upper prefix lenth bounds
    let range = IpPrefixRange::new("2001:db8::/36".parse()?, 37, 37)?;

    assert_eq!(set.ranges().collect::<Vec<_>>(), vec![range]);
    Ok(())
}
```

[`bgpq3`]: https://github.com/snar/bgpq3
