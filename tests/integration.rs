extern crate utils;
use utils::data_set;

use ip::{traits::Prefix as _, Ipv4, Ipv6, Prefix, PrefixRange};

use std::collections::HashSet;

use num::Zero;

use prefixset::PrefixSet;

mod ipv4 {
    use super::*;

    #[test]
    fn set_from_prefixes_contains_all_prefixes() {
        let prefixes: Vec<Prefix<Ipv4>> = data_set("AS-WOLCOMM-ipv4-prefixes", 0, 0).read();
        let set: PrefixSet<_> = prefixes.iter().copied().collect();
        let mut i = 0;
        prefixes.iter().for_each(|prefix| {
            assert!(set.contains(*prefix));
            i += 1;
        });
        println!("prefix count: {}", i);
        println!("prefix iter count: {}", set.prefixes().count());
        println!("range iter count: {}", set.ranges().count());
    }

    #[test]
    fn set_from_prefixes_is_same_as_source() {
        let prefixes: Vec<Prefix<Ipv4>> = data_set("AS-WOLCOMM-ipv4-prefixes", 0, 0).read();
        let set: PrefixSet<_> = prefixes.iter().copied().collect();
        let prefixes_pre: HashSet<_> = prefixes.into_iter().collect();
        let prefixes_post: HashSet<_> = set.prefixes().collect();
        let mut difference: Vec<_> = (&prefixes_pre ^ &prefixes_post).into_iter().collect();
        difference.sort_by_key(|p| (p.network(), p.length()));
        println!("{:#?}", difference);
        assert!(difference.is_empty());
    }

    #[test]
    fn aggregated_and_non_aggregated_versions_eq() {
        let prefixes: Vec<Prefix<Ipv4>> = data_set("AS-WOLCOMM-ipv4-prefixes", 0, 0).read();
        let ranges: Vec<PrefixRange<Ipv4>> = data_set("AS-WOLCOMM-ipv4-ranges", 0, 0).read();
        let s: PrefixSet<_> = prefixes.into_iter().collect();
        let t: PrefixSet<_> = ranges.into_iter().collect();
        assert_eq!(s, t);
    }

    #[test]
    fn aggregated_and_non_aggregated_versions_intersection_eq_union() {
        let prefixes: Vec<Prefix<Ipv4>> = data_set("AS-WOLCOMM-ipv4-prefixes", 0, 0).read();
        let ranges: Vec<PrefixRange<Ipv4>> = data_set("AS-WOLCOMM-ipv4-ranges", 0, 0).read();
        let s: PrefixSet<_> = prefixes.into_iter().collect();
        let t: PrefixSet<_> = ranges.into_iter().collect();
        assert_eq!(s.clone() & t.clone(), s | t);
    }

    #[test]
    fn aggregated_and_non_aggregated_versions_difference_is_empty() {
        let prefixes: Vec<Prefix<Ipv4>> = data_set("AS-WOLCOMM-ipv4-prefixes", 0, 0).read();
        let ranges: Vec<PrefixRange<Ipv4>> = data_set("AS-WOLCOMM-ipv4-ranges", 0, 0).read();
        let s: PrefixSet<_> = prefixes.into_iter().collect();
        let t: PrefixSet<_> = ranges.into_iter().collect();
        assert_eq!(s ^ t, PrefixSet::zero());
    }

    #[test]
    fn intersection_of_sets_from_prefixes_has_expected_size() {
        let s: PrefixSet<_> = data_set::<Prefix<Ipv4>>("AS-WOLCOMM-ipv4-prefixes", 0, 0)
            .read()
            .into_iter()
            .collect();
        let t: PrefixSet<_> = data_set::<Prefix<Ipv4>>("AS-HURRICANE-ipv4-prefixes", 0, 0)
            .read()
            .into_iter()
            .collect();
        let intersection = s & t;
        assert_eq!(intersection.prefixes().count(), 407473)
    }

    #[test]
    fn intersection_of_sets_from_ranges_has_expected_size() {
        let s: PrefixSet<_> = data_set::<PrefixRange<Ipv4>>("AS-WOLCOMM-ipv4-ranges", 0, 0)
            .read()
            .into_iter()
            .collect();
        let t: PrefixSet<_> = data_set::<PrefixRange<Ipv4>>("AS-HURRICANE-ipv4-ranges", 0, 0)
            .read()
            .into_iter()
            .collect();
        let intersection = s & t;
        assert_eq!(intersection.prefixes().count(), 407473)
    }

