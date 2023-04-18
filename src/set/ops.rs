use std::cmp::{Ordering, PartialEq, PartialOrd};
use std::ops::{Add, BitAnd, BitOr, BitXor, Mul, Not, Sub};

use ip::Afi;

use num::{One, Zero};

use super::PrefixSet;

impl<A: Afi> Zero for PrefixSet<A> {
    fn zero() -> Self {
        Self::new()
    }

    fn is_zero(&self) -> bool {
        self.root.is_some()
    }
}

impl<A: Afi> One for PrefixSet<A> {
    fn one() -> Self {
        Self::new()
            .insert(<A as ip::AfiClass>::PrefixRange::ALL)
            .to_owned()
    }
}

impl<A: Afi> BitAnd for PrefixSet<A> {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        match (self.root, rhs.root) {
            (Some(r), Some(s)) => Self::Output::new_with_root(r & s).aggregate().to_owned(),
            _ => PrefixSet::zero(),
        }
    }
}

impl<A: Afi> BitOr for PrefixSet<A> {
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

impl<A: Afi> BitXor for PrefixSet<A> {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        (self.to_owned() | rhs.to_owned()) - (self & rhs)
    }
}

impl<A: Afi> Not for PrefixSet<A> {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self::Output::one() - self
    }
}

impl<A: Afi> Add for PrefixSet<A> {
    type Output = Self;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn add(self, rhs: Self) -> Self::Output {
        self | rhs
    }
}

impl<A: Afi> Sub for PrefixSet<A> {
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

impl<A: Afi> Mul for PrefixSet<A> {
    type Output = Self;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn mul(self, rhs: Self) -> Self::Output {
        self & rhs
    }
}

impl<A: Afi> PartialEq for PrefixSet<A> {
    fn eq(&self, other: &Self) -> bool {
        match (&self.root, &other.root) {
            (Some(r), Some(s)) => r.children().zip(s.children()).all(|(m, n)| m == n),
            (None, None) => true,
            _ => false,
        }
    }
}

impl<A: Afi> PartialOrd for PrefixSet<A> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self == other {
            Some(Ordering::Equal)
        } else if self.prefixes().all(|p| other.contains(p)) {
            Some(Ordering::Less)
        } else if other.prefixes().all(|p| self.contains(p)) {
            Some(Ordering::Greater)
        } else {
            None
        }
    }
}

impl<A: Afi> Eq for PrefixSet<A> {}

#[cfg(test)]
mod tests {

    use ip::{concrete::PrefixRange, Ipv4, Ipv6};

    use paste::paste;

    use crate::tests::{TestError, TestResult};

    use super::*;

