use std::cmp::PartialEq;
use std::ops::{Add, BitAnd, BitOr, BitXor, Mul, Not, Sub};

use num::{One, Zero};

use crate::prefix::{IpPrefix, IpPrefixRange};

use super::PrefixSet;

impl<P: IpPrefix> Zero for PrefixSet<P> {
    fn zero() -> Self {
        Self::new()
    }

    fn is_zero(&self) -> bool {
        self.root.is_some()
    }
}

impl<P: IpPrefix> One for PrefixSet<P> {
    fn one() -> Self {
        Self::new()
            .insert(
                IpPrefixRange::new(P::new(P::BitMap::zero(), 0).unwrap(), 0, P::MAX_LENGTH)
                    .unwrap(),
            )
            .to_owned()
    }
}

impl<P: IpPrefix> BitAnd for PrefixSet<P> {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        match (self.root, rhs.root) {
            (Some(r), Some(s)) => Self::Output::new_with_root(r & s).aggregate().to_owned(),
            _ => PrefixSet::zero(),
        }
    }
}

impl<P: IpPrefix> BitOr for PrefixSet<P> {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        match (&self.root, &rhs.root) {
            (Some(r), Some(s)) => Self::Output::new_with_root(r.to_owned() | s.to_owned())
                .aggregate()
                .to_owned(),
            (Some(_), None) => self,
            (None, Some(_)) => rhs,
            (None, None) => Self::Output::zero(),
        }
    }
}

impl<P: IpPrefix> BitXor for PrefixSet<P> {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        (self.to_owned() | rhs.to_owned()) - (self & rhs)
    }
}

impl<P: IpPrefix> Not for PrefixSet<P> {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self::Output::one() - self
    }
}

impl<P: IpPrefix> Add for PrefixSet<P> {
    type Output = Self;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn add(self, rhs: Self) -> Self::Output {
        self | rhs
    }
}

impl<P: IpPrefix> Sub for PrefixSet<P> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        match (&self.root, &rhs.root) {
            (Some(r), Some(s)) => Self::Output::new_with_root(r.to_owned() - s.to_owned())
                .aggregate()
                .to_owned(),
            _ => self,
        }
    }
}

impl<P: IpPrefix> Mul for PrefixSet<P> {
    type Output = Self;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn mul(self, rhs: Self) -> Self::Output {
        self & rhs
    }
}

