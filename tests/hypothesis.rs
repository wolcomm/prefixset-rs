use std::collections::HashSet;
use std::iter::FromIterator;
use std::ops::Deref;

use itertools::Itertools;
use num::PrimInt;
use proptest::{
    prelude::*,
    arbitrary::{ParamsFor, StrategyFor},
};

use prefixset::{IpPrefix, Ipv4Prefix, Ipv6Prefix, PrefixSet};

#[derive(Clone, Debug)]
struct TestPrefix<P: IpPrefix>(P);

impl<P: IpPrefix> Deref for TestPrefix<P> {
    type Target = P;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<P> Arbitrary for TestPrefix<P>
where
    P: IpPrefix,
    P::BitMap: Arbitrary,
    StrategyFor<P::BitMap>: 'static,
{
    type Parameters = ParamsFor<P::BitMap>;
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
        any_with::<P::BitMap>(args)
            .prop_flat_map(|bits| {
                let min_length = P::MAX_LENGTH - bits.trailing_zeros() as u8;
                (Just(bits), min_length..=P::MAX_LENGTH)
                    .prop_map(|(addr, length)| {
                        TestPrefix(P::new(addr, length).unwrap())
                    })
            })
            .boxed()
    }
}

#[derive(Clone, Debug)]
struct TestPrefixSet<P: IpPrefix> {
    ps: PrefixSet<P>,
    cs: HashSet<P>,
}

impl<P: IpPrefix> FromIterator<TestPrefix<P>> for TestPrefixSet<P> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item=TestPrefix<P>>
    {
        let (ps_iter, cs_iter) = iter.into_iter().tee();
        let ps = ps_iter.into_iter().map(|p| *p).collect();
        let cs = cs_iter.into_iter().map(|p| *p).collect();
        Self { ps, cs }
    }
}

#[derive(Clone)]
struct TestPrefixSetParams<P>(ParamsFor<Vec<TestPrefix<P>>>)
where
    P: IpPrefix,
    TestPrefix<P>: Arbitrary,
    ParamsFor<TestPrefix<P>>: Clone;

impl<P> Default for TestPrefixSetParams<P>
where
    P: IpPrefix,
    TestPrefix<P>: Arbitrary,
    ParamsFor<TestPrefix<P>>: Clone,
{
    fn default() -> Self {
        Self(((500..2000usize).into(), Default::default()))
    }
}

impl<P> Deref for TestPrefixSetParams<P>
where
    P: IpPrefix,
    TestPrefix<P>: Arbitrary,
    ParamsFor<TestPrefix<P>>: Clone,
{
    type Target = ParamsFor<Vec<TestPrefix<P>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<P> Arbitrary for TestPrefixSet<P>
where
    P: IpPrefix + 'static,
    TestPrefix<P>: Arbitrary,
    ParamsFor<TestPrefix<P>>: Clone,
{
    type Parameters = TestPrefixSetParams<P>;
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
        any_with::<Vec<TestPrefix<P>>>((*args).clone())
            .prop_map(|v| v.into_iter().collect())
            .boxed()
    }
}

macro_rules! property_tests {
    ( $( $mod:ident => $p:ty ),* $(,)? ) => {
        $(
            mod $mod {
                use super::*;

                proptest! {
                    #[test]
                    fn prefix_set_size(
                        s in any::<TestPrefixSet<$p>>(),
                    ) {
                        prop_assert_eq!(
                            dbg!(s.ps).iter_prefixes().count(),
                            dbg!(s.cs).len()
                        );
                    }

                    #[test]
                    fn prefix_set_contains(
                        s in any::<TestPrefixSet<$p>>(),
                    ) {
                        prop_assert!(
                            s.cs.to_owned()
                                .into_iter()
                                .all(|p| s.ps.contains(p))
                        );
                    }

                    #[test]
                    fn prefix_set_contained(
                        s in any::<TestPrefixSet<$p>>(),
                    ) {
                        prop_assert!(
                            s.ps.iter_prefixes()
                                .all(|p| s.cs.contains(&p))
                        );
                    }

                    #[test]
                    fn intersections_match(
                        s in any::<TestPrefixSet<$p>>(),
                        t in any::<TestPrefixSet<$p>>(),
                    ) {
                        prop_assert_eq!(
                            (s.ps & t.ps)
                                .iter_prefixes()
                                .collect::<HashSet<_>>(),
                            &s.cs & &t.cs
                        )
                    }

                    #[test]
                    fn unions_match(
                        s in any::<TestPrefixSet<$p>>(),
                        t in any::<TestPrefixSet<$p>>(),
                    ) {
                        prop_assert_eq!(
                            (s.ps | t.ps)
                                .iter_prefixes()
                                .collect::<HashSet<_>>(),
                            &s.cs | &t.cs
                        )
                    }

                    #[test]
                    fn differences_match(
                        s in any::<TestPrefixSet<$p>>(),
                        t in any::<TestPrefixSet<$p>>(),
                    ) {
                        prop_assert_eq!(
                            (s.ps - t.ps)
                                .iter_prefixes()
                                .collect::<HashSet<_>>(),
                            &s.cs - &t.cs
                        )
                    }

                    #[test]
                    fn symmetric_differences_match(
                        s in any::<TestPrefixSet<$p>>(),
                        t in any::<TestPrefixSet<$p>>(),
                    ) {
                        prop_assert_eq!(
                            (s.ps ^ t.ps)
                                .iter_prefixes()
                                .collect::<HashSet<_>>(),
                            &s.cs ^ &t.cs
                        )
                    }
                }
            }
        )*
    }
}

property_tests! {
    ipv4 => Ipv4Prefix,
    ipv6 => Ipv6Prefix,
}
