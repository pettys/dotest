//! Parsing test run output for failed tests and building VSTest name filters.

#[derive(Clone, Debug, Default)]
pub(crate) struct FailedTestInfo {
    pub name: String,
    pub details: Vec<String>,
}

fn is_status_result_line(trimmed: &str) -> bool {
    trimmed.starts_with("Passed ")
        || trimmed.starts_with("Failed ")
        || trimmed.starts_with("Skipped ")
        || trimmed.starts_with('✓')
        || trimmed.starts_with('✗')
        || trimmed.starts_with('⚠')
}

pub(crate) fn extract_failed_tests(lines: &[String]) -> Vec<FailedTestInfo> {
    let mut failed: Vec<FailedTestInfo> = Vec::new();
    let mut i = 0usize;
    while i < lines.len() {
        let trimmed = lines[i].trim();
        if trimmed.starts_with("Failed ") {
            let after = trimmed.trim_start_matches("Failed ").trim();
            let name = after.split(" [").next().unwrap_or(after).trim().to_string();
            if name.is_empty() {
                i += 1;
                continue;
            }

            let mut details = Vec::new();
            let mut j = i + 1;
            while j < lines.len() {
                let next_trimmed = lines[j].trim();
                let lower = next_trimmed.to_lowercase();
                if is_status_result_line(next_trimmed)
                    || lower.starts_with("total tests:")
                    || lower.starts_with("passed:")
                    || lower.starts_with("failed:")
                    || lower.starts_with("skipped:")
                {
                    break;
                }
                details.push(lines[j].clone());
                j += 1;
            }

            if let Some(existing) = failed.iter_mut().find(|f| f.name == name) {
                if existing.details.is_empty() && !details.is_empty() {
                    existing.details = details;
                }
            } else {
                failed.push(FailedTestInfo { name, details });
            }
            i = j;
            continue;
        }
        i += 1;
    }
    failed
}

/// Strip VSTest parameter tail so `Name(a,b)` can be used in `FullyQualifiedName~` filters.
pub(crate) fn filter_key_for_vstest(name: &str) -> String {
    name.split('(').next().unwrap_or(name).trim().to_string()
}

pub(crate) fn build_filter_for_display_names(names: &[String]) -> String {
    names
        .iter()
        .map(|n| {
            let k = filter_key_for_vstest(n);
            format!("FullyQualifiedName~{k}")
        })
        .collect::<Vec<_>>()
        .join("|")
}
