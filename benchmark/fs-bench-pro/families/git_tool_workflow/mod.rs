use super::workspace_common::{Case, Entry, Receipt};
use super::{ordinary_workloads, Result};

pub(crate) const FAMILY_ID: &str = "git_tool_workflow";

pub(crate) fn cases() -> Vec<Case> {
    let mut rows = Vec::new();
    for (kind, prefix, suffix) in [("git-tool", "git-tool-", "")] {
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
    use std::collections::BTreeMap;
    let rows = cases();
    if rows.iter().filter(|row| row.tier >= 100 && !row.id.ends_with("-mixed-v4")).count() != 0
        || rows.iter().filter(|row| row.tier <= 10 && !row.id.ends_with("-compact-v2")).count() != 0
    {
        return Err("git mixed-v4 registry".into());
    }
    for tier in [100, 500] {
        let case = rows
            .iter()
            .find(|row| row.tier == tier)
            .ok_or("git mixed-v4 case")?;
        let wanted_files = if tier == 100 { 2000 } else { 5000 };
        for seed in 1..=3 {
            let entries = fixture(case, seed)?;
            let bytes = common::validate_entries(&entries)?;
            if bytes > common::MIXED_V4_GIT_WORKING_TREE_CAP {
                return Err(format!("git mixed-v4 working tree {bytes} exceeds 80 MiB").into());
            }
            let mut sizes = BTreeMap::<u64, usize>::new();
            let mut mixed_files = 0;
            let mut ignore = false;
            for entry in &entries {
                if let EntryKind::File(content) = &entry.kind {
                    if entry.path == common::MIXED_V4_GIT_BLOB_PATH
                        && content.len() != 50 * common::MIB
                    {
                        return Err("git mixed-v4 50 MiB blob".into());
                    }
                    if entry.path == ".gitignore" {
                        ignore = content.len() == common::MIXED_V4_GIT_IGNORE.len() as u64;
                    }
                    if !entry.path.starts_with("tracked/")
                        && !entry.path.starts_with("added/")
                        && entry.path != ".gitignore"
                    {
                        mixed_files += 1;
                        *sizes.entry(content.len()).or_default() += 1;
                    }
                }
            }
            let wanted = BTreeMap::from([(50 * common::MIB, 1), (4096, wanted_files - 1)]);
            if mixed_files != wanted_files || sizes != wanted || !ignore {
                return Err(format!("git mixed-v4 distribution tier {tier} seed {seed}").into());
            }
            let sample = ordinary_workloads::workspace_sample(case, seed)?;
            sample.validate()?;
            if sample.ranges.len() != 1
                || !sample.ranges.contains_key(common::MIXED_V4_GIT_BLOB_PATH)
            {
                return Err("git mixed-v4 sampled large blob".into());
            }
        }
        git_mixed_v4_bound_proof(case)?;
    }
    Ok(())
}

fn git_mixed_v4_bound_proof(case: &super::workspace_common::Case) -> Result<()> {
    use super::workspace_common as common;
    let root = std::env::temp_dir().join(format!(
        "layerfs-git-mixed-v4-{}-{}",
        case.tier,
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    let outcome = (|| -> Result<()> {
        let receipt = ordinary_workloads::prepare_git_reference(&root, case, 1)?;
        let bound: u64 = receipt
            .get("git_repository_transient_bound_bytes")
            .ok_or("git mixed-v4 bound receipt")?
            .parse()?;
        if bound > 256 * common::MIB {
            return Err(format!("git mixed-v4 conservative bound {bound} exceeds 256 MiB").into());
        }
        let repo = root.join("repository");
        let listed = std::process::Command::new("git")
            .current_dir(&repo)
            .args(["ls-files", "-z"])
            .output()?;
        if !listed.status.success() {
            return Err("git mixed-v4 ls-files".into());
        }
        let tracked = String::from_utf8(listed.stdout)?;
        if tracked.split('\0').any(|path| path == common::MIXED_V4_GIT_BLOB_PATH) {
            return Err("git mixed-v4 added the 50 MiB blob".into());
        }
        let mut git_bytes = 0_u64;
        for path in common::native_paths(&repo.join(".git"))? {
            let native = if path == "." {
                repo.join(".git")
            } else {
                repo.join(".git").join(path)
            };
            if native.is_file() {
                git_bytes = git_bytes
                    .checked_add(native.metadata()?.len())
                    .ok_or("git mixed-v4 .git overflow")?;
            }
        }
        let tree_bytes = common::validate_entries(&ordinary_workloads::expected(case, 1, 1)?)?;
        if tree_bytes > common::MIXED_V4_GIT_WORKING_TREE_CAP {
            return Err("git mixed-v4 expected working tree exceeds 80 MiB".into());
        }
        if git_bytes
            .checked_add(tree_bytes)
            .ok_or("git mixed-v4 complete overflow")?
            > 256 * common::MIB
        {
            return Err(format!(
                "git mixed-v4 complete repository {} exceeds 256 MiB",
                git_bytes + tree_bytes
            )
            .into());
        }
        Ok(())
    })();
    let _ = std::fs::remove_dir_all(&root);
    outcome
}

#[cfg(test)]
mod tests {
    #[test]
    fn mixed_v4_contract() {
        super::self_check().unwrap();
    }
}