    #[test]
    fn union_of_sets_from_prefixes_has_expected_size() {
        let s: PrefixSet<_> = data_set::<Prefix<Ipv4>>("AS-WOLCOMM-ipv4-prefixes", 0, 0)
            .read()
            .into_iter()
            .collect();
        let t: PrefixSet<_> = data_set::<Prefix<Ipv4>>("AS-HURRICANE-ipv4-prefixes", 0, 0)
            .read()
            .into_iter()
            .collect();
        let union = s | t;
        assert_eq!(union.prefixes().count(), 1165336)
    }

    #[test]
    fn union_of_sets_from_ranges_has_expected_size() {
        let s: PrefixSet<_> = data_set::<PrefixRange<Ipv4>>("AS-WOLCOMM-ipv4-ranges", 0, 0)
            .read()
            .into_iter()
            .collect();
        let t: PrefixSet<_> = data_set::<PrefixRange<Ipv4>>("AS-HURRICANE-ipv4-ranges", 0, 0)
            .read()
            .into_iter()
            .collect();
        let union = s | t;
        assert_eq!(union.prefixes().count(), 1165336)
    }

    #[test]
    fn xor_of_sets_from_prefixes_has_expected_size() {
        let s: PrefixSet<_> = data_set::<Prefix<Ipv4>>("AS-WOLCOMM-ipv4-prefixes", 0, 0)
            .read()
            .into_iter()
            .collect();
        let t: PrefixSet<_> = data_set::<Prefix<Ipv4>>("AS-HURRICANE-ipv4-prefixes", 0, 0)
            .read()
            .into_iter()
            .collect();
        let xor = s ^ t;
        assert_eq!(xor.prefixes().count(), 757863)
    }
    #[test]
    fn xor_of_sets_from_ranges_has_expected_size() {
        let s: PrefixSet<_> = data_set::<PrefixRange<Ipv4>>("AS-WOLCOMM-ipv4-ranges", 0, 0)
            .read()
            .into_iter()
            .collect();
        let t: PrefixSet<_> = data_set::<PrefixRange<Ipv4>>("AS-HURRICANE-ipv4-ranges", 0, 0)
            .read()
            .into_iter()
            .collect();
        let xor = s ^ t;
        assert_eq!(xor.prefixes().count(), 757863)
    }

    #[test]
    fn diff_of_sets_from_prefixes_do_not_contain_removed_prefixes() {
        let s: PrefixSet<_> = data_set::<Prefix<Ipv4>>("AS-WOLCOMM-ipv4-prefixes", 0, 0)
            .read()
            .into_iter()
            .collect();
        let t: PrefixSet<_> = data_set::<Prefix<Ipv4>>("AS-HURRICANE-ipv4-prefixes", 0, 0)
            .read()
            .into_iter()
            .collect();
        let diff = s.clone() - t.clone();
        let err = t.prefixes().filter(|p| diff.contains(*p)).count();
        assert_eq!(err, 0)
    }

    #[test]
    fn diff_of_sets_from_ranges_do_not_contain_removed_prefixes() {
        let s: PrefixSet<_> = data_set::<PrefixRange<Ipv4>>("AS-WOLCOMM-ipv4-ranges", 0, 0)
            .read()
            .into_iter()
            .collect();
        let t: PrefixSet<_> = data_set::<PrefixRange<Ipv4>>("AS-HURRICANE-ipv4-ranges", 0, 0)
            .read()
            .into_iter()
            .collect();
        let diff = s.clone() - t.clone();
        let err = t.prefixes().filter(|p| diff.contains(*p)).count();
        assert_eq!(err, 0)
    }

    // #[test]
    // fn neg_of_set_from_prefixes_has_expected_size() {
    //     let s: PrefixSet<_> = data_set::<Ipv4Prefix>("AS-WOLCOMM-ipv4-prefixes", 0, 0)
    //         .read()
    //         .into_iter()
    //         .collect();
    //     let not = !s;
    //     assert_eq!(not.iter_prefixes().count(), 8_589_176_728)
    // }
}

