//! Explicit selected Commit sequences over the existing namespace fixture.
//! Shared runner owns input custody, container resources, deadlines and cleanup.
use super::workspace_bench::{emit, quote};
use super::*;
use std::io::Read;

struct Edit {
    path: String,
    size: u64,
    offset: u64,
    replacement: Vec<u8>,
}

fn plans(fixture: &Path, total: u64, count: usize, step: usize) -> AnyResult<Vec<Edit>> {
    if count as u64 > total {
        return Err("sequence edit count exceeds fixture".into());
    }
    let mut edits = Vec::with_capacity(count);
    let mut next = 0;
    for index in 0..count {
        next = next.max(index as u64 * total / count as u64);
        let mut replacement = format!("C{count:03}{index:06}").into_bytes();
        if step > 0 {
            replacement[..4].copy_from_slice(format!("{step:04}").as_bytes());
        }
        loop {
            if next >= total {
                return Err("sequence selection exhausted editable files".into());
            }
            let path = format!("d{:04}/f{next:06}", next / 100);
            next += 1;
            let size = std::fs::metadata(fixture.join(&path))?.len();
            if size >= replacement.len() as u64 {
                let offset = workload_source::namespace_edit_offset(size)?;
                if offset + replacement.len() as u64 > size {
                    return Err("sequence replacement exceeds file".into());
                }
                edits.push(Edit {
                    path,
                    size,
                    offset,
                    replacement,
                });
                break;
            }
        }
    }
    Ok(edits)
}

fn verify(
    store: &LayerStackStore,
    branch: BranchId,
    root: Option<layerfs_content::ObjectId>,
    fixture: &Path,
    edits: &[Edit],
) -> AnyResult<()> {
    let pinned = store.pin_branch(branch)?;
    let root = root.unwrap_or(pinned.root);
    let reader = layerfs_layerstack_store::CoreReader(&pinned.reader);
    let mut expected = vec![0; 64 * 1024];
    let mut actual = Vec::with_capacity(expected.len());
    for edit in edits {
        let path = layerfs_content::CanonicalPath::new(&edit.path)?;
        let (stat, _) = layerfs_content::filesystem::stat(&reader, root, &path)?;
        if layerfs_content::file::content::length(
            &reader,
            layerfs_content::file::content::FileContentRoot(stat.content_root),
        )? != edit.size
        {
            return Err(format!("sequence file length mismatch: {}", edit.path).into());
        }
        let mut source = std::fs::File::open(fixture.join(&edit.path))?;
        let mut offset = 0;
        while offset < edit.size {
            let len = (edit.size - offset).min(expected.len() as u64) as usize;
            source.read_exact(&mut expected[..len])?;
            for (i, byte) in edit.replacement.iter().enumerate() {
                let at = edit.offset + i as u64;
                if (offset..offset + len as u64).contains(&at) {
                    expected[(at - offset) as usize] = *byte;
                }
            }
            actual.clear();
            layerfs_content::filesystem::read_range(
                &reader,
                root,
                &path,
                offset..offset + len as u64,
                &mut actual,
            )?;
            if actual != expected[..len] {
                return Err(format!("sequence content mismatch: {} at {offset}", edit.path).into());
            }
            offset += len as u64;
        }
    }
    Ok(())
}

