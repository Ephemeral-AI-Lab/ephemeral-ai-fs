use super::workspace_common::{Case, Entry, Receipt};
use super::{ordinary_workloads, Result};

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
    ordinary_workloads::check_cases(&cases(), 16)?;
    mixed_v4_check()
}

fn mixed_v4_check() -> Result<()> {
    use super::workspace_common::{self as common, EntryKind};
    use std::collections::{BTreeMap, BTreeSet};
    let rows = cases();
    if rows.len() != 16
        || rows.iter().filter(|row| row.tier <= 10 && !row.id.ends_with("-compact-v2")).count() != 0
        || rows.iter().filter(|row| row.tier >= 100 && !row.id.ends_with("-mixed-v4")).count() != 0
    {
        return Err("locality mixed-v4 registry cardinality".into());
    }
    for tier in [100, 500] {
        let wanted = if tier == 100 {
            BTreeMap::from([(52_428_800, 1), (4096, 1600), (114_976, 175), (114_975, 224)])
        } else {
            BTreeMap::from([
                (314_572_800, 1),
                (104_857_600, 1),
                (4096, 4000),
                (88_651, 900),
                (88_650, 98),
            ])
        };
        let file_count = if tier == 100 { 2000 } else { 5000 };
        let peak_kind = "workspace-dense-rewrite";
        if common::mixed_v4_peak_bytes(peak_kind, tier)? >= common::mixed_v4_container_bound() {
            return Err("workspace-mixed-v4 peak bytes exceed 2 GiB container".into());
        }
        for seed in 1..=3 {
            let case = rows
                .iter()
                .find(|row| row.tier == tier && row.kind == "workspace-clean-commit")
                .ok_or("locality mixed-v4 clean-commit")?;
            let entries = fixture(case, seed)?;
            let mut sizes = BTreeMap::<u64, usize>::new();
            let mut files = 0;
            let mut wide = 0;
            for entry in &entries {
                if let EntryKind::File(content) = &entry.kind {
                    files += 1;
                    *sizes.entry(content.len()).or_default() += 1;
                    if entry.path.starts_with("wide/") && !entry.path[5..].contains('/') {
                        wide += 1;
                    }
                }
            }
            if sizes != wanted
                || files != file_count
                || common::validate_entries(&entries)? != tier as u64 * common::MIB
                || wide >= 32_768
                || wide > 2_048
            {
                return Err(format!("workspace-mixed-v4 exact distribution tier {tier} seed {seed}").into());
            }
            let by_path = entries
                .iter()
                .map(|entry| (entry.path.as_str(), entry))
                .collect::<BTreeMap<_, _>>();
            let shards_v1 = common::shards(seed, common::mixed_v4_shard_count(tier)?, "")?;
            let shards_by_path = shards_v1
                .iter()
                .map(|entry| (entry.path.as_str(), entry))
                .collect::<BTreeMap<_, _>>();
            for ordinal in 0..file_count {
                let path = ordinary_workloads::shard_path(ordinal / 200, ordinal % 200);
                let EntryKind::File(content) = &by_path[path.as_str()].kind else {
                    return Err("workspace-mixed-v4 file kind".into());
                };
                if content.len() != common::mixed_v4_len(tier, ordinal)? {
                    return Err("workspace-mixed-v4 ordinal assignment".into());
                }
                if let Some(EntryKind::File(other)) = shards_by_path
                    .get(path.as_str())
                    .map(|entry| &entry.kind)
                {
                    if format!("{content:?}") == format!("{other:?}") {
                        return Err("workspace-mixed-v4 cache identity collided with shards-v1".into());
                    }
                }
            }
            let move_case = rows
                .iter()
                .find(|row| row.tier == tier && row.kind == "workspace-fixed-move")
                .unwrap();
            if fixture(move_case, seed)?
                .iter()
                .all(|entry| entry.path != "regular/s000/f064.dat")
            {
                return Err("workspace-mixed-v4 frozen move source".into());
            }
            let sdk_case = rows
                .iter()
                .find(|row| row.tier == tier && row.kind == "workspace-distributed-sdk-edit")
                .unwrap();
            let edits = ordinary_workloads::sdk_edits(sdk_case, seed)?;
            let large = common::mixed_v4_large_sizes(tier)?;
            let large_paths = (0..large.len())
                .map(|ordinal| ordinary_workloads::shard_path(ordinal / 200, ordinal % 200))
                .collect::<BTreeSet<_>>();
            let mut paths = BTreeSet::new();
            if edits.len() != tier {
                return Err("workspace-mixed-v4 SDK edit count".into());
            }
            for edit in &edits {
                if !paths.insert(&edit.path)
                    || large_paths.contains(&edit.path)
                    || edit.replacement.len() != 4096
                    || edit.delete_len != 4096
                {
                    return Err("workspace-mixed-v4 SDK targets".into());
                }
            }
            let rewrite = rows
                .iter()
                .find(|row| row.tier == tier && row.kind == "workspace-dense-rewrite")
                .unwrap();
            let rewritten = expected(rewrite, seed, 1)?;
            if rewritten
                .iter()
                .filter(|entry| matches!(entry.kind, EntryKind::File(_)))
                .count()
                != file_count
                || common::validate_entries(&rewritten)? != tier as u64 * common::MIB
            {
                return Err("workspace-mixed-v4 dense-rewrite totals".into());
            }
            for case in [case, move_case, sdk_case, rewrite] {
                let sample = ordinary_workloads::workspace_sample(case, seed)?;
                sample.validate()?;
                let final_entries = expected(case, seed, 1)?;
                for entry in &sample.entries {
                    if matches!(entry.kind, EntryKind::File(_))
                        && !final_entries
                            .iter()
                            .any(|other| format!("{entry:?}") == format!("{other:?}"))
                    {
                        return Err("workspace-mixed-v4 sample recipe".into());
                    }
                }
                let covered = sample
                    .ranges
                    .keys()
                    .cloned()
                    .collect::<BTreeSet<_>>();
                if covered.len() != large.len()
                    || sample.ranges.values().any(|ranges| ranges.len() != 3)
                {
                    return Err("workspace-mixed-v4 all-large range coverage".into());
                }
                for ordinal in 0..large.len() {
                    let path = ordinary_workloads::shard_path(ordinal / 200, ordinal % 200);
                    if !sample.ranges.contains_key(&path) {
                        return Err("workspace-mixed-v4 large file omitted from sample".into());
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
    fn mixed_v4_contract() {
        super::self_check().unwrap();
    }
}
