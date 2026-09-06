use super::workspace_common::{Case, Entry, Receipt};
use super::{ordinary_workloads, Result};

pub(crate) const FAMILY_ID: &str = "namespace_mutation";

pub(crate) fn cases() -> Vec<Case> {
    let mut rows = Vec::new();
    for (kind, prefix, suffix) in [(
        "namespace-subtree-relocate-delete",
        "namespace-subtree-relocate-delete-",
        "",
    )] {
        for tier in [1, 10, 100, 500] {
            rows.push(Case {
                id: format!("{prefix}{tier}{suffix}{}", if tier<=10 {"-compact-v2"} else {"-mixed-v4"}),
                family: FAMILY_ID,
                tier,
                kind,
            });
        }
    }
    rows
}

pub(crate) fn fixture(case: &Case, seed: u8) -> Result<Vec<Entry>> {
    ordinary_workloads::fixture(case, seed)
}

pub(crate) fn expected(case: &Case, seed: u8, step: usize) -> Result<Vec<Entry>> {
    ordinary_workloads::expected(case, seed, step)
}

pub(crate) fn apply(case: &Case, seed: u8, step: usize, verify: bool) -> Result<Receipt> {
    ordinary_workloads::apply(case, seed, step, verify)
}

pub(crate) fn self_check() -> Result<()> {
    ordinary_workloads::check_cases(&cases(), 4)?;
    mixed_v4_check()
}

fn mixed_v4_check() -> Result<()> {
    use super::workspace_common::{self as common, EntryKind};
    let rows = cases();
    if rows.iter().filter(|row| row.tier >= 100 && !row.id.ends_with("-mixed-v4")).count() != 0
        || rows.iter().filter(|row| row.tier <= 10 && !row.id.ends_with("-compact-v2")).count() != 0
    {
        return Err("namespace mixed-v4 registry".into());
    }
    for tier in [100, 500] {
        let case = rows
            .iter()
            .find(|row| row.tier == tier)
            .ok_or("namespace mixed-v4 case")?;
        let background = if tier == 100 { 2000 } else { 5000 };
        let affected = if tier == 100 { 200 } else { 1000 };
        for seed in 1..=3 {
            let entries = fixture(case, seed)?;
            let files = entries
                .iter()
                .filter(|entry| matches!(entry.kind, EntryKind::File(_)))
                .count();
            let bytes = common::validate_entries(&entries)?;
            if files != background + 2 * affected
                || bytes != tier as u64 * common::MIB + 2 * affected as u64 * 1024
            {
                return Err(format!("namespace mixed-v4 totals tier {tier} seed {seed}").into());
            }
            if entries.iter().any(|entry| entry.path.starts_with("background/d")) {
                return Err("namespace mixed-v4 must not keep the 100k-file background".into());
            }
            let sample = ordinary_workloads::workspace_sample(case, seed)?;
            sample.validate()?;
            if sample.ranges.len() != if tier == 100 { 1 } else { 2 } {
                return Err("namespace mixed-v4 large-file sample".into());
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn mixed_v4_contract() {
        super::self_check().unwrap();
    }
}
