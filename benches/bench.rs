use std::convert::TryInto;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use criterion::{criterion_group, criterion_main, Criterion};

use prefixset::{IpPrefix, Ipv4Prefix, Ipv6Prefix, PrefixSet};

fn read_prefixes<T>(filename: T) -> Vec<IpPrefix>
where
    T: AsRef<Path>,
{
    let file = File::open(filename).unwrap();
    BufReader::new(file)
        .lines()
        .into_iter()
        .map(|line| line.unwrap().parse::<IpPrefix>().unwrap().try_into().unwrap())
        .collect()
}

mod ipv4 {
    use super::*;

    fn read_prefixes<T>(filename: T) -> Vec<Ipv4Prefix>
    where
        T: AsRef<Path>,
    {
        super::read_prefixes(filename)
            .into_iter()
            .map(|p| p.try_into().unwrap())
            .collect()
    }

    pub fn construct(c: &mut Criterion) {
        let mut g = c.benchmark_group("PrefixSet set construction");
        g.measurement_time(std::time::Duration::from_secs(30))
            .sample_size(50);
        for bench in ["AS-WOLCOMM-IPv4-NONAGG"] {
            let filename = format!("./benches/data/{}.txt", bench);
            let prefixes = read_prefixes(filename);
            g.bench_function(
                bench,
                |b| b.iter(
                    || {
                        let mut set = PrefixSet::new();
                        for prefix in &prefixes {
                            set = set.add(prefix.to_owned()).unwrap();
                        }
                    }
                )
            );
        }
    }
}

mod ipv6 {
    use super::*;

    fn read_prefixes<T>(filename: T) -> Vec<Ipv6Prefix>
    where
        T: AsRef<Path>,
    {
        super::read_prefixes(filename)
            .into_iter()
            .map(|p| p.try_into().unwrap())
            .collect()
    }

    pub fn construct(c: &mut Criterion) {
        let mut g = c.benchmark_group("PrefixSet set construction");
        g.measurement_time(std::time::Duration::from_secs(30))
            .sample_size(50);
        for bench in ["AS-WOLCOMM-IPv6-NONAGG"] {
            let filename = format!("./benches/data/{}.txt", bench);
            let prefixes = read_prefixes(filename);
            g.bench_function(
                bench,
                |b| b.iter(
                    || {
                        let mut set = PrefixSet::new();
                        for prefix in &prefixes {
                            set = set.add(prefix.to_owned()).unwrap();
                        }
                    }
                )
            );
        }
    }
}

criterion_group!(benches, ipv4::construct, ipv6::construct);
criterion_main!(benches);
