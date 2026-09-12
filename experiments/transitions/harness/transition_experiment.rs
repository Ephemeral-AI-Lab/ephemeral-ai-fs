//! Exploratory size transitions; public SDK edits with a real container FUSE projection.
use super::workspace_bench::{emit, quote};
use super::*;
use layerfs_content::{file::content, filesystem, CanonicalPath, ObjectId};
use std::os::unix::fs::MetadataExt;

const MOUNT: &str = "/workspace/transitions";
const SEED: u64 = 71503;

struct Edit {
    start: usize,
    delete: usize,
    bytes: Vec<u8>,
}

fn plan(case: &str) -> AnyResult<(Vec<u8>, Vec<Vec<Edit>>)> {
    let size = match case {
        "edit-64k" | "chain-64k" => 64 * 1024,
        "edit-127k" | "oscillate" | "batched-crossings" => 127 * 1024,
        "edit-128k" => 128 * 1024,
        "edit-129k" => 129 * 1024,
        "shrink-1m" => 1024 * 1024,
        "edit-16m" | "shrink-16m" => 16 * 1024 * 1024,
        _ => return Err("unknown transition case".into()),
    };
    let initial = workload_source::sdk_edit_common::splitmix_bytes(SEED, size);
    let mut current = initial.clone();
    let mut groups = Vec::new();
    let targets = match case {
        "oscillate" => vec![129, 127, 129, 127, 129, 127],
        "batched-crossings" => vec![130, 126, 140, 120],
        "shrink-1m" | "shrink-16m" => vec![80],
        _ => Vec::new(),
    };
    if targets.is_empty() {
        for step in 0..if case == "chain-64k" { 10 } else { 1 } {
            let start = size / 2 + step * 2048;
            let edit = Edit {
                start,
                delete: 1,
                bytes: vec![current[start] ^ 0x55],
            };
            current[start] = edit.bytes[0];
            groups.push(vec![edit]);
        }
    } else {
        let mut batch = Vec::new();
        for target in targets {
            let target = target * 1024;
            let edit = if target > current.len() {
                Edit {
                    start: current.len(),
                    delete: 0,
                    bytes: workload_source::sdk_edit_common::splitmix_bytes(
                        SEED + 1,
                        target - current.len(),
                    ),
                }
            } else {
                Edit {
                    start: target,
                    delete: current.len() - target,
                    bytes: Vec::new(),
                }
            };
            current.splice(
                edit.start..edit.start + edit.delete,
                edit.bytes.iter().copied(),
            );
            if case == "batched-crossings" {
                batch.push(edit);
            } else {
                groups.push(vec![edit]);
            }
        }
        if !batch.is_empty() {
            groups.push(batch);
        }
    }
    Ok((initial, groups))
}

fn request(branch: BranchId, container: &ContainerId) -> CreateWorkspaceSession {
    CreateWorkspaceSession {
        branch_id: branch,
        placement: WorkspacePlacement::Container {
            container_id: container.clone(),
            root: MOUNT.into(),
        },
        projection: Some(WorkspaceProjection::Fuse),
    }
}

fn file_root(
    store: &LayerStackStore,
    branch: BranchId,
    root: Option<ObjectId>,
) -> AnyResult<ObjectId> {
    let pinned = store.pin_branch(branch)?;
    let reader = layerfs_layerstack_store::CoreReader(&pinned.reader);
    Ok(filesystem::stat(
        &reader,
        root.unwrap_or(pinned.root),
        &CanonicalPath::new("file")?,
    )?
    .0
    .content_root)
}

fn verify_store(
    store: &LayerStackStore,
    branch: BranchId,
    root: ObjectId,
    expected: &[u8],
) -> AnyResult<()> {
    let pinned = store.pin_branch(branch)?;
    let reader = layerfs_layerstack_store::CoreReader(&pinned.reader);
    let mut actual = Vec::new();
    filesystem::stream(&reader, root, &CanonicalPath::new("file")?, &mut actual)?;
    if actual != expected {
        return Err("reopened retained-version bytes differ".into());
    }
    Ok(())
}

