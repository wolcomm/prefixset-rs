use std::error::Error;

use ip::{Ipv4, Prefix};

use prefixset::PrefixSet;

/// Collect a `Vec<&str>` into a `PrefixSet<Ipv4>` and
/// print the contained ranges.
///
/// # Comparison with `bgpq3`
///
/// ``` sh
/// $  whois -h whois.radb.net AS37271:RS-EXAMPLE
/// route-set:  AS37271:RS-EXAMPLE
/// mp-members: 192.0.2.0/27
/// mp-members: 192.0.2.32/27
/// mp-members: 192.0.2.64/27
/// mp-members: 192.0.2.96/27
/// mp-members: 192.0.2.128/26
/// mp-members: 192.0.2.128/27
/// mp-members: 192.0.2.160/27
/// mp-members: 192.0.2.192/27
/// mp-members: 192.0.2.224/27
/// descr:      Example route-set
/// mnt-by:     MAINT-AS37271
/// changed:    benm@workonline.africa 20210819
/// source:     RADB
/// $
/// $  bgpq3 -A AS37271:RS-EXAMPLE -l RS-EXAMPLE
/// no ip prefix-list RS-EXAMPLE
/// ip prefix-list RS-EXAMPLE permit 192.0.2.0/25 ge 27 le 27
/// ip prefix-list RS-EXAMPLE permit 192.0.2.128/26 le 27
/// ip prefix-list RS-EXAMPLE permit 192.0.2.192/26 ge 27 le 27
/// ```
///
fn main() -> Result<(), Box<dyn Error>> {
    let set: PrefixSet<_> = vec![
        "192.0.2.0/27",
        "192.0.2.32/27",
        "192.0.2.64/27",
        "192.0.2.96/27",
        "192.0.2.128/26",
        "192.0.2.128/27",
        "192.0.2.160/27",
        "192.0.2.192/27",
        "192.0.2.224/27",
    ]
    .into_iter()
    .map(|prefix| prefix.parse::<Prefix<Ipv4>>())
    .collect::<Result<_, _>>()?;
    set.ranges().for_each(|range| println!("{}", range));
    Ok(())
}
