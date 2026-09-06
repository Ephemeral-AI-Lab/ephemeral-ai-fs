use super::workspace_common::{Case, Entry, Receipt};
use super::{ordinary_workloads, Result};

pub(crate) const FAMILY_ID: &str = "tiny_file_churn";

pub(crate) fn cases() -> Vec<Case> {
    let mut rows = Vec::new();
    for (kind, prefix, suffix) in [
        ("tiny-create", "tiny-create-", ""),
        ("tiny-stat", "tiny-stat-", ""),
        ("tiny-unlink", "tiny-unlink-", ""),
        ("tiny-bulk-create", "tiny-bulk-create-", ""),
        ("tiny-bulk-delete", "tiny-bulk-delete-", ""),
    ] {
        for tier in [1, 10, 100, 500] {
            rows.push(Case {
                id: format!(
                    "{prefix}{tier}{suffix}{}",
                    if tier <= 10 {
                        "-compact-v2"
                    } else if kind.starts_with("tiny-bulk-") {
                        "-mixed-v3"
                    } else {
                        "-mixed-v4"
                    }
                ),
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
    ordinary_workloads::check_cases(&cases(), 20)?;
    mixed_v3_check()?;
    mixed_v4_tiny_check()
}

fn mixed_v4_tiny_check() -> Result<()> {
    use super::workspace_common::{self as common, EntryKind};
    let rows = cases();
    for kind in ["tiny-create", "tiny-stat", "tiny-unlink"] {
        for tier in [100, 500] {
            let case = rows
                .iter()
                .find(|row| row.kind == kind && row.tier == tier)
                .ok_or("tiny mixed-v4 case")?;
            if !case.id.ends_with("-mixed-v4") {
                return Err("tiny mixed-v4 id".into());
            }
            let background = if tier == 100 { 2000 } else { 5000 };
            for seed in 1..=3 {
                let input = fixture(case, seed)?;
                let prepared = input
                    .iter()
                    .filter(|entry| {
                        matches!(entry.kind, EntryKind::File(_))
                            && !entry.path.starts_with("tiny/")
                    })
                    .count();
                if prepared != background {
                    return Err(format!("tiny mixed-v4 background files {kind} {tier}").into());
                }
                if common::validate_entries(&input)?
                    < tier as u64 * common::MIB
                {
                    return Err("tiny mixed-v4 background bytes".into());
                }
                let sample = ordinary_workloads::workspace_sample(case, seed)?;
                sample.validate()?;
                if sample.ranges.len() != if tier == 100 { 1 } else { 2 } {
                    return Err("tiny mixed-v4 large-file sample".into());
                }
            }
        }
    }
    Ok(())
}

// Descriptor and bounded-byte checks; no product, full materialization or benchmark.
fn mixed_v3_check() -> Result<()> {
    use super::workspace_common::{self as common, EntryKind};
    use std::collections::BTreeMap;
    for tier in [100, 500] {
        let rows = cases();
        let create = rows
            .iter()
            .find(|row| row.tier == tier && row.kind == "tiny-bulk-create")
            .unwrap();
        let delete = rows
            .iter()
            .find(|row| row.tier == tier && row.kind == "tiny-bulk-delete")
            .unwrap();
        for seed in 1..=3 {
            let witness = fixture(create, seed)?;
            let populated = expected(create, seed, 1)?;
            let delete_input = fixture(delete, seed)?;
            if format!("{populated:?}") != format!("{delete_input:?}")
                || format!("{witness:?}") != format!("{:?}", expected(delete, seed, 1)?)
            {
                return Err("mixed create/delete/witness equivalence".into());
            }
            let mut sizes = BTreeMap::<u64, usize>::new();
            let mut target_files = 0;
            for (index, entry) in populated.iter().enumerate() {
                if !entry.path.starts_with("bulk/") {
                    continue;
                }
                if let EntryKind::File(content) = &entry.kind {
                    target_files += 1;
                    *sizes.entry(content.len()).or_default() += 1;
                    let counterpart = &delete_input[index];
                    let EntryKind::File(other) = &counterpart.kind else {
                        return Err("mixed file kind".into());
                    };
                    let mut a = [0; 64];
                    let mut b = [0; 64];
                    for offset in [0, content.len() / 2, content.len() - 64] {
                        content.read_at(offset, &mut a)?;
                        other.read_at(offset, &mut b)?;
                        if a != b {
                            return Err("mixed deterministic content".into());
                        }
                    }
                }
            }
            let wanted = if tier == 100 {
                BTreeMap::from([(52_428_800, 1), (4096, 800), (246_995, 194), (246_994, 5)])
            } else {
                BTreeMap::from([
                    (104_857_600, 3),
                    (4096, 4000),
                    (193_913, 936),
                    (193_912, 61),
                ])
            };
            if sizes != wanted
                || target_files != tier * 10
                || common::validate_entries(&populated)? != (tier as u64 + 1) * common::MIB
                || common::validate_entries(&witness)? != common::MIB
                || witness
                    .iter()
                    .filter(|entry| matches!(entry.kind, EntryKind::File(_)))
                    .count()
                    != 200
            {
                return Err("mixed exact distribution".into());
            }
            let by_path = populated
                .iter()
                .map(|entry| (&entry.path, entry))
                .collect::<BTreeMap<_, _>>();
            for ordinal in 0..tier * 10 {
                let path = format!(
                    "bulk/{}",
                    ordinary_workloads::shard_path(ordinal / 200, ordinal % 200)
                );
                let EntryKind::File(content) = &by_path[&path].kind else {
                    unreachable!()
                };
                let wanted_len = if tier == 100 {
                    match ordinal {
                        0 => 52_428_800,
                        1..=800 => 4096,
                        801..=994 => 246_995,
                        _ => 246_994,
                    }
                } else {
                    match ordinal {
                        0..=2 => 104_857_600,
                        3..=4002 => 4096,
                        4003..=4938 => 193_913,
                        _ => 193_912,
                    }
                };
                if content.len() != wanted_len {
                    return Err("mixed ordinal assignment".into());
                }
            }
            let original_shape = common::shards(seed, tier / 20, "bulk")?;
            if original_shape
                .iter()
                .map(|entry| &entry.path)
                .collect::<Vec<_>>()
                != populated
                    .iter()
                    .filter(|entry| entry.path == "." || entry.path.starts_with("bulk"))
                    .map(|entry| &entry.path)
                    .collect::<Vec<_>>()
            {
                return Err("mixed shared path shape".into());
            }
            for case in [create, delete] {
                let sample = ordinary_workloads::tiny_sample(case, seed)?;
                sample.validate()?;
                let final_entries = expected(case, seed, 1)?;
                for entry in &sample.entries {
                    if !final_entries
                        .iter()
                        .any(|other| format!("{entry:?}") == format!("{other:?}"))
                    {
                        return Err("mixed sample recipe".into());
                    }
                }
                for path in &sample.absent {
                    if final_entries.iter().any(|entry| &entry.path == path) {
                        return Err("mixed sampled absence".into());
                    }
                }
                let large = if tier == 100 { 1 } else { 3 };
                if case.kind == "tiny-bulk-create" {
                    if sample.ranges.len() != large
                        || sample.ranges.values().any(|ranges| ranges.len() != 3)
                    {
                        return Err("mixed all-large range coverage".into());
                    }
                } else {
                    for ordinal in 0..large {
                        if !sample.absent.contains(&format!(
                            "bulk/{}",
                            ordinary_workloads::shard_path(0, ordinal)
                        )) {
                            return Err("mixed all-large absence coverage".into());
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn mixed_v3_contract() {
        super::self_check().unwrap();
    }
}