pub(super) fn run(args: &[OsString]) -> AnyResult<()> {
    let [_, root, fixture, container, scenario, count, commits, reopen, active_cache, mode] = args
    else {
        return Err("workspace-sequence ROOT FIXTURE CONTAINER SCENARIO COUNT COMMITS REOPEN ACTIVE_CACHE performance|verify".into());
    };
    let count: usize = count.to_str().ok_or("count")?.parse()?;
    let commits: usize = commits.to_str().ok_or("commits")?.parse()?;
    let reopen = match reopen.to_str() {
        Some("true") => true,
        Some("false") => false,
        _ => return Err("reopen".into()),
    };
    let proof = match mode.to_str() {
        Some("verify") => true,
        Some("performance") => false,
        _ => return Err("mode".into()),
    };
    let active_cache = match active_cache.to_str() {
        Some("true") => true,
        Some("false") => false,
        _ => return Err("active cache".into()),
    };
    if active_cache && count > 100 {
        return Err("active-cache diagnostic count exceeds100".into());
    }
    if count > 100_000 || !(1..=1000).contains(&commits) {
        return Err("sequence bound".into());
    }
    let scenario = namespace_scenario(scenario.to_str().ok_or("scenario")?)?;
    let root = Path::new(root).join("work");
    std::fs::create_dir(&root)?;
    let fixture = Path::new(fixture);
    let container = ContainerId(container.to_string_lossy().into_owned());
    let binding = benchmark_container_binding(&root, &container)?.ok_or("authenticated binding")?;
    let database = root.join("store.sqlite");
    let mut store = Arc::new(LayerStackStore::create(&database)?);
    let mut client = benchmark_client(store.clone(), Some(&binding))?;
    let started = Instant::now();
    let init = client.initialize_layerstack(
        EntityName::new("sequence")?,
        LayerStackInitialization::Directory(fixture.to_path_buf()),
    )?;
    let bootstrap_ns = elapsed_ns(started);
    let scans = store.take_layerstack_initialization_receipts();
    if scans.len() != 1
        || scans[0].scanned_files != scenario.regular_files
        || scans[0].scanned_bytes != scenario.logical_bytes
    {
        return Err("sequence full fixture count".into());
    }
    let branch = client.fork_branch(
        EntityName::new("sequence-main")?,
        LocalForkSource::Layer {
            layer_id: init.genesis_layer_id,
        },
    )?;
    emit(
        "sequence-bootstrap",
        &[
            ("wall_ns", bootstrap_ns.to_string()),
            ("files", scenario.regular_files.to_string()),
            ("bytes", scenario.logical_bytes.to_string()),
        ],
    );
    let mut total_edit = 0;
    let mut total_commit = 0;
    let mut total_reopen = 0;
    let mut total_chain = 0;
    let initial_root = if proof {
        Some(store.pin_branch(branch)?.root)
    } else {
        None
    };
    let mut history = Vec::with_capacity(commits);
    for step in 0..commits {
        let edits = plans(fixture, scenario.regular_files, count, step)?;
        // Request construction is setup, outside the edit and full chain clocks.
        let requests = edits
            .iter()
            .map(|edit| (edit.path.clone(), edit.offset, edit.replacement.clone()))
            .collect::<Vec<_>>();
        let original_root = if count == 0 {
            Some(store.pin_branch(branch)?.root)
        } else {
            None
        };
        let mut reopen_ns = 0;
        let before = process_resource_snapshot()?;
        let chain_start = Instant::now();
        if reopen {
            let started = Instant::now();
            drop(client);
            drop(store);
            store = Arc::new(LayerStackStore::connect(&database)?);
            client = benchmark_client(store.clone(), Some(&binding))?;
            reopen_ns = elapsed_ns(started);
        }
        let session = client.create_workspace_session(CreateWorkspaceSession {
            branch_id: branch,
            placement: WorkspacePlacement::Container {
                container_id: container.clone(),
                root: "/workspace/sequence".into(),
            },
            projection: Some(WorkspaceProjection::Fuse),
        })?;
        let result = (|| -> AnyResult<(u64, u64)> {
            if active_cache && !edits.is_empty() {
                let paths = edits
                    .iter()
                    .map(|edit| format!("/workspace/sequence/{}", edit.path))
                    .collect::<Vec<_>>()
                    .join(" ");
                let started = Instant::now();
                let output = execute(
                    &client,
                    session.id,
                    vec![
                        "/bin/sh".into(),
                        "-c".into(),
                        format!("cat {paths} > /dev/null").into(),
                    ],
                )?;
                emit(
                    "sequence-cache-setup",
                    &[
                        ("wall_ns", elapsed_ns(started).to_string()),
                        ("exec_calls", "1".into()),
                        ("receipt", quote(&format!("{:?}", output.receipt))),
                    ],
                );
            }
            let mut edit_ns = 0;
            for (index, (path, start, replacement)) in requests.into_iter().enumerate() {
                let request = WorkspaceFileRangeEdit {
                    workspace_id: session.id,
                    path,
                    start,
                    delete_len: replacement.len() as u64,
                    replacement: WorkspaceFileReplacement::Inline(replacement),
                };
                let started = Instant::now();
                let result = client.edit_workspace_file_range(request);
                edit_ns += elapsed_ns(started);
                if let Err(error) = result {
                    emit(
                        "sequence-edit-failure",
                        &[
                            ("accepted_edits", index.to_string()),
                            ("error", quote(&format!("{error:?}"))),
                            ("edit_ns", edit_ns.to_string()),
                        ],
                    );
                    return Err(error.into());
                }
            }
            let started = Instant::now();
            let status = client.commit_workspace_session_with_status(session.id)?;
            let commit_ns = elapsed_ns(started);
            if status.presentation_failed {
                return Err("sequence Commit presentation failure".into());
            }
            match status.result {
                WorkspaceCommitResult::Created { .. } if count > 0 => {}
                WorkspaceCommitResult::UpToDate { .. } if count == 0 => {}
                other => return Err(format!("sequence unexpected Commit: {other:?}").into()),
            }
            Ok((edit_ns, commit_ns))
        })();
        let cleanup = client.end_workspace_session(
            session.id,
            if result.is_ok() {
                EndWorkspaceMode::Clean
            } else {
                EndWorkspaceMode::Discard
            },
        );
        let chain_ns = elapsed_ns(chain_start);
        if let Err(error) = &cleanup {
            emit(
                "sequence-cleanup-failure",
                &[
                    ("step", step.to_string()),
                    ("error", quote(&format!("{error:?}"))),
                ],
            );
        }
        let (edit_ns, commit_ns) = result?;
        cleanup?;
        let after = process_resource_snapshot()?;
        if after.swaps != before.swaps {
            return Err("sequence host swap".into());
        }
        if count == 0 && Some(store.pin_branch(branch)?.root) != original_root {
            return Err("sequence nochange root changed".into());
        }
        if proof {
            history.push(store.pin_branch(branch)?.root);
        }
        total_edit += edit_ns;
        total_commit += commit_ns;
        total_reopen += reopen_ns;
        total_chain += chain_ns;
        let snapshot = client.monitor_snapshot()?;
        for operation in snapshot.operations.iter().rev().take(3) {
            emit(
                "sequence-operation",
                &[("step", step.to_string()), ("receipt", operation.to_json())],
            );
            for receipt in &operation.storage {
                if let StorageReceipt::WorkspaceCommit(r) = receipt {
                    emit(
                        "sequence-commit-phases",
                        &[
                            ("step", step.to_string()),
                            ("receipt", quote(&format!("{r:?}"))),
                        ],
                    );
                }
            }
        }
        for diagnostic in layerfs_sdk::take_workspace_commit_diagnostics() {
            emit(
                "sequence-commit-diagnostics",
                &[
                    ("step", step.to_string()),
                    ("receipt", quote(&format!("{diagnostic:?}"))),
                ],
            );
        }
        emit(
            "sequence-step",
            &[
                ("step", step.to_string()),
                ("edit_ns", edit_ns.to_string()),
                ("commit_ns", commit_ns.to_string()),
                ("edit_commit_ns", (edit_ns + commit_ns).to_string()),
                ("reopen_ns", reopen_ns.to_string()),
                ("complete_chain_ns", chain_ns.to_string()),
                (
                    "user_cpu_ns",
                    after
                        .user_cpu_ns
                        .saturating_sub(before.user_cpu_ns)
                        .to_string(),
                ),
                (
                    "system_cpu_ns",
                    after
                        .system_cpu_ns
                        .saturating_sub(before.system_cpu_ns)
                        .to_string(),
                ),
                (
                    "process_lifetime_peak_rss_bytes",
                    after.peak_resident_bytes.to_string(),
                ),
                (
                    "swaps",
                    after.swaps.saturating_sub(before.swaps).to_string(),
                ),
            ],
        );
        if proof {
            let started = Instant::now();
            verify(&store, branch, None, fixture, &edits)?;
            drop(client);
            drop(store);
            store = Arc::new(LayerStackStore::connect(&database)?);
            verify(&store, branch, None, fixture, &edits)?;
            let changed = edits
                .iter()
                .map(|edit| edit.path.as_str())
                .collect::<std::collections::BTreeSet<_>>();
            let mut unchanged = Vec::new();
            for ordinal in 0..scenario.regular_files {
                let path = format!("d{:04}/f{ordinal:06}", ordinal / 100);
                if changed.contains(path.as_str()) {
                    continue;
                }
                unchanged.push(Edit {
                    size: std::fs::metadata(fixture.join(&path))?.len(),
                    path,
                    offset: 0,
                    replacement: Vec::new(),
                });
                if unchanged.len() == 100 {
                    break;
                }
            }
            verify(&store, branch, None, fixture, &unchanged)?;
            client = benchmark_client(store.clone(), Some(&binding))?;
            emit("sequence-verification", &[("status", quote("PASS")), ("changed_files_checked", count.to_string()),
                ("fresh_reopen", "true".into()), ("wall_ns", elapsed_ns(started).to_string()),
                ("unchanged_files_checked", unchanged.len().to_string()),
                ("coverage", quote("all changed-file bytes/lengths before and after reopen, plus declared unchanged-file sample; no exhaustive unchanged namespace claim"))]);
        }
    }
    if proof {
        let mut originals = plans(fixture, scenario.regular_files, count, 0)?;
        for edit in &mut originals {
            edit.replacement.clear();
        }
        verify(&store, branch, initial_root, fixture, &originals)?;
        let selected = [0, commits / 2, commits - 1]
            .into_iter()
            .collect::<std::collections::BTreeSet<_>>();
        for step in &selected {
            verify(
                &store,
                branch,
                Some(history[*step]),
                fixture,
                &plans(fixture, scenario.regular_files, count, *step)?,
            )?;
        }
        emit(
            "sequence-history-verification",
            &[
                ("status", quote("PASS")),
                ("initial_snapshot", "true".into()),
                ("retained_snapshots", selected.len().to_string()),
                ("changed_files_per_snapshot", count.to_string()),
            ],
        );
    }
    emit(
        "workspace-sequence",
        &[
            ("schema", quote("workspace-sequence-v1")),
            ("status", quote("PASS")),
            (
                "mode",
                quote(if proof { "verification" } else { "performance" }),
            ),
            ("edit_call_count", (count * commits).to_string()),
            ("commit_call_count", commits.to_string()),
            ("edit_ns", total_edit.to_string()),
            ("commit_ns", total_commit.to_string()),
            ("edit_commit_ns", (total_edit + total_commit).to_string()),
            ("reopen_ns", total_reopen.to_string()),
            ("complete_chain_ns", total_chain.to_string()),
            ("admission_eligible", "false".into()),
        ],
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn selection_is_bounded_and_nochange_needs_no_payload() {
        assert!(plans(Path::new("/nonexistent"), 100, 101, 0).is_err());
        assert!(plans(Path::new("/nonexistent"), 100, 0, 0)
            .unwrap()
            .is_empty());
    }
}