impl<P: IpPrefix> PartialEq for PrefixSet<P> {
    fn eq(&self, other: &Self) -> bool {
        match (&self.root, &other.root) {
            (Some(r), Some(s)) => r.iter_subtree().zip(s.iter_subtree()).all(|(m, n)| m == n),
            (None, None) => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::iter::FromIterator;

    use paste::paste;

    use crate::prefix::{Ipv4Prefix, Ipv6Prefix};
    use crate::tests::TestResult;

    use super::*;

    impl<P: IpPrefix> FromIterator<&'static str> for PrefixSet<P> {
        fn from_iter<I>(iter: I) -> Self
        where
            I: IntoIterator<Item = &'static str>,
        {
            let (ranges, remaining): (Vec<_>, Vec<_>) = iter
                .into_iter()
                .map(|s| s.parse::<IpPrefixRange<P>>().map_err(|_| s))
                .partition(|res| res.is_ok());
            let (prefixes, errors): (Vec<_>, Vec<_>) = remaining
                .into_iter()
                .map(|s| s.unwrap_err().parse::<P>())
                .partition(|res| res.is_ok());
            assert!(errors.is_empty());
            Self::new()
                .insert_from(ranges.into_iter().map(|r| r.unwrap()))
                .insert_from(prefixes.into_iter().map(|p| p.unwrap()))
                .to_owned()
        }
    }

    macro_rules! test_exprs {
        ( $($fn_id:ident {$lhs:expr, $rhs:expr});* ) => {
            test_exprs!(@ipv4 {$($fn_id {$lhs, $rhs});*});
            test_exprs!(@ipv6 {$($fn_id {$lhs, $rhs});*});
        };
        ( @ipv4 {$($fn_id:ident {$lhs:expr, $rhs:expr});*} ) => {
            paste! {
                test_exprs!($(Ipv4Prefix => [<ipv4_ $fn_id>] {$lhs, $rhs});*);
            }
        };
        ( @ipv6 {$($fn_id:ident {$lhs:expr, $rhs:expr});*} ) => {
            paste! {
                test_exprs!($(Ipv6Prefix => [<ipv6_ $fn_id>] {$lhs, $rhs});*);
            }
        };
        ( $($p:ty => $fn_id:ident {$lhs:expr, $rhs:expr});* ) => {
            paste! {
                $(
                    #[test]
                    fn $fn_id() -> TestResult {
                        let res: PrefixSet<$p> = dbg!($lhs);
                        assert_eq!(res, dbg!($rhs));
                        Ok(())
                    }
                )*
            }
        };
    }

    macro_rules! test_unary_op {
        ( $( !$operand:ident == $expect:ident),* ) => {
            test_unary_op!(@call $(not $operand == $expect),*);
        };
        ( @call $($op:ident $operand:ident == $expect:ident),* ) => {
            paste! {
                test_exprs!($(
                    [<$op _ $operand _is_ $expect>] {
                        PrefixSet::$operand().$op(),
                        PrefixSet::$expect()
                    }
                );*);
            }
        }
    }

    macro_rules! test_binary_op {
        ( $($lhs:ident & $rhs:ident == $expect:ident),* ) => {
            test_binary_op!(@call $($lhs bitand $rhs == $expect),*);
        };
        ( $($lhs:ident | $rhs:ident == $expect:ident),* ) => {
            test_binary_op!(@call $($lhs bitor $rhs == $expect),*);
        };
        ( $($lhs:ident ^ $rhs:ident == $expect:ident),* ) => {
            test_binary_op!(@call $($lhs bitxor $rhs == $expect),*);
        };
        ( $($lhs:ident + $rhs:ident == $expect:ident),* ) => {
            test_binary_op!(@call $($lhs add $rhs == $expect),*);
        };
        ( $($lhs:ident - $rhs:ident == $expect:ident),* ) => {
            test_binary_op!(@call $($lhs sub $rhs == $expect),*);
        };
        ( $($lhs:ident * $rhs:ident == $expect:ident),* ) => {
            test_binary_op!(@call $($lhs mul $rhs == $expect),*);
        };
        ( @call $($lhs:ident $op:ident $rhs:ident == $expect:ident),* ) => {
            paste! {
                test_exprs!($(
                    [<$lhs _ $op _ $rhs _is_ $expect>] {
                        PrefixSet::$lhs().$op(PrefixSet::$rhs()),
                        PrefixSet::$expect()
                    }
                );*);
            }
        }
    }

    #[test]
    fn ipv4_zero_set_is_empty() -> TestResult {
        assert_eq!(PrefixSet::<Ipv4Prefix>::zero().iter_prefixes().count(), 0);
        Ok(())
    }

    #[test]
    fn ipv6_zero_set_is_empty() -> TestResult {
        assert_eq!(PrefixSet::<Ipv6Prefix>::zero().iter_prefixes().count(), 0);
        Ok(())
    }

    test_unary_op!(!zero == one, !one == zero);

    test_binary_op!(
        zero & zero == zero,
        zero & one == zero,
        one & zero == zero,
        one & one == one
    );

    test_binary_op!(
        zero | zero == zero,
        zero | one == one,
        one | zero == one,
        one | one == one
    );

    test_binary_op!(
        zero ^ zero == zero,
        zero ^ one == one,
        one ^ zero == one,
        one ^ one == zero
    );

    test_binary_op!(
        zero + zero == zero,
        zero + one == one,
        one + zero == one,
        one + one == one
    );

    test_binary_op!(
        zero - zero == zero,
        zero - one == zero,
        one - zero == one,
        one - one == zero
    );

    test_binary_op!(
        zero * zero == zero,
        zero * one == zero,
        one * zero == zero,
        one * one == one
    );

    test_exprs!( @ipv4 {
        intersect_disjoint_nodes {
            vec!["1.0.0.0/8,8,16"].into_iter().collect::<PrefixSet<_>>()
                & vec!["2.0.0.0/8,8,16"].into(),
            PrefixSet::zero()
        };
        intersect_disjoint_ranges {
            vec!["1.0.0.0/8,8,11"].into_iter().collect::<PrefixSet<_>>()
                & vec!["1.0.0.0/8,12,15"].into(),
            PrefixSet::zero()
        };
        intersect_overlapping_nodes {
            vec!["1.0.0.0/8,12,16"].into_iter().collect::<PrefixSet<_>>()
                & vec!["1.0.0.0/12,12,16"].into(),
            vec!["1.0.0.0/12,12,16"].into()
        };
        intersect_overlapping_ranges {
            vec!["1.0.0.0/8,8,12"].into_iter().collect::<PrefixSet<_>>()
                & vec!["1.0.0.0/8,12,16"].into(),
            vec!["1.0.0.0/8,12,12"].into()
        };
        intersect_overlapping_set_with_parent {
            vec!["1.0.0.0/8", "1.0.0.0/16"].into_iter().collect::<PrefixSet<_>>()
                & vec!["1.0.0.0/16"].into(),
            vec!["1.0.0.0/16"].into()
        };
        intersect_overlapping_set_with_sibling {
            vec!["1.0.0.0/8", "2.0.0.0/8"].into_iter().collect::<PrefixSet<_>>()
                & vec!["1.0.0.0/8"].into(),
            vec!["1.0.0.0/8"].into()
        };
        intersect_overlapping_set_with_child {
            vec!["1.0.0.0/8", "1.0.0.0/16"].into_iter().collect::<PrefixSet<_>>()
                & vec!["1.0.0.0/8"].into(),
            vec!["1.0.0.0/8"].into()
        };
        intersect_covering_parent {
            vec!["1.0.0.0/16"].into_iter().collect::<PrefixSet<_>>()
                & vec!["1.0.0.0/8,16,16"].into(),
            vec!["1.0.0.0/16"].into()
        };
        intersect_covered_child {
            vec!["1.0.0.0/8,16,16"].into_iter().collect::<PrefixSet<_>>()
                & vec!["1.0.0.0/16"].into(),
            vec!["1.0.0.0/16"].into()
        };
        intersect_overlapping_set_with_covered_child {
            vec!["1.0.0.0/8", "1.0.0.0/16"].into_iter().collect::<PrefixSet<_>>()
                & vec!["1.0.0.0/8,16,16"].into(),
            vec!["1.0.0.0/16"].into()
        };
        union_disjoint_nodes {
            vec!["2.0.0.0/8,8,16"].into_iter().collect::<PrefixSet<_>>()
                | vec!["3.0.0.0/8,8,16"].into(),
            vec!["2.0.0.0/7,8,16"].into()
        };
        union_disjoint_ranges {
            vec!["1.0.0.0/8,8,11"].into_iter().collect::<PrefixSet<_>>()
                | vec!["1.0.0.0/8,12,15"].into(),
            vec!["1.0.0.0/8,8,15"].into()
        };
        union_overlapping_nodes {
            vec!["1.0.0.0/8,12,16"].into_iter().collect::<PrefixSet<_>>()
                | vec!["1.0.0.0/12,12,16"].into(),
            vec!["1.0.0.0/8,12,16"].into()
        };
        union_overlapping_ranges {
            vec!["1.0.0.0/8,8,12"].into_iter().collect::<PrefixSet<_>>()
                | vec!["1.0.0.0/8,12,16"].into(),
            vec!["1.0.0.0/8,8,16"].into()
        };
        union_overlapping_set_with_parent {
            vec!["1.0.0.0/8", "1.0.0.0/16"].into_iter().collect::<PrefixSet<_>>()
                | vec!["1.0.0.0/16"].into(),
            vec!["1.0.0.0/8", "1.0.0.0/16"].into()
        };
        union_overlapping_set_with_sibling {
            vec!["1.0.0.0/8", "2.0.0.0/8"].into_iter().collect::<PrefixSet<_>>()
                | vec!["1.0.0.0/8"].into(),
            vec!["1.0.0.0/8", "2.0.0.0/8"].into()
        };
        union_overlapping_set_with_child {
            vec!["1.0.0.0/8", "1.0.0.0/16"].into_iter().collect::<PrefixSet<_>>()
                | vec!["1.0.0.0/8"].into(),
            vec!["1.0.0.0/8", "1.0.0.0/16"].into()
        };
        union_covering_parent {
            vec!["1.0.0.0/16"].into_iter().collect::<PrefixSet<_>>()
                | vec!["1.0.0.0/8,16,16"].into(),
            vec!["1.0.0.0/8,16,16"].into()
        };
        union_covered_child {
            vec!["1.0.0.0/8,16,16"].into_iter().collect::<PrefixSet<_>>()
                | vec!["1.0.0.0/16"].into(),
            vec!["1.0.0.0/8,16,16"].into()
        };
        union_overlapping_set_with_covered_child {
            vec!["1.0.0.0/8", "1.0.0.0/16"].into_iter().collect::<PrefixSet<_>>()
                | vec!["1.0.0.0/8,16,16"].into(),
            vec!["1.0.0.0/8", "1.0.0.0/8,16,16"].into()
        };
        xor_disjoint_nodes {
            vec!["2.0.0.0/8,8,16"].into_iter().collect::<PrefixSet<_>>()
                ^ vec!["3.0.0.0/8,8,16"].into(),
            vec!["2.0.0.0/7,8,16"].into()
        };
        xor_disjoint_ranges {
            vec!["1.0.0.0/8,8,11"].into_iter().collect::<PrefixSet<_>>()
                ^ vec!["1.0.0.0/8,12,15"].into(),
            vec!["1.0.0.0/8,8,15"].into()
        };
        xor_overlapping_nodes {
            vec!["1.0.0.0/8,12,16"].into_iter().collect::<PrefixSet<_>>()
                ^ vec!["1.0.0.0/12,12,16"].into(),
            vec![
                "1.16.0.0/12,12,16",
                "1.32.0.0/11,12,16",
                "1.64.0.0/10,12,16",
                "1.128.0.0/9,12,16"
            ].into()
        };
        xor_overlapping_ranges {
            vec!["1.0.0.0/8,8,12"].into_iter().collect::<PrefixSet<_>>()
                ^ vec!["1.0.0.0/8,12,16"].into(),
            vec!["1.0.0.0/8,8,11", "1.0.0.0/8,13,16"].into()
        };
        xor_overlapping_set_with_parent {
            vec!["1.0.0.0/8", "1.0.0.0/16"].into_iter().collect::<PrefixSet<_>>()
                ^ vec!["1.0.0.0/16"].into(),
            vec!["1.0.0.0/8"].into()
        };
        xor_overlapping_set_with_sibling {
            vec!["1.0.0.0/8", "2.0.0.0/8"].into_iter().collect::<PrefixSet<_>>()
                ^ vec!["1.0.0.0/8"].into(),
            vec!["2.0.0.0/8"].into()
        };
        xor_overlapping_set_with_child {
            vec!["1.0.0.0/8", "1.0.0.0/16"].into_iter().collect::<PrefixSet<_>>()
                ^ vec!["1.0.0.0/8"].into(),
            vec!["1.0.0.0/16"].into()
        };
        xor_covering_parent {
            vec!["1.0.0.0/16"].into_iter().collect::<PrefixSet<_>>()
                ^ vec!["1.0.0.0/8,16,16"].into(),
            vec![
                "1.1.0.0/16",
                "1.2.0.0/15,16,16",
                "1.4.0.0/14,16,16",
                "1.8.0.0/13,16,16",
                "1.16.0.0/12,16,16",
                "1.32.0.0/11,16,16",
                "1.64.0.0/10,16,16",
                "1.128.0.0/9,16,16",
            ].into()
        };
        xor_covered_child {
            vec!["1.0.0.0/8,16,16"].into_iter().collect::<PrefixSet<_>>()
                ^ vec!["1.0.0.0/16"].into(),
            vec![
                "1.1.0.0/16",
                "1.2.0.0/15,16,16",
                "1.4.0.0/14,16,16",
                "1.8.0.0/13,16,16",
                "1.16.0.0/12,16,16",
                "1.32.0.0/11,16,16",
                "1.64.0.0/10,16,16",
                "1.128.0.0/9,16,16",
            ].into()
        };
        xor_overlapping_set_with_covered_child {
            vec!["1.0.0.0/8", "1.0.0.0/16"].into_iter().collect::<PrefixSet<_>>()
                ^ vec!["1.0.0.0/8,16,16"].into(),
            vec!["1.0.0.0/8"].into()
        };
        sub_disjoint_nodes {
            vec!["2.0.0.0/8,8,16"].into_iter().collect::<PrefixSet<_>>()
                - vec!["3.0.0.0/8,8,16"].into(),
            vec!["2.0.0.0/8,8,16"].into()
        };
        sub_disjoint_ranges {
            vec!["1.0.0.0/8,8,11"].into_iter().collect::<PrefixSet<_>>()
                - vec!["1.0.0.0/8,12,15"].into(),
            vec!["1.0.0.0/8,8,11"].into()
        };
        sub_overlapping_nodes {
            vec!["1.0.0.0/8,12,16"].into_iter().collect::<PrefixSet<_>>()
                - vec!["1.0.0.0/12,12,16"].into(),
            vec![
                "1.16.0.0/12,12,16",
                "1.32.0.0/11,12,16",
                "1.64.0.0/10,12,16",
                "1.128.0.0/9,12,16"
            ].into()
        };
        sub_overlapping_ranges {
            vec!["1.0.0.0/8,8,12"].into_iter().collect::<PrefixSet<_>>()
                - vec!["1.0.0.0/8,12,16"].into(),
            vec!["1.0.0.0/8,8,11"].into()
        };
        sub_overlapping_set_with_parent {
            vec!["1.0.0.0/8", "1.0.0.0/16"].into_iter().collect::<PrefixSet<_>>()
                - vec!["1.0.0.0/16"].into(),
            vec!["1.0.0.0/8"].into()
        };
        sub_overlapping_set_with_sibling {
            vec!["1.0.0.0/8", "2.0.0.0/8"].into_iter().collect::<PrefixSet<_>>()
                - vec!["1.0.0.0/8"].into(),
            vec!["2.0.0.0/8"].into()
        };
        sub_overlapping_set_with_child {
            vec!["1.0.0.0/8", "1.0.0.0/16"].into_iter().collect::<PrefixSet<_>>()
                - vec!["1.0.0.0/8"].into(),
            vec!["1.0.0.0/16"].into()
        };
        sub_covering_parent {
            vec!["1.0.0.0/16"].into_iter().collect::<PrefixSet<_>>()
                - vec!["1.0.0.0/8,16,16"].into(),
            PrefixSet::zero()
        };
        sub_covered_child {
            vec!["1.0.0.0/8,16,16"].into_iter().collect::<PrefixSet<_>>()
                - vec!["1.0.0.0/16"].into(),
            vec![
                "1.1.0.0/16",
                "1.2.0.0/15,16,16",
                "1.4.0.0/14,16,16",
                "1.8.0.0/13,16,16",
                "1.16.0.0/12,16,16",
                "1.32.0.0/11,16,16",
                "1.64.0.0/10,16,16",
                "1.128.0.0/9,16,16",
            ].into()
        };
        sub_overlapping_set_with_covered_child {
            vec!["1.0.0.0/8", "1.0.0.0/16"].into_iter().collect::<PrefixSet<_>>()
                - vec!["1.0.0.0/8,16,16"].into(),
            vec![
                "1.0.0.0/8",
                "1.1.0.0/16",
                "1.2.0.0/15,16,16",
                "1.4.0.0/14,16,16",
                "1.8.0.0/13,16,16",
                "1.16.0.0/12,16,16",
                "1.32.0.0/11,16,16",
                "1.64.0.0/10,16,16",
                "1.128.0.0/9,16,16",
            ].into()
        };
        sub_complex_deaggregation {
            vec!["2.0.0.0/8,8,10", "3.0.0.0/8,8,9"].into_iter().collect::<PrefixSet<_>>()
                - vec!["2.0.0.0/10", "3.0.0.0/8,8,10"].into(),
            vec![
                "2.0.0.0/8,8,9",
                "2.64.0.0/10",
                "2.128.0.0/10",
                "2.192.0.0/10",
            ].into()
        };
        not_singleton {
            ! vec!["1.0.0.0/8"].into_iter().collect::<PrefixSet<_>>(),
            vec![
                "0.0.0.0/0,0,7",
                "0.0.0.0/0,9,32",
                "0.0.0.0/8",
                "2.0.0.0/7,8,8",
                "4.0.0.0/6,8,8",
                "8.0.0.0/5,8,8",
                "16.0.0.0/4,8,8",
                "32.0.0.0/3,8,8",
                "64.0.0.0/2,8,8",
                "128.0.0.0/1,8,8"
            ].into()
        };
        not_range {
            ! vec!["1.0.0.0/8,8,16"].into_iter().collect::<PrefixSet<_>>(),
            vec![
                "0.0.0.0/0,0,7",
                "0.0.0.0/0,17,32",
                "0.0.0.0/8,8,16",
                "2.0.0.0/7,8,16",
                "4.0.0.0/6,8,16",
                "8.0.0.0/5,8,16",
                "16.0.0.0/4,8,16",
                "32.0.0.0/3,8,16",
                "64.0.0.0/2,8,16",
                "128.0.0.0/1,8,16",
            ].into()
        }
    });

    #[test]
    fn dups() -> TestResult {
        let s: PrefixSet<Ipv4Prefix> = vec!["27.0.0.0/22,22,24", "27.0.4.0/22,22,23"].into();
        dbg!(&s);
        assert_eq!(s.iter_prefixes().count(), 10);
        let t: PrefixSet<Ipv4Prefix> = vec!["27.0.0.0/24,24,24", "27.0.4.0/22,22,24"].into();
        dbg!(&t);
        assert_eq!(t.iter_prefixes().count(), 8);
        let st = s.clone() - t.clone();
        dbg!(&st);
        assert_eq!(st.iter_prefixes().count(), 6);
        t.iter_prefixes()
            .for_each(|p| assert!(!st.contains(p.to_owned())));
        // let ts = t.clone() | s.clone();
        // dbg!(&ts);
        // assert_eq!(ts.iter_prefixes().count(), 129);
        Ok(())
    }
}
