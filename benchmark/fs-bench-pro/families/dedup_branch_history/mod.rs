use super::dedup_workloads as d;
use super::workspace_common::{self, Case, Content, Entry, EntryKind, SdkEdit};
use super::Result;
pub(crate) const FAMILY: &str = "dedup_branch_history";
pub(crate) fn cases() -> Vec<Case> {
    let mut rows = d::cases(
        FAMILY,
        &[
            ("distributed", "dedup-history-distributed"),
            ("hotset", "dedup-history-hotset"),
            ("recurring", "dedup-history-recurring"),
            ("metadata", "dedup-history-metadata"),
            ("unrelated", "dedup-history-unrelated"),
        ],
    );
    for row in &mut rows {
        if row.kind == "unrelated" && matches!(row.tier, 100 | 500) {
            row.id.push_str("-mixed-v2");
        }
    }
    rows
}
/// Routine proof coverage; workload depth and all-parent checks are unchanged.
pub(crate) fn verification_steps(case: &Case) -> Vec<usize> {
    let n = case.tier;
    if n <= 10 {
        return (0..=n).collect();
    }
    let mut steps = match case.kind {
        "distributed" if n > 200 => vec![0, 1, 199, 200, 201, n / 2, n - 1, n],
        "hotset" => vec![0, 1, 7, 8, 9, n / 2, n - 1, n],
        "recurring" => vec![0, 1, 2, 3, n - 1, n],
        _ => vec![0, 1, 2, n / 2 - 1, n / 2, n - 1, n],
    };
    steps.sort_unstable();
    steps.dedup();
    steps
}

pub(crate) fn fixture(case: &Case, seed: u8) -> Result<Vec<Entry>> {
    d::validate(case, FAMILY, seed)?;
    if d::history_unrelated_mixed_v2(case) {
        d::mixed_v2_entries(seed)
    } else {
        workspace_common::shards(seed, 1, "")
    }
}
pub(crate) fn edit(case: &Case, seed: u8, step: usize) -> Result<SdkEdit> {
    d::history_edit(case, seed, step, &fixture(case, seed)?)
}
// step is the number of completed Created commits; zero denotes genesis.
pub(crate) fn expected(case: &Case, seed: u8, step: usize) -> Result<Vec<Entry>> {
    if step > case.tier {
        return Err("history expected step outside prefix".into());
    }
    let genesis = fixture(case, seed)?;
    let mut entries = genesis.clone();
    if step == 0 {
        return Ok(entries);
    }
    if case.kind == "unrelated" {
        let mixed = d::history_unrelated_mixed_v2(case);
        let files = if mixed { d::MIXED_V2_FILES } else { 200 };
        for j in 0..files {
            let path = if mixed {
                d::mixed_v2_path(j)?
            } else {
                d::shard_path(j)
            };
            let entry = entries
                .iter_mut()
                .find(|e| e.path == path)
                .ok_or("history path")?;
            let EntryKind::File(c) = &entry.kind else {
                return Err("history file type".into());
            };
            entry.kind = EntryKind::File(if mixed {
                d::mixed_v2_rewrite(seed, &path, j, step - 1, c.len())?
            } else {
                d::content(
                    FAMILY,
                    "unrelated",
                    seed,
                    200 * (step - 1) + j,
                    "bytes",
                    c.len(),
                )?
            });
        }
    } else if case.kind == "metadata" {
        entries
            .iter_mut()
            .find(|e| e.path == d::shard_path(0))
            .ok_or("metadata path")?
            .mode = if step % 2 == 1 { 0o600 } else { 0o640 };
    } else {
        for k in 0..step {
            let change = d::history_edit(case, seed, k, &genesis)?;
            let entry = entries
                .iter_mut()
                .find(|e| e.path == change.path)
                .ok_or("edit target")?;
            let EntryKind::File(old) = &entry.kind else {
                return Err("edit file type".into());
            };
            entry.kind = EntryKind::File(old.splice(
                change.start,
                change.delete_len,
                Content::Literal(change.replacement),
            )?);
        }
    }
    Ok(entries)
}
pub(crate) fn self_check() -> Result<()> {
    d::check_registry(&cases(), 20)?;
    for case in cases() {
        if d::total(&fixture(&case, 1)?) != d::MIB
            || (case.tier as u64 + 1) * d::MIB >= 1_073_741_824
        {
            return Err("history size bound".into());
        }
    }
    mixed_v2_check()
}