    impl<A: Afi> FromIterator<&'static str> for PrefixSet<A> {
        fn from_iter<T: IntoIterator<Item = &'static str>>(iter: T) -> Self {
            iter.into_iter()
                .map(|s| {
                    let mut elems = s.split(',');
                    let prefix = elems
                        .next()
                        .ok_or(TestError::from("no prefix"))
                        .and_then(|s| s.parse().map_err(TestError::from))?;
                    match (elems.next(), elems.next()) {
                        (Some(l), Some(u)) => {
                            let lower = l.parse::<usize>()?.try_into()?;
                            let upper = u.parse::<usize>()?.try_into()?;
                            PrefixRange::new(prefix, lower..=upper).map_err(TestError::from)
                        }
                        (None, None) => Ok(prefix.into()),
                        _ => Err(TestError::from("failed to parse range")),
                    }
                })
                .map(Result::unwrap)
                .collect()
        }
    }

    macro_rules! test_exprs {
        ( $($fn_id:ident {$lhs:expr, $rhs:expr});* ) => {
            test_exprs!(@ipv4 {$($fn_id {$lhs, $rhs});*});
            test_exprs!(@ipv6 {$($fn_id {$lhs, $rhs});*});
        };
        ( @ipv4 {$($fn_id:ident {$lhs:expr, $rhs:expr});*} ) => {
            paste! {
                test_exprs!($(Ipv4 => [<ipv4_ $fn_id>] {$lhs, $rhs});*);
            }
        };
        ( @ipv6 {$($fn_id:ident {$lhs:expr, $rhs:expr});*} ) => {
            paste! {
                test_exprs!($(Ipv6 => [<ipv6_ $fn_id>] {$lhs, $rhs});*);
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
        assert_eq!(PrefixSet::<Ipv4>::zero().prefixes().count(), 0);
        Ok(())
    }

    #[test]
    fn ipv6_zero_set_is_empty() -> TestResult {
        assert_eq!(PrefixSet::<Ipv6>::zero().prefixes().count(), 0);
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
                & vec!["2.0.0.0/8,8,16"].into_iter().collect(),
            PrefixSet::zero()
        };
        intersect_disjoint_ranges {
            vec!["1.0.0.0/8,8,11"].into_iter().collect::<PrefixSet<_>>()
                & vec!["1.0.0.0/8,12,15"].into_iter().collect(),
            PrefixSet::zero()
        };
        intersect_overlapping_nodes {
            vec!["1.0.0.0/8,12,16"].into_iter().collect::<PrefixSet<_>>()
                & vec!["1.0.0.0/12,12,16"].into_iter().collect(),
            vec!["1.0.0.0/12,12,16"].into_iter().collect()
        };
        intersect_overlapping_ranges {
            vec!["1.0.0.0/8,8,12"].into_iter().collect::<PrefixSet<_>>()
                & vec!["1.0.0.0/8,12,16"].into_iter().collect(),
            vec!["1.0.0.0/8,12,12"].into_iter().collect()
        };
        intersect_overlapping_set_with_parent {
            vec!["1.0.0.0/8", "1.0.0.0/16"].into_iter().collect::<PrefixSet<_>>()
                & vec!["1.0.0.0/16"].into_iter().collect(),
            vec!["1.0.0.0/16"].into_iter().collect()
        };
        intersect_overlapping_set_with_sibling {
            vec!["1.0.0.0/8", "2.0.0.0/8"].into_iter().collect::<PrefixSet<_>>()
                & vec!["1.0.0.0/8"].into_iter().collect(),
            vec!["1.0.0.0/8"].into_iter().collect()
        };
        intersect_overlapping_set_with_child {
            vec!["1.0.0.0/8", "1.0.0.0/16"].into_iter().collect::<PrefixSet<_>>()
                & vec!["1.0.0.0/8"].into_iter().collect(),
            vec!["1.0.0.0/8"].into_iter().collect()
        };
        intersect_covering_parent {
            vec!["1.0.0.0/16"].into_iter().collect::<PrefixSet<_>>()
                & vec!["1.0.0.0/8,16,16"].into_iter().collect(),
            vec!["1.0.0.0/16"].into_iter().collect()
        };
        intersect_covered_child {
            vec!["1.0.0.0/8,16,16"].into_iter().collect::<PrefixSet<_>>()
                & vec!["1.0.0.0/16"].into_iter().collect(),
            vec!["1.0.0.0/16"].into_iter().collect()
        };
        intersect_overlapping_set_with_covered_child {
            vec!["1.0.0.0/8", "1.0.0.0/16"].into_iter().collect::<PrefixSet<_>>()
                & vec!["1.0.0.0/8,16,16"].into_iter().collect(),
            vec!["1.0.0.0/16"].into_iter().collect()
        };
        union_disjoint_nodes {
            vec!["2.0.0.0/8,8,16"].into_iter().collect::<PrefixSet<_>>()
                | vec!["3.0.0.0/8,8,16"].into_iter().collect(),
            vec!["2.0.0.0/7,8,16"].into_iter().collect()
        };
        union_disjoint_ranges {
            vec!["1.0.0.0/8,8,11"].into_iter().collect::<PrefixSet<_>>()
                | vec!["1.0.0.0/8,12,15"].into_iter().collect(),
            vec!["1.0.0.0/8,8,15"].into_iter().collect()
        };
        union_overlapping_nodes {
            vec!["1.0.0.0/8,12,16"].into_iter().collect::<PrefixSet<_>>()
                | vec!["1.0.0.0/12,12,16"].into_iter().collect(),
            vec!["1.0.0.0/8,12,16"].into_iter().collect()
        };
        union_overlapping_ranges {
            vec!["1.0.0.0/8,8,12"].into_iter().collect::<PrefixSet<_>>()
                | vec!["1.0.0.0/8,12,16"].into_iter().collect(),
            vec!["1.0.0.0/8,8,16"].into_iter().collect()
        };
        union_overlapping_set_with_parent {
            vec!["1.0.0.0/8", "1.0.0.0/16"].into_iter().collect::<PrefixSet<_>>()
                | vec!["1.0.0.0/16"].into_iter().collect(),
            vec!["1.0.0.0/8", "1.0.0.0/16"].into_iter().collect()
        };
        union_overlapping_set_with_sibling {
            vec!["1.0.0.0/8", "2.0.0.0/8"].into_iter().collect::<PrefixSet<_>>()
                | vec!["1.0.0.0/8"].into_iter().collect(),
            vec!["1.0.0.0/8", "2.0.0.0/8"].into_iter().collect()
        };
        union_overlapping_set_with_child {
            vec!["1.0.0.0/8", "1.0.0.0/16"].into_iter().collect::<PrefixSet<_>>()
                | vec!["1.0.0.0/8"].into_iter().collect(),
            vec!["1.0.0.0/8", "1.0.0.0/16"].into_iter().collect()
        };
        union_covering_parent {
            vec!["1.0.0.0/16"].into_iter().collect::<PrefixSet<_>>()
                | vec!["1.0.0.0/8,16,16"].into_iter().collect(),
            vec!["1.0.0.0/8,16,16"].into_iter().collect()
        };
        union_covered_child {
            vec!["1.0.0.0/8,16,16"].into_iter().collect::<PrefixSet<_>>()
                | vec!["1.0.0.0/16"].into_iter().collect(),
            vec!["1.0.0.0/8,16,16"].into_iter().collect()
        };
        union_overlapping_set_with_covered_child {
            vec!["1.0.0.0/8", "1.0.0.0/16"].into_iter().collect::<PrefixSet<_>>()
                | vec!["1.0.0.0/8,16,16"].into_iter().collect(),
            vec!["1.0.0.0/8", "1.0.0.0/8,16,16"].into_iter().collect()
        };
        xor_disjoint_nodes {
            vec!["2.0.0.0/8,8,16"].into_iter().collect::<PrefixSet<_>>()
                ^ vec!["3.0.0.0/8,8,16"].into_iter().collect(),
            vec!["2.0.0.0/7,8,16"].into_iter().collect()
        };
        xor_disjoint_ranges {
            vec!["1.0.0.0/8,8,11"].into_iter().collect::<PrefixSet<_>>()
                ^ vec!["1.0.0.0/8,12,15"].into_iter().collect(),
            vec!["1.0.0.0/8,8,15"].into_iter().collect()
        };
        xor_overlapping_nodes {
            vec!["1.0.0.0/8,12,16"].into_iter().collect::<PrefixSet<_>>()
                ^ vec!["1.0.0.0/12,12,16"].into_iter().collect(),
            vec![
                "1.16.0.0/12,12,16",
                "1.32.0.0/11,12,16",
                "1.64.0.0/10,12,16",
                "1.128.0.0/9,12,16"
            ].into_iter().collect()
        };
        xor_overlapping_ranges {
            vec!["1.0.0.0/8,8,12"].into_iter().collect::<PrefixSet<_>>()
                ^ vec!["1.0.0.0/8,12,16"].into_iter().collect(),
            vec!["1.0.0.0/8,8,11", "1.0.0.0/8,13,16"].into_iter().collect()
        };
        xor_overlapping_set_with_parent {
            vec!["1.0.0.0/8", "1.0.0.0/16"].into_iter().collect::<PrefixSet<_>>()
                ^ vec!["1.0.0.0/16"].into_iter().collect(),
            vec!["1.0.0.0/8"].into_iter().collect()
        };
        xor_overlapping_set_with_sibling {
            vec!["1.0.0.0/8", "2.0.0.0/8"].into_iter().collect::<PrefixSet<_>>()
                ^ vec!["1.0.0.0/8"].into_iter().collect(),
            vec!["2.0.0.0/8"].into_iter().collect()
        };
        xor_overlapping_set_with_child {
            vec!["1.0.0.0/8", "1.0.0.0/16"].into_iter().collect::<PrefixSet<_>>()
                ^ vec!["1.0.0.0/8"].into_iter().collect(),
            vec!["1.0.0.0/16"].into_iter().collect()
        };
        xor_covering_parent {
            vec!["1.0.0.0/16"].into_iter().collect::<PrefixSet<_>>()
                ^ vec!["1.0.0.0/8,16,16"].into_iter().collect(),
            vec![
                "1.1.0.0/16",
                "1.2.0.0/15,16,16",
                "1.4.0.0/14,16,16",
                "1.8.0.0/13,16,16",
                "1.16.0.0/12,16,16",
                "1.32.0.0/11,16,16",
                "1.64.0.0/10,16,16",
                "1.128.0.0/9,16,16",
            ].into_iter().collect()
        };
        xor_covered_child {
            vec!["1.0.0.0/8,16,16"].into_iter().collect::<PrefixSet<_>>()
                ^ vec!["1.0.0.0/16"].into_iter().collect(),
            vec![
                "1.1.0.0/16",
                "1.2.0.0/15,16,16",
                "1.4.0.0/14,16,16",
                "1.8.0.0/13,16,16",
                "1.16.0.0/12,16,16",
                "1.32.0.0/11,16,16",
                "1.64.0.0/10,16,16",
                "1.128.0.0/9,16,16",
            ].into_iter().collect()
        };
        xor_overlapping_set_with_covered_child {
            vec!["1.0.0.0/8", "1.0.0.0/16"].into_iter().collect::<PrefixSet<_>>()
                ^ vec!["1.0.0.0/8,16,16"].into_iter().collect(),
            vec!["1.0.0.0/8"].into_iter().collect()
        };
        sub_disjoint_nodes {
            vec!["2.0.0.0/8,8,16"].into_iter().collect::<PrefixSet<_>>()
                - vec!["3.0.0.0/8,8,16"].into_iter().collect(),
            vec!["2.0.0.0/8,8,16"].into_iter().collect()
        };
        sub_disjoint_ranges {
            vec!["1.0.0.0/8,8,11"].into_iter().collect::<PrefixSet<_>>()
                - vec!["1.0.0.0/8,12,15"].into_iter().collect(),
            vec!["1.0.0.0/8,8,11"].into_iter().collect()
        };
        sub_overlapping_nodes {
            vec!["1.0.0.0/8,12,16"].into_iter().collect::<PrefixSet<_>>()
                - vec!["1.0.0.0/12,12,16"].into_iter().collect(),
            vec![
                "1.16.0.0/12,12,16",
                "1.32.0.0/11,12,16",
                "1.64.0.0/10,12,16",
                "1.128.0.0/9,12,16"
            ].into_iter().collect()
        };
        sub_overlapping_ranges {
            vec!["1.0.0.0/8,8,12"].into_iter().collect::<PrefixSet<_>>()
                - vec!["1.0.0.0/8,12,16"].into_iter().collect(),
            vec!["1.0.0.0/8,8,11"].into_iter().collect()
        };
        sub_overlapping_set_with_parent {
            vec!["1.0.0.0/8", "1.0.0.0/16"].into_iter().collect::<PrefixSet<_>>()
                - vec!["1.0.0.0/16"].into_iter().collect(),
            vec!["1.0.0.0/8"].into_iter().collect()
        };
        sub_overlapping_set_with_sibling {
            vec!["1.0.0.0/8", "2.0.0.0/8"].into_iter().collect::<PrefixSet<_>>()
                - vec!["1.0.0.0/8"].into_iter().collect(),
            vec!["2.0.0.0/8"].into_iter().collect()
        };
        sub_overlapping_set_with_child {
            vec!["1.0.0.0/8", "1.0.0.0/16"].into_iter().collect::<PrefixSet<_>>()
                - vec!["1.0.0.0/8"].into_iter().collect(),
            vec!["1.0.0.0/16"].into_iter().collect()
        };
        sub_covering_parent {
            vec!["1.0.0.0/16"].into_iter().collect::<PrefixSet<_>>()
                - vec!["1.0.0.0/8,16,16"].into_iter().collect(),
            PrefixSet::zero()
        };
        sub_covered_child {
            vec!["1.0.0.0/8,16,16"].into_iter().collect::<PrefixSet<_>>()
                - vec!["1.0.0.0/16"].into_iter().collect(),
            vec![
                "1.1.0.0/16",
                "1.2.0.0/15,16,16",
                "1.4.0.0/14,16,16",
                "1.8.0.0/13,16,16",
                "1.16.0.0/12,16,16",
                "1.32.0.0/11,16,16",
                "1.64.0.0/10,16,16",
                "1.128.0.0/9,16,16",
            ].into_iter().collect()
        };
        sub_overlapping_set_with_covered_child {
            vec!["1.0.0.0/8", "1.0.0.0/16"].into_iter().collect::<PrefixSet<_>>()
                - vec!["1.0.0.0/8,16,16"].into_iter().collect(),
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
            ].into_iter().collect()
        };
        sub_complex_deaggregation {
            vec!["2.0.0.0/8,8,10", "3.0.0.0/8,8,9"].into_iter().collect::<PrefixSet<_>>()
                - vec!["2.0.0.0/10", "3.0.0.0/8,8,10"].into_iter().collect(),
            vec![
                "2.0.0.0/8,8,9",
                "2.64.0.0/10",
                "2.128.0.0/10",
                "2.192.0.0/10",
            ].into_iter().collect()
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
            ].into_iter().collect()
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
            ].into_iter().collect()
        }
    });
}
