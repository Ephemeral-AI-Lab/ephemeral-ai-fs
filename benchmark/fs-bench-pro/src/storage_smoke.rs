//! Three approved development smokes. Host SDK owns every Store and publication.
use super::workspace_bench::{emit, quote};
use super::*;
use std::os::unix::fs::{FileExt, MetadataExt};

const MOUNT: &str = "/workspace/storage-smoke";
const WORKLOAD: &str = "/usr/local/bin/fs-benchmark-workload";

fn timed<T>(phase: &str, action: impl FnOnce() -> AnyResult<T>) -> AnyResult<T> {
    let before = process_resource_snapshot()?;
    let start = Instant::now();
    let result = action();
    let elapsed = elapsed_ns(start);
    let after = process_resource_snapshot()?;
    emit(
        "storage-smoke-phase",
        &[
            ("phase", quote(phase)),
            ("elapsed_ns", elapsed.to_string()),
            ("success", result.is_ok().to_string()),
            (
                "host_cpu_ns",
                (after.user_cpu_ns - before.user_cpu_ns + after.system_cpu_ns
                    - before.system_cpu_ns)
                    .to_string(),
            ),
            ("host_rss_bytes", after.resident_bytes.to_string()),
            (
                "host_lifetime_peak_rss_bytes",
                after.peak_resident_bytes.to_string(),
            ),
            (
                "host_footprint_bytes",
                after.physical_footprint_bytes.to_string(),
            ),
            (
                "host_disk_read_bytes",
                after
                    .disk_read_bytes
                    .saturating_sub(before.disk_read_bytes)
                    .to_string(),
            ),
            (
                "host_disk_write_bytes",
                after
                    .disk_write_bytes
                    .saturating_sub(before.disk_write_bytes)
                    .to_string(),
            ),
        ],
    );
    if after.resident_bytes > 8 * 1024 * 1024 * 1024 || after.swaps != before.swaps {
        return Err("storage smoke host resource bound".into());
    }
    result
}