fn verify_fuse(client: &Client, workspace: WorkspaceId, expected: &[u8]) -> AnyResult<()> {
    let digest = workload_source::sdk_edit_common::sha256_hex(expected);
    let script = format!("set -eu; test \"$(stat -f -c %t {MOUNT})\" = 65735546; test \"$(stat -c %s {MOUNT}/file)\" = {}; test \"$(sha256sum {MOUNT}/file | cut -d ' ' -f1)\" = {digest}", expected.len());
    let result = execute(
        client,
        workspace,
        vec!["/bin/sh".into(), "-c".into(), script.into()],
    )?;
    emit(
        "transition-fuse-proof",
        &[
            ("status", quote("PASS")),
            ("bytes", expected.len().to_string()),
            ("receipt", quote(&format!("{:?}", result.receipt))),
        ],
    );
    Ok(())
}

pub(super) fn run(args: &[OsString]) -> AnyResult<()> {
    let [_, root, container, case, mode] = args else {
        return Err("transition-experiment ROOT CONTAINER CASE performance|verify".into());
    };
    let proof = match mode.to_str() {
        Some("performance") => false,
        Some("verify") => true,
        _ => return Err("mode".into()),
    };
    let case = case.to_str().ok_or("case")?;
    let (mut expected, groups) = plan(case)?;
    let root = Path::new(root);
    let input = root.join("input");
    std::fs::create_dir(&input)?;
    std::fs::write(input.join("file"), &expected)?;
    let container = ContainerId(container.to_string_lossy().into_owned());
    let binding =
        benchmark_container_binding(root, &container)?.ok_or("authenticated binding required")?;
    let database = root.join("store.sqlite");
    let store = Arc::new(LayerStackStore::create(&database)?);
    let client = benchmark_client(store.clone(), Some(&binding))?;
    let init_start = Instant::now();
    let init = client.initialize_layerstack(
        EntityName::new("transitions")?,
        LayerStackInitialization::Directory(input),
    )?;
    let init_ns = elapsed_ns(init_start);
    let branch = client.fork_branch(
        EntityName::new("main")?,
        LocalForkSource::Layer {
            layer_id: init.genesis_layer_id,
        },
    )?;
    let initial_root = store.pin_branch(branch)?.root;
    let initial_file = file_root(&store, branch, None)?;
    let mut history = Vec::new();
    if proof {
        history.push((initial_root, None, expected.clone()));
    }
    let container_before = container_cgroup_snapshot(&container)?;
    emit(
        "transition-setup",
        &[
            ("case", quote(case)),
            (
                "mode",
                quote(if proof { "verification" } else { "performance" }),
            ),
            ("init_ns", init_ns.to_string()),
            ("initial_bytes", expected.len().to_string()),
            (
                "fixture_sha256",
                quote(&workload_source::sdk_edit_common::sha256_hex(&expected)),
            ),
            ("initial_file_root", quote(&initial_file.to_string())),
            ("admission_eligible", "false".into()),
        ],
    );
    let mut content_roots = vec![initial_file];
    for (step, edits) in groups.into_iter().enumerate() {
        let before_len = expected.len();
        for edit in &edits {
            expected.splice(
                edit.start..edit.start + edit.delete,
                edit.bytes.iter().copied(),
            );
        }
        let after_len = expected.len();
        let calls = edits.len();
        let supplied: usize = edits.iter().map(|edit| edit.bytes.len()).sum();
        let resources_before = process_resource_snapshot()?;
        let lifecycle = Instant::now();
        let session = client.create_workspace_session(request(branch, &container))?;
        let result = (|| -> AnyResult<(u64, u64, CommitId, layerfs_layerstack_store::PhysicalStorageReceipt)> {
            if !proof {
                // Confirm FUSE without reading or warming the selected file payload.
                execute(&client, session.id, vec!["/bin/sh".into(), "-c".into(),
                    format!("test \"$(stat -f -c %t {MOUNT})\" = 65735546").into()])?;
            }
            let mut edit_ns = 0;
            layerfs_sdk::take_workspace_commit_diagnostics();
            for edit in edits {
                let request = WorkspaceFileRangeEdit { workspace_id: session.id, path: "file".into(),
                    start: edit.start as u64, delete_len: edit.delete as u64,
                    replacement: WorkspaceFileReplacement::Inline(edit.bytes) };
                let started = Instant::now();
                let changed = client.edit_workspace_file_range(request);
                edit_ns += elapsed_ns(started);
                changed?;
            }
            if proof { verify_fuse(&client, session.id, &expected)?; }
            let physical_before = store.physical_storage_receipt();
            let started = Instant::now();
            let status = client.commit_workspace_session_with_status(session.id)?;
            let commit_ns = elapsed_ns(started);
            let physical = store.physical_storage_receipt().since(physical_before);
            if status.presentation_failed { return Err("Commit presentation failed".into()); }
            let commit = match status.result {
                WorkspaceCommitResult::Created { commit_id, .. } => commit_id,
                other => return Err(format!("expected Created: {other:?}").into()),
            };
            if proof { verify_fuse(&client, session.id, &expected)?; }
            Ok((edit_ns, commit_ns, commit, physical))
        })();
        let cleanup = client.end_workspace_session(
            session.id,
            if result.is_ok() {
                EndWorkspaceMode::Clean
            } else {
                EndWorkspaceMode::Discard
            },
        );
        let lifecycle_ns = elapsed_ns(lifecycle);
        if let Err(error) = &cleanup {
            emit(
                "transition-cleanup-error",
                &[("error", quote(&error.to_string()))],
            );
        }
        let (edit_ns, commit_ns, commit, physical) = result?;
        cleanup?;
        let resources_after = process_resource_snapshot()?;
        if resources_after.swaps != resources_before.swaps {
            return Err("host swap".into());
        }
        let diagnostics = layerfs_sdk::take_workspace_commit_diagnostics();
        let cdc: u64 = diagnostics.iter().map(|d| d.cdc_bytes_scanned).sum();
        let pinned = store.pin_branch(branch)?;
        let content_root = file_root(&store, branch, Some(pinned.root))?;
        let reader = layerfs_layerstack_store::CoreReader(&pinned.reader);
        let content = content::inspect(&reader, content::FileContentRoot(content_root))?;
        let representation = match content {
            content::Content::Small { .. } => "small",
            content::Content::Chunked(_) => "chunked",
        };
        if (representation == "small") != (after_len > 0 && after_len < content::SMALL_LIMIT) {
            return Err("wrong size representation".into());
        }
        if content.logical_len() != after_len as u64 {
            return Err("wrong saved length".into());
        }
        if proof {
            verify_store(&store, branch, pinned.root, &expected)?;
            history.push((pinned.root, Some(commit), expected.clone()));
        }
        let allocated = std::fs::metadata(&database)?.blocks() * 512;
        emit(
            "transition-step",
            &[
                ("case", quote(case)),
                ("step", step.to_string()),
                ("before_bytes", before_len.to_string()),
                ("after_bytes", after_len.to_string()),
                ("sdk_edit_calls", calls.to_string()),
                ("supplied_bytes", supplied.to_string()),
                ("edit_ns", edit_ns.to_string()),
                ("commit_ns", commit_ns.to_string()),
                ("edit_commit_ns", (edit_ns + commit_ns).to_string()),
                ("lifecycle_ns", lifecycle_ns.to_string()),
                ("representation", quote(representation)),
                ("cdc_bytes_scanned", cdc.to_string()),
                ("diagnostic_records", diagnostics.len().to_string()),
                ("file_root", quote(&content_root.to_string())),
                (
                    "reused_prior_content_root",
                    content_roots.contains(&content_root).to_string(),
                ),
                ("store_allocated_bytes", allocated.to_string()),
                (
                    "commit_decoded_read_bytes",
                    physical.decoded_read_bytes.to_string(),
                ),
                (
                    "commit_encoded_read_bytes",
                    physical.encoded_read_bytes.to_string(),
                ),
                ("commit_full_selected", physical.full_selected.to_string()),
                ("commit_delta_selected", physical.delta_selected.to_string()),
                (
                    "selected_encoded_bytes",
                    physical.selected_encoded_bytes.to_string(),
                ),
                (
                    "native_full_count",
                    physical.native_admitted_full_count.to_string(),
                ),
                (
                    "native_prefix_count",
                    physical.native_admitted_prefix_count.to_string(),
                ),
                (
                    "host_user_cpu_ns",
                    resources_after
                        .user_cpu_ns
                        .saturating_sub(resources_before.user_cpu_ns)
                        .to_string(),
                ),
                (
                    "host_system_cpu_ns",
                    resources_after
                        .system_cpu_ns
                        .saturating_sub(resources_before.system_cpu_ns)
                        .to_string(),
                ),
                (
                    "host_lifetime_peak_rss_bytes",
                    resources_after.peak_resident_bytes.to_string(),
                ),
                ("commit_id", quote(&commit.to_string())),
            ],
        );
        content_roots.push(content_root);
    }
    drop(client);
    drop(store);
    if proof {
        let store = Arc::new(LayerStackStore::connect(&database)?);
        let client = benchmark_client(store.clone(), Some(&binding))?;
        for (index, (root_id, commit, expected)) in history.iter().enumerate() {
            verify_store(&store, branch, *root_id, expected)?;
            let source = match commit {
                Some(commit_id) => LocalForkSource::Branch {
                    branch_id: branch,
                    commit_id: *commit_id,
                },
                None => LocalForkSource::Layer {
                    layer_id: init.genesis_layer_id,
                },
            };
            let fork = client.fork_branch(EntityName::new(format!("proof-{index}"))?, source)?;
            let session = client.create_workspace_session(request(fork, &container))?;
            let checked = verify_fuse(&client, session.id, expected);
            let cleanup = client.end_workspace_session(session.id, EndWorkspaceMode::Discard);
            checked?;
            cleanup?;
        }
    }
    let container_after = container_cgroup_snapshot(&container)?;
    if container_after.oom != container_before.oom
        || container_after.oom_kill != container_before.oom_kill
        || container_after.swap_current != 0
    {
        return Err("container OOM/swap".into());
    }
    emit(
        "transition-complete",
        &[
            ("status", quote("PASS")),
            ("retained_versions_verified", history.len().to_string()),
            (
                "container_peak_memory_bytes",
                container_after.memory_peak.to_string(),
            ),
            ("fuse_verified", "true".into()),
            ("compactions", "0".into()),
            ("admission_eligible", "false".into()),
        ],
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn plans_cross_exact_boundaries_and_restore_the_original() {
        let (initial, groups) = plan("oscillate").unwrap();
        let mut bytes = initial.clone();
        for (step, group) in groups.iter().enumerate() {
            for edit in group {
                bytes.splice(
                    edit.start..edit.start + edit.delete,
                    edit.bytes.iter().copied(),
                );
            }
            assert_eq!(
                bytes.len(),
                if step % 2 == 0 {
                    129 * 1024
                } else {
                    127 * 1024
                }
            );
        }
        assert_eq!(bytes, initial);
        let (mut bytes, groups) = plan("batched-crossings").unwrap();
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].len(), 4);
        for edit in &groups[0] {
            bytes.splice(
                edit.start..edit.start + edit.delete,
                edit.bytes.iter().copied(),
            );
        }
        assert_eq!(bytes.len(), 120 * 1024);
        assert!(plan("unknown").is_err());
    }
}