mod ipv6 {
    use super::*;

    #[test]
    fn set_from_prefixes_contains_all_prefixes() {
        let prefixes: Vec<Prefix<Ipv6>> = data_set("AS-WOLCOMM-ipv6-prefixes", 0, 0).read();
        let set: PrefixSet<Ipv6> = prefixes.iter().copied().collect();
        let mut i = 0;
        prefixes.iter().for_each(|prefix| {
            assert!(set.contains(*prefix));
            i += 1;
        });
        println!("prefix count: {}", i);
        println!("prefix iter count: {}", set.prefixes().count());
        println!("range iter count: {}", set.ranges().count());
    }

    #[test]
    fn set_from_prefixes_is_same_as_source() {
        let prefixes: Vec<Prefix<Ipv6>> = data_set("AS-WOLCOMM-ipv6-prefixes", 0, 0).read();
        let set: PrefixSet<_> = prefixes.iter().copied().collect();
        let prefixes_pre: HashSet<_> = prefixes.into_iter().collect();
        let prefixes_post: HashSet<_> = set.prefixes().collect();
        let mut difference: Vec<_> = (&prefixes_pre ^ &prefixes_post).into_iter().collect();
        difference.sort_by_key(|p| (p.network(), p.length()));
        println!("{:#?}", difference);
        assert!(difference.is_empty());
    }

    #[test]
    fn aggregated_and_non_aggregated_versions_eq() {
        let prefixes: Vec<Prefix<Ipv6>> = data_set("AS-WOLCOMM-ipv6-prefixes", 0, 0).read();
        let ranges: Vec<PrefixRange<Ipv6>> = data_set("AS-WOLCOMM-ipv6-ranges", 0, 0).read();
        let s: PrefixSet<_> = prefixes.into_iter().collect();
        let t: PrefixSet<_> = ranges.into_iter().collect();
        assert_eq!(s, t);
    }

    #[test]
    fn aggregated_and_non_aggregated_versions_intersection_eq_union() {
        let prefixes: Vec<Prefix<Ipv6>> = data_set("AS-WOLCOMM-ipv6-prefixes", 0, 0).read();
        let ranges: Vec<PrefixRange<Ipv6>> = data_set("AS-WOLCOMM-ipv6-ranges", 0, 0).read();
        let s: PrefixSet<_> = prefixes.into_iter().collect();
        let t: PrefixSet<_> = ranges.into_iter().collect();
        assert_eq!(s.clone() & t.clone(), s | t);
    }

    #[test]
    fn aggregated_and_non_aggregated_versions_difference_is_empty() {
        let prefixes: Vec<Prefix<Ipv6>> = data_set("AS-WOLCOMM-ipv6-prefixes", 0, 0).read();
        let ranges: Vec<PrefixRange<Ipv6>> = data_set("AS-WOLCOMM-ipv6-ranges", 0, 0).read();
        let s: PrefixSet<_> = prefixes.into_iter().collect();
        let t: PrefixSet<_> = ranges.into_iter().collect();
        assert_eq!(s ^ t, PrefixSet::zero());
    }

    #[test]
    fn intersection_of_sets_from_prefixes_has_expected_size() {
        let s: PrefixSet<_> = data_set::<Prefix<Ipv6>>("AS-WOLCOMM-ipv6-prefixes", 0, 0)
            .read()
            .into_iter()
            .collect();
        let t: PrefixSet<_> = data_set::<Prefix<Ipv6>>("AS-HURRICANE-ipv6-prefixes", 0, 0)
            .read()
            .into_iter()
            .collect();
        let intersection = s & t;
        assert_eq!(intersection.prefixes().count(), 146252)
    }

    #[test]
    fn intersection_of_sets_from_ranges_has_expected_size() {
        let s: PrefixSet<_> = data_set::<PrefixRange<Ipv6>>("AS-WOLCOMM-ipv6-ranges", 0, 0)
            .read()
            .into_iter()
            .collect();
        let t: PrefixSet<_> = data_set::<PrefixRange<Ipv6>>("AS-HURRICANE-ipv6-ranges", 0, 0)
            .read()
            .into_iter()
            .collect();
        let intersection = s & t;
        assert_eq!(intersection.prefixes().count(), 146252)
    }