fn storage(store: &LayerStackStore, label: &str) -> AnyResult<()> {
    let mut apparent = 0;
    let mut allocated = 0;
    for suffix in ["", "-wal", "-shm", "-journal"] {
        match std::fs::metadata(format!("{}{suffix}", store.path().display())) {
            Ok(m) => {
                apparent += m.len();
                allocated += m.blocks() * 512;
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
    }
    // Outside operation timing, and before any verifier-created records.
    let canonical = store.canonical_storage()?;
    emit(
        "storage-smoke-allocation",
        &[
            ("label", quote(label)),
            ("store_apparent_bytes", apparent.to_string()),
            ("store_allocated_bytes", allocated.to_string()),
            ("canonical_bytes", canonical.encoded_bytes.to_string()),
            ("canonical_objects", canonical.objects.to_string()),
        ],
    );
    if apparent > 16 * 1024 * 1024 * 1024 || allocated > 16 * 1024 * 1024 * 1024 {
        return Err("storage smoke Store budget".into());
    }
    Ok(())
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

fn workload(client: &Client, id: WorkspaceId, args: &[&str]) -> AnyResult<OutputPage> {
    let argv = std::iter::once(WORKLOAD)
        .chain(args.iter().copied())
        .map(OsString::from)
        .collect();
    let output = timed("exec", || execute(client, id, argv))?;
    emit(
        "storage-smoke-execution",
        &[
            ("receipt", quote(&format!("{:?}", output.receipt))),
            (
                "output",
                quote(&String::from_utf8_lossy(
                    &output
                        .chunks
                        .iter()
                        .flat_map(|c| c.bytes.iter().copied())
                        .collect::<Vec<_>>(),
                )),
            ),
        ],
    );
    Ok(output)
}

fn commit(
    client: &Client,
    store: &LayerStackStore,
    id: WorkspaceId,
    branch: BranchId,
    index: usize,
    sdk: bool,
) -> AnyResult<()> {
    let status = timed("commit", || {
        Ok(client.commit_workspace_session_with_status(id)?)
    })?;
    let (head, created) = match status.result {
        WorkspaceCommitResult::Created { commit_id, .. } => (Some(commit_id), true),
        WorkspaceCommitResult::UpToDate { head } => (head, false),
        other => return Err(format!("storage smoke Commit: {other:?}").into()),
    };
    // A published-but-finalization-failed result must survive any later failure.
    emit(
        "storage-smoke-acknowledged",
        &[
            ("index", index.to_string()),
            (
                "commit_id",
                head.map(|h| quote(&h.to_string())).unwrap_or("null".into()),
            ),
            ("created", created.to_string()),
            (
                "presentation_failed",
                status.presentation_failed.to_string(),
            ),
        ],
    );
    if status.presentation_failed {
        return Err("published but finalization failed".into());
    }
    visible_head(client, branch, head)?;
    let snapshot = client.monitor_snapshot()?;
    let receipt = snapshot
        .operations
        .iter()
        .rev()
        .find(|r| r.operation.family == OperationFamily::WorkspaceCommit)
        .ok_or("Commit receipt")?;
    let candidate = receipt.candidate;
    let count = |f: fn(CandidateStats) -> u64| {
        candidate
            .map(f)
            .map(|v| v.to_string())
            .unwrap_or("null".into())
    };
    let fuse = edit_fuse_metrics(&snapshot);
    emit(
        "storage-smoke-receipt",
        &[
            ("candidate_bytes", count(|c| c.candidate_bytes)),
            ("inserted_bytes", count(|c| c.inserted_bytes)),
            ("reused_bytes", count(|c| c.reused_bytes)),
            ("candidate_objects", count(|c| c.candidate_objects)),
            (
                "admission_transactions",
                count(|c| c.admission_transactions),
            ),
            (
                "fuse_write_requests_cumulative",
                fuse.kernel_write_requests.to_string(),
            ),
            (
                "fuse_write_bytes_cumulative",
                fuse.kernel_write_bytes.to_string(),
            ),
            ("commit_receipt", quote(&format!("{receipt:?}"))),
            (
                "commit_diagnostics",
                quote(&format!(
                    "{:?}",
                    layerfs_sdk::take_workspace_commit_diagnostics()
                )),
            ),
        ],
    );
    if sdk && (fuse.kernel_write_requests != 0 || fuse.kernel_write_bytes != 0) {
        return Err("SDK edit used FUSE writes".into());
    }
    storage(store, &format!("step-{index}"))?;
    emit(
        "storage-smoke-step",
        &[
            ("index", index.to_string()),
            (
                "commit_id",
                head.map(|h| quote(&h.to_string())).unwrap_or("null".into()),
            ),
            ("created", created.to_string()),
        ],
    );
    Ok(())
}

pub fn dispatch(args: &[OsString]) -> AnyResult<()> {
    let [_, root, container, mode, case, input] = args else {
        return Err(
            "storage-smoke-session ROOT CONTAINER performance|verification CASE INPUT".into(),
        );
    };
    let root = Path::new(root);
    let input = Path::new(input);
    let container = ContainerId(container.to_string_lossy().into_owned());
    let case = case.to_str().ok_or("smoke case encoding")?;
    let performance = mode == "performance";
    if !performance && mode != "verification" {
        return Err("smoke mode".into());
    }
    if !matches!(
        case,
        "deepseek-five"
            | "small-files"
            | "sdk-text-32k"
            | "sdk-binary-8m"
            | "fuse-text-32k"
            | "fuse-binary-8m"
    ) {
        return Err("unknown storage smoke case".into());
    }
    let binding = benchmark_container_binding(root, &container)?.ok_or("authenticated binding")?;
    let store = Arc::new(if performance {
        LayerStackStore::create(root.join("store.sqlite"))?
    } else {
        LayerStackStore::connect(root.join("store.sqlite"))?
    });
    let client = benchmark_client(store.clone(), Some(&binding))?;
    let mut active = None;
    let result = (|| -> AnyResult<()> {
        let (branch, layer) = if performance {
            let source = if case == "deepseek-five" {
                LayerStackInitialization::Empty
            } else {
                LayerStackInitialization::Directory(input.join("initial"))
            };
            let initialized = timed("init", || {
                Ok(client.initialize_layerstack(EntityName::new("storage-smoke")?, source)?)
            })?;
            emit(
                "storage-smoke-init-receipt",
                &[(
                    "receipt",
                    quote(&format!(
                        "{:?}",
                        store.take_layerstack_initialization_receipts()
                    )),
                )],
            );
            storage(&store, "after-init")?;
            let branch = timed("fork", || {
                Ok(client.fork_branch(
                    EntityName::new("history")?,
                    LocalForkSource::Layer {
                        layer_id: initialized.genesis_layer_id,
                    },
                )?)
            })?;
            std::fs::write(root.join("branch-id"), branch.to_string())?;
            std::fs::write(
                root.join("layer-id"),
                initialized.genesis_layer_id.to_string(),
            )?;
            let session = timed("mount", || {
                Ok(client.create_workspace_session(request(branch, &container))?)
            })?;
            active = Some(session.id);
            (branch, initialized.genesis_layer_id)
        } else {
            (
                std::fs::read_to_string(root.join("branch-id"))?.parse()?,
                std::fs::read_to_string(root.join("layer-id"))?.parse::<layerfs_sdk::LayerId>()?,
            )
        };
        emit(
            "storage-smoke-ready",
            &[
                ("case", quote(case)),
                ("branch_id", quote(&branch.to_string())),
                ("layer_id", quote(&layer.to_string())),
            ],
        );
        for (ordinal, line) in std::io::stdin().lock().lines().enumerate() {
            let line = line?;
            let fields: Vec<_> = line.split('\t').collect();
            match fields.as_slice() {
                ["close"] => break,
                ["read", pass] if performance && case == "small-files" => {
                    let output = workload(
                        &client,
                        active.ok_or("smoke Workspace")?,
                        &["storage-smoke-read", MOUNT],
                    )?;
                    emit(
                        "storage-smoke-read",
                        &[
                            ("pass", quote(pass)),
                            ("read_bytes", parse_read_bytes(&output)?.to_string()),
                        ],
                    );
                }
                ["step", index] if performance => {
                    let index: usize = index.parse()?;
                    let id = active.ok_or("smoke Workspace")?;
                    let sdk = case.starts_with("sdk-");
                    if !(1..=if case == "small-files" { 3 } else { 5 }).contains(&index) {
                        return Err("smoke step range".into());
                    }
                    let mut members = 0;
                    let mut execs = 0;
                    if case == "deepseek-five" {
                        workload(
                            &client,
                            id,
                            &["storage-smoke-import", "/input/checkpoint", MOUNT],
                        )?;
                        execs = 1;
                    } else if case == "small-files" {
                        workload(
                            &client,
                            id,
                            &["storage-smoke-small-change", MOUNT, &index.to_string()],
                        )?;
                        execs = 1;
                    } else if index < 5 {
                        if sdk {
                            let size = if case.ends_with("text-32k") {
                                32768
                            } else {
                                8388608
                            };
                            let spans: Vec<usize> = if index == 4 {
                                vec![1, 2, 3]
                            } else {
                                vec![index]
                            };
                            let mut edits = Vec::new();
                            for span in spans {
                                let start = size * span as u64 / 4;
                                let mut bytes = vec![b'A' + index as u8; 4096];
                                if index == 4 {
                                    std::fs::File::open(input.join("initial/file"))?
                                        .read_exact_at(&mut bytes, start)?;
                                }
                                edits.push(WorkspaceFileRangeEdit {
                                    workspace_id: id,
                                    path: "file".into(),
                                    start,
                                    delete_len: 4096,
                                    replacement: WorkspaceFileReplacement::Inline(bytes),
                                });
                            }
                            members = edits.len();
                            timed("sdk-edit", || {
                                if index == 4 {
                                    client.edit_workspace_file_ranges(edits)?;
                                } else {
                                    client.edit_workspace_file_range(
                                        edits.pop().ok_or("edit member")?,
                                    )?;
                                }
                                Ok(())
                            })?;
                        } else {
                            workload(
                                &client,
                                id,
                                &[
                                    "storage-smoke-file-change",
                                    MOUNT,
                                    "/input/fixture",
                                    &index.to_string(),
                                ],
                            )?;
                            execs = 1;
                        }
                    }
                    emit(
                        "storage-smoke-calls",
                        &[
                            ("index", index.to_string()),
                            ("sdk_edit_member_count", members.to_string()),
                            ("sdk_edit_call_count", usize::from(members > 0).to_string()),
                            ("exec_process_count", execs.to_string()),
                            ("commit_call_count", "1".into()),
                        ],
                    );
                    commit(&client, &store, id, branch, index, sdk)?;
                }
                ["verify", identity] if !performance => {
                    let source = if *identity == "initial" {
                        LocalForkSource::Layer { layer_id: layer }
                    } else {
                        LocalForkSource::Branch {
                            branch_id: branch,
                            commit_id: identity.parse()?,
                        }
                    };
                    let fork = client
                        .fork_branch(EntityName::new(format!("verify-{ordinal}"))?, source)?;
                    let session = timed("verify-mount", || {
                        Ok(client.create_workspace_session(request(fork, &container))?)
                    })?;
                    active = Some(session.id);
                    workload(
                        &client,
                        session.id,
                        &["storage-smoke-observe", MOUNT, "/input/observed.tsv"],
                    )?;
                    timed("verify-end", || {
                        Ok(client.end_workspace_session(session.id, EndWorkspaceMode::Clean)?)
                    })?;
                    active = None;
                    emit(
                        "storage-smoke-verified-read",
                        &[("identity", quote(identity))],
                    );
                }
                _ => return Err("storage smoke control protocol".into()),
            }
        }
        Ok(())
    })();
    let cleanup = if let Some(id) = active {
        timed("end", || {
            Ok(client.end_workspace_session(
                id,
                if result.is_ok() {
                    EndWorkspaceMode::Clean
                } else {
                    EndWorkspaceMode::Discard
                },
            )?)
        })
        .map(|_| ())
    } else {
        Ok(())
    };
    let clean = cleanup.is_ok()
        && client.active_workspace_count()? == 0
        && client.active_execution_count()? == 0;
    if performance {
        storage(&store, "after-end")?;
    }
    drop(client);
    drop(store);
    emit(
        "storage-smoke-closed",
        &[
            ("cleanup_ok", clean.to_string()),
            ("success", result.is_ok().to_string()),
        ],
    );
    result?;
    cleanup?;
    if !clean {
        return Err("storage smoke runtime cleanup".into());
    }
    Ok(())
}