fn mixed_v2_check() -> Result<()> {
    use std::collections::BTreeMap;
    let rows = cases();
    if rows.len() != 20
        || rows
            .iter()
            .filter(|row| row.id.ends_with("-mixed-v2"))
            .count()
            != 2
        || rows
            .iter()
            .any(|row| row.kind != "unrelated" && row.id.ends_with("-mixed-v2"))
        || rows.iter().any(|row| {
            row.kind == "unrelated"
                && matches!(row.tier, 1 | 10)
                && (row.id.ends_with("-mixed-v2") || d::history_unrelated_mixed_v2(row))
        })
    {
        return Err("history-unrelated-mixed-v2 membership".into());
    }
    if d::mixed_v2_peak_bytes() >= workspace_common::mixed_v4_container_bound() {
        return Err("history-unrelated-mixed-v2 peak bytes exceed 2 GiB container".into());
    }
    let wanted = BTreeMap::from([
        (d::MIXED_V2_LARGE, 1usize),
        (d::MIXED_V2_SMALL, 6),
        (d::MIXED_V2_MEDIUM, 3),
    ]);
    let one_hundred = rows
        .iter()
        .find(|row| row.id == "dedup-history-unrelated-100-mixed-v2")
        .ok_or("history-unrelated-mixed-v2 100")?;
    let five_hundred = rows
        .iter()
        .find(|row| row.id == "dedup-history-unrelated-500-mixed-v2")
        .ok_or("history-unrelated-mixed-v2 500")?;
    for seed in 1..=3 {
        let genesis = fixture(one_hundred, seed)?;
        if format!("{genesis:?}") != format!("{:?}", fixture(five_hundred, seed)?) {
            return Err("history-unrelated-mixed-v2 shared genesis".into());
        }
        let mut sizes = BTreeMap::<u64, usize>::new();
        let mut files = 0;
        for entry in &genesis {
            if let EntryKind::File(content) = &entry.kind {
                files += 1;
                *sizes.entry(content.len()).or_default() += 1;
            }
        }
        if files != d::MIXED_V2_FILES
            || d::total(&genesis) != d::MIB
            || sizes != wanted
            || workspace_common::validate_entries(&genesis)? != d::MIB
        {
            return Err(format!("history-unrelated-mixed-v2 distribution seed {seed}").into());
        }
        for ordinal in 0..d::MIXED_V2_FILES {
            let path = d::mixed_v2_path(ordinal)?;
            let EntryKind::File(content) = &genesis
                .iter()
                .find(|entry| entry.path == path)
                .ok_or("history-unrelated-mixed-v2 path")?
                .kind
            else {
                return Err("history-unrelated-mixed-v2 file kind".into());
            };
            if content.len() != d::mixed_v2_len(ordinal)? {
                return Err("history-unrelated-mixed-v2 ordinal assignment".into());
            }
        }
        let shard = workspace_common::shards(seed, 1, "")?;
        let mixed_files: BTreeMap<_, _> = genesis
            .iter()
            .filter(|entry| matches!(entry.kind, EntryKind::File(_)))
            .map(|entry| (entry.path.as_str(), entry))
            .collect();
        if mixed_files
            .keys()
            .any(|path| shard.iter().any(|entry| entry.path == *path && matches!(entry.kind, EntryKind::File(_))))
        {
            return Err("history-unrelated-mixed-v2 path collided with 200-file shard".into());
        }
        for step in 0..=100 {
            if format!("{:?}", expected(one_hundred, seed, step)?)
                != format!("{:?}", expected(five_hundred, seed, step)?)
            {
                return Err("history-unrelated-mixed-v2 prefix 100 subset 500".into());
            }
        }
        let sample = d::history_sample(one_hundred, seed)?;
        sample.validate()?;
        let large_path = d::mixed_v2_path(0)?;
        let small_first = d::mixed_v2_path(1)?;
        let small_last = d::mixed_v2_path(6)?;
        let medium_first = d::mixed_v2_path(7)?;
        let medium_last = d::mixed_v2_path(9)?;
        if sample.ranges.len() != 1
            || sample.ranges.get(&large_path).map(Vec::len) != Some(3)
            || !sample.entries.iter().any(|entry| entry.path == large_path)
            || !sample.entries.iter().any(|entry| entry.path == small_first)
            || !sample.entries.iter().any(|entry| entry.path == small_last)
            || !sample.entries.iter().any(|entry| entry.path == medium_first)
            || !sample.entries.iter().any(|entry| entry.path == medium_last)
        {
            return Err("history-unrelated-mixed-v2 sampled large/small/medium coverage".into());
        }
        let final_entries = expected(one_hundred, seed, 100)?;
        for entry in &sample.entries {
            if matches!(entry.kind, EntryKind::File(_))
                && !final_entries
                    .iter()
                    .any(|other| format!("{entry:?}") == format!("{other:?}"))
            {
                return Err("history-unrelated-mixed-v2 sample recipe".into());
            }
        }
        let unrelated_one = rows
            .iter()
            .find(|row| row.id == "dedup-history-unrelated-1")
            .ok_or("unrelated-1")?;
        if fixture(unrelated_one, seed)?
            .iter()
            .filter(|entry| matches!(entry.kind, EntryKind::File(_)))
            .count()
            != 200
        {
            return Err("unrelated-1/10 must keep the 200-file shard".into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn mixed_v2_contract() {
        super::self_check().unwrap();
    }
}

pub(crate) fn apply(
    case: &Case,
    seed: u8,
    step: usize,
    verify: bool,
) -> Result<super::workspace_common::Receipt> {
    d::apply(case, seed, step, verify)
}

#[cfg(test)]
mod checkpoint_tests {
    #[test]
    fn history_samples_cover_cycles_and_final_states() {
        for case in super::cases() {
            let steps = super::verification_steps(&case);
            assert_eq!(steps.first(), Some(&0));
            assert_eq!(steps.last(), Some(&case.tier));
            assert!(steps.windows(2).all(|pair| pair[0] < pair[1]));
            if case.tier > 10 {
                assert!((6..=8).contains(&steps.len()));
                assert!(steps.contains(&(case.tier - 1)));
                if case.kind == "hotset" { assert!(steps.contains(&8) && steps.contains(&9)); }
                if case.kind == "distributed" && case.tier > 200 {
                    assert!(steps.contains(&199) && steps.contains(&200) && steps.contains(&201));
                }
            }
        }
    }
}
