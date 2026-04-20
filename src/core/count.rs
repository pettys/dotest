//! Summing discovered tests for `dotest count` and sanity-checks (aligned with UI totals).

/// Sum rows for `dotest count <query>`:
/// - **Short name** (no dots): match the **discovery tree path** (disk-first) by top-level folder segment.
/// - **Qualified prefix** (contains `.`, e.g. `Tmly.Test.Imports`): match the **VSTest filter** column.
pub fn sum_for_count_query(tests: &[(String, String, usize)], query: &str) -> usize {
    let q = query.trim().trim_end_matches('.');
    if q.is_empty() {
        return 0;
    }
    if q.contains('.') {
        tests
            .iter()
            .filter(|(_, fk, _)| fk == q || fk.starts_with(&format!("{}.", q)))
            .map(|(_, _, c)| c)
            .sum()
    } else {
        tests
            .iter()
            .filter(|(tree, _, _)| tree_in_top_level_disk_folder(tree, q))
            .map(|(_, _, c)| c)
            .sum()
    }
}

fn tree_in_top_level_disk_folder(tree: &str, folder: &str) -> bool {
    tree.split('.')
        .next()
        .map(|seg| seg.eq_ignore_ascii_case(folder))
        .unwrap_or(false)
}

pub fn tree_under_prefix(tree: &str, prefix: &str) -> bool {
    tree == prefix || tree.starts_with(&format!("{}.", prefix))
}

/// Resolve a short folder label (`Groups`, `Imports`) to the canonical first path segment from data.
pub fn resolve_short_segment_to_prefix(tests: &[(String, String, usize)], segment: &str) -> Option<String> {
    let want = segment.trim();
    if want.is_empty() {
        return None;
    }
    if want.contains('.') {
        return Some(want.to_string());
    }
    for (tree, _, _) in tests {
        let first = tree.split('.').next()?;
        if first.eq_ignore_ascii_case(want) {
            return Some(first.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tree_under_prefix_boundary() {
        assert!(tree_under_prefix("Tmly.Test.Groups.Foo.Bar", "Tmly.Test.Groups"));
        assert!(!tree_under_prefix("Tmly.Test.GroupsHelper.Foo", "Tmly.Test.Groups"));
        assert!(tree_under_prefix("Tmly.Test.Groups", "Tmly.Test.Groups"));
    }

    #[test]
    fn resolve_segment_finds_top_level_folder() {
        let tests = vec![
            ("Groups.OrgTreeTests.X".to_string(), "Tmly.Test.Groups.OrgTreeTests.X".to_string(), 1),
            ("Imports.A.M".to_string(), "Tmly.Test.Imports.A.M".to_string(), 1),
        ];
        let p = resolve_short_segment_to_prefix(&tests, "Groups").unwrap();
        assert_eq!(p, "Groups");
    }

    #[test]
    fn sum_count_query_disk_vs_vstest() {
        let tests = vec![
            ("Imports.M1".to_string(), "Ns.Imports.C.M1".to_string(), 3),
            ("Imports.M2".to_string(), "Ns.Imports.C.M2".to_string(), 1),
            ("Groups.G1".to_string(), "Ns.Groups.G1".to_string(), 10),
        ];
        assert_eq!(sum_for_count_query(&tests, "Imports"), 4);
        assert_eq!(sum_for_count_query(&tests, "Ns.Imports"), 4);
    }
}
