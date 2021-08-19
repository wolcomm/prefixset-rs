use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

use prefixset::{IpPrefix, IpPrefixRange, Ipv4Prefix, PrefixSet};

/// Read a list of IPv4 prefix ranges from a file, collect them into a `PrefixSet`
/// and print the contained ranges.
fn main() -> Result<(), Box<dyn Error>> {
    let ranges = read_ranges::<Ipv4Prefix>("AS-WOLCOMM-ipv4-ranges")?;
    let set: PrefixSet<_> = ranges.iter().collect();
    set.ranges().for_each(|range| println!("{}", range));
    Ok(())
}

fn read_ranges<P: IpPrefix>(name: &str) -> Result<Vec<IpPrefixRange<P>>, Box<dyn Error>> {
    let path = format!("./test_data/{}.txt", name);
    let file = File::open(path)?;
    BufReader::new(file)
        .lines()
        .map(|line| Ok(line?.parse::<IpPrefixRange<P>>()?))
        .collect::<Result<_, Box<dyn Error>>>()
}