    #[test]
    fn union_of_sets_from_prefixes_has_expected_size() {
        let s: PrefixSet<_> = data_set::<Prefix<Ipv6>>("AS-WOLCOMM-ipv6-prefixes", 0, 0)
            .read()
            .into_iter()
            .collect();
        let t: PrefixSet<_> = data_set::<Prefix<Ipv6>>("AS-HURRICANE-ipv6-prefixes", 0, 0)
            .read()
            .into_iter()
            .collect();
        let union = s | t;
        assert_eq!(union.prefixes().count(), 347267)
    }

    #[test]
    fn union_of_sets_from_ranges_has_expected_size() {
        let s: PrefixSet<_> = data_set::<PrefixRange<Ipv6>>("AS-WOLCOMM-ipv6-ranges", 0, 0)
            .read()
            .into_iter()
            .collect();
        let t: PrefixSet<_> = data_set::<PrefixRange<Ipv6>>("AS-HURRICANE-ipv6-ranges", 0, 0)
            .read()
            .into_iter()
            .collect();
        let union = s | t;
        assert_eq!(union.prefixes().count(), 347267)
    }

    #[test]
    fn xor_of_sets_from_prefixes_has_expected_size() {
        let s: PrefixSet<_> = data_set::<Prefix<Ipv6>>("AS-WOLCOMM-ipv6-prefixes", 0, 0)
            .read()
            .into_iter()
            .collect();
        let t: PrefixSet<_> = data_set::<Prefix<Ipv6>>("AS-HURRICANE-ipv6-prefixes", 0, 0)
            .read()
            .into_iter()
            .collect();
        let xor = s ^ t;
        assert_eq!(xor.prefixes().count(), 201015)
    }
    #[test]
    fn xor_of_sets_from_ranges_has_expected_size() {
        let s: PrefixSet<_> = data_set::<PrefixRange<Ipv6>>("AS-WOLCOMM-ipv6-ranges", 0, 0)
            .read()
            .into_iter()
            .collect();
        let t: PrefixSet<_> = data_set::<PrefixRange<Ipv6>>("AS-HURRICANE-ipv6-ranges", 0, 0)
            .read()
            .into_iter()
            .collect();
        let xor = s ^ t;
        assert_eq!(xor.prefixes().count(), 201015)
    }

    #[test]
    fn diff_of_sets_from_prefixes_do_not_contain_removed_prefixes() {
        let s: PrefixSet<_> = data_set::<Prefix<Ipv6>>("AS-WOLCOMM-ipv6-prefixes", 0, 0)
            .read()
            .into_iter()
            .collect();
        let t: PrefixSet<_> = data_set::<Prefix<Ipv6>>("AS-HURRICANE-ipv6-prefixes", 0, 0)
            .read()
            .into_iter()
            .collect();
        let diff = s.clone() - t.clone();
        let err = t.prefixes().filter(|p| diff.contains(*p)).count();
        assert_eq!(err, 0)
    }

    #[test]
    fn diff_of_sets_from_ranges_do_not_contain_removed_prefixes() {
        let s: PrefixSet<_> = data_set::<PrefixRange<Ipv6>>("AS-WOLCOMM-ipv6-ranges", 0, 0)
            .read()
            .into_iter()
            .collect();
        let t: PrefixSet<_> = data_set::<PrefixRange<Ipv6>>("AS-HURRICANE-ipv6-ranges", 0, 0)
            .read()
            .into_iter()
            .collect();
        let diff = s.clone() - t.clone();
        let err = t.prefixes().filter(|p| diff.contains(*p)).count();
        assert_eq!(err, 0)
    }

    // #[test]
    // fn neg_of_set_from_prefixes_has_expected_size() {
    //     let s: PrefixSet<_> = data_set::<Ipv6Prefix>("AS-WOLCOMM-ipv6-prefixes", 0, 0)
    //         .read()
    //         .into_iter()
    //         .collect();
    //     let not = !s;
    //     assert_eq!(not.iter_prefixes().count(), 8_589_176_728)
    // }
}
