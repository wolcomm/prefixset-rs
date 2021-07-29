extern crate itertools;
extern crate utils;

use utils::{data_sets, DataSet};

use std::iter::FromIterator;
use std::str::FromStr;
use std::time::Duration;

use criterion::{criterion_main, Criterion, Throughput};
use itertools::Itertools;

use prefixset::{IpPrefix, IpPrefixRange, Ipv4Prefix, Ipv6Prefix, PrefixSet};

trait BenchHelper<'a, T>
where
    T: 'a + Clone + FromStr,
    <T as FromStr>::Err: std::fmt::Debug,
    Self: IpPrefix,
    <Self as FromStr>::Err: std::fmt::Debug,
    PrefixSet<Self>: FromIterator<T>,
{
    const DATA_SETS: &'a [DataSet<T>];

    fn benches(mut c: &mut Criterion) {
        Self::construct(&mut c);
        Self::iterate_prefix_ranges(&mut c);
        Self::iterate_prefixes(&mut c);
        Self::compute_intersection(&mut c);
        Self::compute_union(&mut c);
        Self::compute_difference(&mut c);
    }

    fn construct(c: &mut Criterion) {
        let mut g = c.benchmark_group("construction");
        g.measurement_time(Duration::from_secs(30));
        g.sample_size(50);

        for ds in Self::DATA_SETS {
            let prefixes = ds.read();
            g.throughput(Throughput::Elements(prefixes.len() as u64));
            g.bench_function(ds.name(), |b| {
                b.iter(|| -> PrefixSet<Self> { prefixes.to_owned().into_iter().collect() })
            });
        }
        g.finish()
    }

    fn iterate_prefix_ranges(c: &mut Criterion) {
        let mut g = c.benchmark_group("prefix range iteration");
        g.measurement_time(Duration::from_secs(10));

        for ds in Self::DATA_SETS {
            let set: PrefixSet<Self> = ds.read().into_iter().collect();
            g.throughput(Throughput::Elements(ds.ranges() as u64));
            g.bench_function(ds.name(), |b| {
                b.iter(|| assert_eq!(set.iter_prefix_ranges().count(), ds.ranges()))
            });
        }
        g.finish()
    }

    fn iterate_prefixes(c: &mut Criterion) {
        let mut g = c.benchmark_group("prefix iteration");
        g.measurement_time(Duration::from_secs(10));

        for ds in Self::DATA_SETS {
            let set: PrefixSet<Self> = ds.read().into_iter().collect();
            g.throughput(Throughput::Elements(ds.prefixes() as u64));
            g.bench_function(ds.name(), |b| {
                b.iter(|| assert_eq!(set.iter_prefixes().count(), ds.prefixes()))
            });
        }
        g.finish()
    }

    fn compute_intersection(c: &mut Criterion) {
        let mut g = c.benchmark_group("intersection computation");
        g.measurement_time(Duration::from_secs(30));
        g.sample_size(20);

        Self::DATA_SETS
            .iter()
            .tuple_combinations()
            .for_each(|(x, y)| {
                let name = format!("{} & {}", x.name(), y.name());
                let s: PrefixSet<_> = x.read().into_iter().collect();
                let t: PrefixSet<_> = y.read().into_iter().collect();
                g.bench_function(name, |b| b.iter(|| s.clone() & t.clone()));
            });
        g.finish()
    }

    fn compute_union(c: &mut Criterion) {
        let mut g = c.benchmark_group("union computation");
        g.measurement_time(Duration::from_secs(30));
        g.sample_size(20);

        Self::DATA_SETS
            .iter()
            .tuple_combinations()
            .for_each(|(x, y)| {
                let name = format!("{} | {}", x.name(), y.name());
                let s: PrefixSet<_> = x.read().into_iter().collect();
                let t: PrefixSet<_> = y.read().into_iter().collect();
                g.bench_function(name, |b| b.iter(|| s.clone() | t.clone()));
            });
        g.finish()
    }

    fn compute_difference(c: &mut Criterion) {
        let mut g = c.benchmark_group("difference computation");
        g.measurement_time(Duration::from_secs(30));
        g.sample_size(20);

        Self::DATA_SETS
            .iter()
            .tuple_combinations()
            .for_each(|(x, y)| {
                let name = format!("{} ^ {}", x.name(), y.name());
                let s: PrefixSet<_> = x.read().into_iter().collect();
                let t: PrefixSet<_> = y.read().into_iter().collect();
                g.bench_function(name, |b| b.iter(|| s.clone() ^ t.clone()));
            });
        g.finish()
    }
}

impl<'a> BenchHelper<'a, Ipv4Prefix> for Ipv4Prefix {
    const DATA_SETS: &'a [DataSet<Ipv4Prefix>] = data_sets!(
        name = "AS-WOLCOMM-ipv4-prefixes", prefixes = 755053, ranges = 163330;
        name = "AS-HURRICANE-ipv4-prefixes", prefixes = 817756, ranges = 145101;
    );
}
impl<'a> BenchHelper<'a, IpPrefixRange<Ipv4Prefix>> for Ipv4Prefix {
    const DATA_SETS: &'a [DataSet<IpPrefixRange<Ipv4Prefix>>] = data_sets!(
        name = "AS-WOLCOMM-ipv4-ranges", prefixes = 755053, ranges = 163330;
        name = "AS-HURRICANE-ipv4-ranges", prefixes = 817756, ranges = 145101;
    );
}
impl<'a> BenchHelper<'a, Ipv6Prefix> for Ipv6Prefix {
    const DATA_SETS: &'a [DataSet<Ipv6Prefix>] = data_sets!(
        name = "AS-WOLCOMM-ipv6-prefixes", prefixes = 274714, ranges = 34740;
        name = "AS-HURRICANE-ipv6-prefixes", prefixes = 218805, ranges = 24774;
    );
}
impl<'a> BenchHelper<'a, IpPrefixRange<Ipv6Prefix>> for Ipv6Prefix {
    const DATA_SETS: &'a [DataSet<IpPrefixRange<Ipv6Prefix>>] = data_sets!(
        name = "AS-WOLCOMM-ipv6-ranges", prefixes = 274714, ranges = 34740;
        name = "AS-HURRICANE-ipv6-ranges", prefixes = 218805, ranges = 24774;
    );
}

fn benches() {
    let mut c = Criterion::default().configure_from_args();
    <Ipv4Prefix as BenchHelper<Ipv4Prefix>>::benches(&mut c);
    <Ipv4Prefix as BenchHelper<IpPrefixRange<Ipv4Prefix>>>::benches(&mut c);
    <Ipv6Prefix as BenchHelper<Ipv6Prefix>>::benches(&mut c);
    <Ipv6Prefix as BenchHelper<IpPrefixRange<Ipv6Prefix>>>::benches(&mut c);
}
criterion_main!(benches);
