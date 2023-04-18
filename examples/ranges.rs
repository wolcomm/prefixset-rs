use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

use ip::{Afi, Ipv4, PrefixRange};

use prefixset::PrefixSet;

// use prefixset::{IpPrefix, IpPrefixRange, Ipv4Prefix, PrefixSet};

/// Read a list of IPv4 prefix ranges from a file, collect them into a `PrefixSet`
/// and print the contained ranges.
fn main() -> Result<(), Box<dyn Error>> {
    let ranges = read_ranges::<Ipv4>("AS-WOLCOMM-ipv4-ranges")?;
    let set: PrefixSet<Ipv4> = ranges.into_iter().collect();
    set.ranges().for_each(|range| println!("{}", range));
    Ok(())
}

fn read_ranges<A: Afi>(name: &str) -> Result<Vec<PrefixRange<A>>, Box<dyn Error>> {
    let path = format!("./test_data/{}.txt", name);
    let file = File::open(path)?;
    BufReader::new(file)
        .lines()
        .map(|line| Ok(line?.parse::<PrefixRange<A>>()?))
        .collect::<Result<_, Box<dyn Error>>>()
}
