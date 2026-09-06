use super::workspace_common::{self as common, Case, Entry, EntryKind, Receipt};
use super::{ordinary_workloads, Result};
use std::collections::BTreeMap;

pub(crate) const FAMILY_ID: &str = "workspace_change_locality";

pub(crate) fn cases() -> Vec<Case> {
    let mut rows = Vec::new();
    for (kind, prefix, suffix) in [
        ("workspace-clean-commit", "workspace-clean-commit-", ""),
        ("workspace-fixed-move", "workspace-fixed-move-", ""),
        (
            "workspace-distributed-sdk-edit",
            "workspace-distributed-sdk-edit-",
            "",
        ),
        ("workspace-dense-rewrite", "workspace-dense-rewrite-", ""),
    ] {
        for tier in [1, 10, 100, 500] {
            rows.push(Case {
                id: format!(
                    "{prefix}{tier}{suffix}{}",
                    if tier <= 10 {
                        "-compact-v2"
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
    ordinary_workloads::check_cases(&cases(), 16)?;
    mixed_v4_check()
}

fn mixed_v4_wanted(tier: usize) -> BTreeMap<u64, usize> {
    if tier == 100 {
        BTreeMap::from([
            (52_428_800, 1),
            (4096, 1600),
            (114_976, 175),
            (114_975, 224),
        ])
    } else {
        BTreeMap::from([
            (314_572_800, 1),
            (104_857_600, 1),
            (4096, 4000),
            (88_651, 900),
            (88_650, 98),
        ])
    }
}

fn mixed_v4_wanted_len(tier: usize, ordinal: usize) -> u64 {
    if tier == 100 {
        match ordinal {
            0 => 52_428_800,
            1..=1600 => 4096,
            1601..=1775 => 114_976,
            _ => 114_975,
        }
    } else {
        match ordinal {
            0 => 314_572_800,
            1 => 104_857_600,
            2..=4001 => 4096,
            4002..=4901 => 88_651,
            _ => 88_650,
        }
    }
}

fn mixed_v4_check() -> Result<()> {
    const MIB: u64 = 1_048_576;
    let rows = cases();
    if rows.len() != 16
        || rows
            .iter()
            .filter(|row| row.tier <= 10 && row.id.ends_with("-compact-v2"))
            .count()
            != 8
        || rows
            .iter()
            .filter(|row| row.tier >= 100 && row.id.ends_with("-mixed-v4"))
            .count()
            != 8
        || rows
            .iter()
            .any(|row| row.tier >= 100 && !row.id.ends_with("-mixed-v4"))
    {
        return Err("locality mixed-v4 registry cardinality".into());
    }
    for tier in [100, 500] {
        let count = ordinary_workloads::mixed_v4_file_count(tier)?;
        let wanted = mixed_v4_wanted(tier);
        for seed in 1..=3 {
            let representative = rows
                .iter()
                .find(|row| row.tier == tier && row.kind == "workspace-clean-commit")
                .unwrap();
            let entries = fixture(representative, seed)?;
            let mut sizes = BTreeMap::<u64, usize>::new();
            let mut files = 0;
            for entry in &entries {
                if let EntryKind::File(content) = &entry.kind {
                    files += 1;
                    *sizes.entry(content.len()).or_default() += 1;
                }
            }
            if sizes != wanted
                || files != count
                || common::validate_entries(&entries)? != tier as u64 * MIB
            {
                return Err("mixed-v4 exact distribution".into());
            }
            let by_path = entries
                .iter()
                .map(|entry| (entry.path.as_str(), entry))
                .collect::<BTreeMap<_, _>>();
            for ordinal in 0..count {
                let path = ordinary_workloads::shard_path(ordinal / 200, ordinal % 200);
                let EntryKind::File(content) = &by_path[&path.as_str()].kind else {
                    return Err("mixed-v4 ordinal path".into());
                };
                if content.len() != mixed_v4_wanted_len(tier, ordinal)
                    || content.len() != ordinary_workloads::mixed_v4_len(tier, ordinal)?
                {
                    return Err("mixed-v4 ordinal assignment".into());
                }
            }
            let wide = files_with_prefix(&entries, "wide/");
            if wide != count / 200 * 64 || wide >= 32_000 {
                return Err("mixed-v4 wide fan-out".into());
            }
            let move_path = "regular/s000/f064.dat";
            let EntryKind::File(moved) = &by_path[move_path].kind else {
                return Err("mixed-v4 frozen move path".into());
            };
            if moved.len() != 4096 {
                return Err("mixed-v4 frozen move path is not 4 KiB".into());
            }
            for kind in [
                "workspace-clean-commit",
                "workspace-fixed-move",
                "workspace-distributed-sdk-edit",
                "workspace-dense-rewrite",
            ] {
                let case = rows
                    .iter()
                    .find(|row| row.tier == tier && row.kind == kind)
                    .unwrap();
                let sample = ordinary_workloads::mixed_v4_sample(case, seed)?;
                sample.validate()?;
                let large = if tier == 100 { 1 } else { 2 };
                if sample.ranges.len() != large
                    || sample.ranges.values().any(|ranges| ranges.len() != 3)
                {
                    return Err("mixed-v4 all-large range coverage".into());
                }
                let final_entries = expected(case, seed, 1)?;
                for entry in &sample.entries {
                    if !final_entries
                        .iter()
                        .any(|other| format!("{entry:?}") == format!("{other:?}"))
                    {
                        return Err("mixed-v4 sample recipe".into());
                    }
                }
                for path in &sample.absent {
                    if final_entries.iter().any(|entry| &entry.path == path) {
                        return Err("mixed-v4 sampled absence".into());
                    }
                }
            }
            let rewrite = rows
                .iter()
                .find(|row| row.tier == tier && row.kind == "workspace-dense-rewrite")
                .unwrap();
            let rewritten = expected(rewrite, seed, 1)?;
            let input = fixture(rewrite, seed)?;
            let input_by = input
                .iter()
                .map(|entry| (entry.path.as_str(), entry))
                .collect::<BTreeMap<_, _>>();
            for ordinal in 0..large_count(tier) {
                let path = ordinary_workloads::shard_path(ordinal / 200, ordinal % 200);
                let EntryKind::File(before) = &input_by[path.as_str()].kind else {
                    return Err("mixed-v4 rewrite input".into());
                };
                let EntryKind::File(after) = &rewritten
                    .iter()
                    .find(|entry| entry.path == path)
                    .ok_or("mixed-v4 rewrite output")?
                    .kind
                else {
                    return Err("mixed-v4 rewrite kind".into());
                };
                let mut a = [0; 64];
                let mut b = [0; 64];
                before.read_at(0, &mut a)?;
                after.read_at(0, &mut b)?;
                if a == b || before.len() != after.len() {
                    return Err("mixed-v4 rewrite identity collision".into());
                }
            }
            let sdk = rows
                .iter()
                .find(|row| row.tier == tier && row.kind == "workspace-distributed-sdk-edit")
                .unwrap();
            let edits = ordinary_workloads::sdk_edits(sdk, seed)?;
            if edits.len() != tier
                || edits.iter().any(|edit| {
                    edit.delete_len != 4096
                        || edit.replacement.len() != 4096
                        || edit.path == ordinary_workloads::shard_path(0, 0)
                        || (tier == 500 && edit.path == ordinary_workloads::shard_path(0, 1))
                })
            {
                return Err("mixed-v4 SDK targets".into());
            }
            let unique = edits
                .iter()
                .map(|edit| &edit.path)
                .collect::<std::collections::BTreeSet<_>>();
            if unique.len() != edits.len() {
                return Err("mixed-v4 SDK distinct targets".into());
            }
        }
    }
    mixed_v4_git_bound_check()?;
    Ok(())
}

fn files_with_prefix(entries: &[Entry], prefix: &str) -> usize {
    entries
        .iter()
        .filter(|entry| matches!(entry.kind, EntryKind::File(_)) && entry.path.starts_with(prefix))
        .count()
}

fn large_count(tier: usize) -> usize {
    if tier == 100 {
        1
    } else {
        2
    }
}

fn mixed_v4_git_bound_check() -> Result<()> {
    const MIB: u64 = 1_048_576;
    for tier in [100, 500] {
        let files = ordinary_workloads::mixed_v4_file_count(tier)? as u64;
        let working = 50 * MIB + (files - 1) * 4096 + 500 * 2500;
        let conservative = working.saturating_mul(2).saturating_add(32 * MIB);
        if working > 80 * MIB || conservative > 256 * MIB {
            return Err("mixed-v4 git 80/256 MiB bound".into());
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
