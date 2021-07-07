use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;
use std::time::Duration;

use criterion::{criterion_group, criterion_main, Criterion};

use prefixset::{IpPrefix, Ipv4Prefix, Ipv6Prefix, PrefixSet};

trait BenchHelper
where
    Self: IpPrefix,
    <Self as FromStr>::Err: std::fmt::Debug,
{
    const DATA_SETS: [(u32, &'static str); 1];

    fn read_prefixes(data_set: &str) -> Vec<Self> {
        let path = format!("./benches/data/{}.txt", data_set);
        let file = File::open(path).unwrap();
        BufReader::new(file)
            .lines()
            .into_iter()
            .map(|line| line.unwrap().parse::<Self>().unwrap())
            .collect()
    }

    fn construct_set(prefixes: &Vec<Self>) -> PrefixSet<Self> {
        let mut set = PrefixSet::new();
        for prefix in prefixes {
            set.add(prefix.to_owned()).unwrap();
        }
        set
    }

    fn iterate_set(set: PrefixSet<Self>, expected: u32) {
        let mut i = 0u32;
        set.into_iter().for_each(|_| i += 1);
        assert_eq!(i, expected)
    }
}

impl BenchHelper for Ipv4Prefix {
    const DATA_SETS: [(u32, &'static str); 1] = [
        (725492, "AS-WOLCOMM-IPv4-NONAGG")
    ];
}
impl BenchHelper for Ipv6Prefix {
    const DATA_SETS: [(u32, &'static str); 1] = [
        (273873, "AS-WOLCOMM-IPv6-NONAGG")
    ];
}

trait BenchTest
where
    Self: BenchHelper,
    <Self as FromStr>::Err: std::fmt::Debug,
{
    fn construct(c: &mut Criterion) {
        let mut g = c.benchmark_group("PrefixSet construction");
        g.measurement_time(Duration::from_secs(30));
        g.sample_size(50);

        for (_, data_set) in Self::DATA_SETS {
            let prefixes = Self::read_prefixes(data_set);
            g.bench_function(data_set, |b| {
                b.iter( || Self::construct_set(&prefixes))
            });
        }
        g.finish()
    }

    fn iterate(c: &mut Criterion) {
        let mut g = c.benchmark_group("PrefixSet iteration");
        g.measurement_time(Duration::from_secs(10));

        for (items, data_set) in Self::DATA_SETS {
            let prefixes = Self::read_prefixes(data_set);
            let set = Self::construct_set(&prefixes);
            g.bench_function(data_set, |b| {
                b.iter(|| Self::iterate_set(set.clone(), items))
            });
        };
        g.finish()
    }
}

impl BenchTest for Ipv4Prefix {}
impl BenchTest for Ipv6Prefix {}

criterion_group!(
    benches,
    Ipv4Prefix::construct,
    Ipv4Prefix::iterate,
    Ipv6Prefix::construct,
    Ipv6Prefix::iterate,
);
criterion_main!(benches);
