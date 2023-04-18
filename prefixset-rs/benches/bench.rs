extern crate itertools;
extern crate utils;

use utils::{data_sets, DataSet};

use std::time::Duration;

use criterion::{criterion_main, Criterion, Throughput};

use ip::{Ipv4, Ipv6, Prefix, PrefixRange};

use itertools::Itertools;

use prefixset::PrefixSet;

macro_rules! benchmarks {
    ( $id:ident: $t:ty =>
        [
            $( name = $name:literal, prefixes = $prefixes:literal, ranges = $ranges:literal );*
            $(;)?
        ]
    ) => {
        mod $id {
            use super::*;

            static DATA_SETS: &[DataSet<$t>] = data_sets!( $(
                name = $name, prefixes = $prefixes, ranges = $ranges
            );* );

            pub fn benches(mut c: &mut Criterion) {
                construct_by_move(&mut c);
                // construct_by_copy(&mut c);
                iterate_prefix_ranges(&mut c);
                iterate_prefixes(&mut c);
                compute_intersection(&mut c);
                compute_union(&mut c);
                compute_difference(&mut c);
            }

            fn construct_by_move(c: &mut Criterion) {
                let mut g = c.benchmark_group("construction by move");
                g.measurement_time(Duration::from_secs(20));
                g.sample_size(20);

                for ds in DATA_SETS {
                    let prefixes = ds.read();
                    g.throughput(Throughput::Elements(prefixes.len() as u64));
                    g.bench_function(ds.name(), |b| {
                        b.iter(|| -> PrefixSet<_> { prefixes.clone().into_iter().collect() })
                    });
                }
                g.finish()
            }

            // fn construct_by_copy(c: &mut Criterion) {
            //     let mut g = c.benchmark_group("construction by copy");
            //     g.measurement_time(Duration::from_secs(20));
            //     g.sample_size(20);
            //
            //     for ds in DATA_SETS {
            //         let prefixes = ds.read();
            //         g.throughput(Throughput::Elements(prefixes.len() as u64));
            //         g.bench_function(ds.name(), |b| {
            //             b.iter(|| -> PrefixSet<_> { prefixes.iter().collect() })
            //         });
            //     }
            //     g.finish()
            // }

            fn iterate_prefix_ranges(c: &mut Criterion) {
                let mut g = c.benchmark_group("prefix range iteration");
                g.measurement_time(Duration::from_secs(10));

                for ds in DATA_SETS {
                    let set: PrefixSet<_> = ds.read().into_iter().collect();
                    g.throughput(Throughput::Elements(ds.ranges() as u64));
                    g.bench_function(ds.name(), |b| {
                        b.iter(|| assert_eq!(set.ranges().count(), ds.ranges()))
                    });
                }
                g.finish()
            }

            fn iterate_prefixes(c: &mut Criterion) {
                let mut g = c.benchmark_group("prefix iteration");
                g.measurement_time(Duration::from_secs(10));

                for ds in DATA_SETS {
                    let set: PrefixSet<_> = ds.read().into_iter().collect();
                    g.throughput(Throughput::Elements(ds.prefixes() as u64));
                    g.bench_function(ds.name(), |b| {
                        b.iter(|| assert_eq!(set.prefixes().count(), ds.prefixes()))
                    });
                }
                g.finish()
            }

            fn compute_intersection(c: &mut Criterion) {
                let mut g = c.benchmark_group("intersection computation");
                g.measurement_time(Duration::from_secs(30));
                g.sample_size(20);

                DATA_SETS.into_iter()
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

                DATA_SETS.iter()
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

                DATA_SETS.iter()
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
    }
}

benchmarks! {
    ipv4_prefixes: Prefix<Ipv4> => [
        name = "AS-WOLCOMM-ipv4-prefixes", prefixes = 755053, ranges = 163330;
        name = "AS-HURRICANE-ipv4-prefixes", prefixes = 817756, ranges = 145101;
    ]
}

benchmarks! {
    ipv4_ranges: PrefixRange<Ipv4> => [
        name = "AS-WOLCOMM-ipv4-ranges", prefixes = 755053, ranges = 163330;
        name = "AS-HURRICANE-ipv4-ranges", prefixes = 817756, ranges = 145101;
    ]
}

benchmarks! {
    ipv6_prefixes: Prefix<Ipv6> => [
        name = "AS-WOLCOMM-ipv6-prefixes", prefixes = 274714, ranges = 34740;
        name = "AS-HURRICANE-ipv6-prefixes", prefixes = 218805, ranges = 24774;
    ]
}

benchmarks! {
    ipv6_ranges: PrefixRange<Ipv6> => [
        name = "AS-WOLCOMM-ipv6-ranges", prefixes = 274714, ranges = 34740;
        name = "AS-HURRICANE-ipv6-ranges", prefixes = 218805, ranges = 24774;
    ]
}

fn benches() {
    let mut c = Criterion::default().configure_from_args();
    ipv4_prefixes::benches(&mut c);
    ipv6_prefixes::benches(&mut c);
    ipv4_ranges::benches(&mut c);
    ipv6_ranges::benches(&mut c);
}

criterion_main!(benches);
