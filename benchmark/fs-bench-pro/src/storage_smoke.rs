//! Three approved development smokes. Host SDK owns every Store and publication.
use super::workspace_bench::{emit, quote};
use super::*;
use std::os::unix::fs::{FileExt, MetadataExt};

const MOUNT: &str = "/workspace/storage-smoke";
const WORKLOAD: &str = "/usr/local/bin/fs-benchmark-workload";

fn physical_json(receipt: layerfs_layerstack_store::PhysicalStorageReceipt) -> String {
    let fields = [
        ("group_fetches", receipt.group_fetches),
        ("encoded_read_bytes", receipt.encoded_read_bytes),
        ("decoded_read_bytes", receipt.decoded_read_bytes),
        ("decompression_calls", receipt.decompression_calls),
        ("base_fetches", receipt.base_fetches),
        ("blob_ranges", receipt.blob_ranges),
        ("eligible_targets", receipt.eligible_targets),
        ("absent_predecessors", receipt.absent_predecessors),
        ("targets_without_hints", receipt.targets_without_hints),
        ("usable_bases", receipt.usable_bases),
        ("predecessor_hints", receipt.predecessor_hints),
        ("candidate_trials", receipt.candidate_trials),
        (
            "correspondence_reserved_bytes",
            receipt.correspondence_reserved_bytes,
        ),
        (
            "correspondence_descriptors",
            receipt.correspondence_descriptors,
        ),
        (
            "correspondence_budget_skips",
            receipt.correspondence_budget_skips,
        ),
        ("budget_skips", receipt.budget_skips),
        ("fetch_budget_skips", receipt.fetch_budget_skips),
        ("match_budget_skips", receipt.match_budget_skips),
        ("instruction_budget_skips", receipt.instruction_budget_skips),
        ("memory_budget_skips", receipt.memory_budget_skips),
        ("match_comparisons", receipt.match_comparisons),
        ("seed_hash_bytes", receipt.seed_hash_bytes),
        ("matching_ns", receipt.matching_ns),
        ("full_selected", receipt.full_selected),
        ("delta_selected", receipt.delta_selected),
        ("rejected_mixed_groups", receipt.rejected_mixed_groups),
        ("full_alternative_bytes", receipt.full_alternative_bytes),
        ("mixed_alternative_bytes", receipt.mixed_alternative_bytes),
        ("selected_encoded_bytes", receipt.selected_encoded_bytes),
        ("encoding_calls", receipt.encoding_calls),
        ("encoding_ns", receipt.encoding_ns),
        ("diag_invalid", receipt.diag_invalid),
        ("diag_limit_memory_first_empty_count", receipt.diag_limit_memory_first_empty_count),
        ("diag_limit_memory_first_empty_bytes", receipt.diag_limit_memory_first_empty_bytes),
        ("diag_limit_memory_inherited_empty_count", receipt.diag_limit_memory_inherited_empty_count),
        ("diag_limit_memory_inherited_empty_bytes", receipt.diag_limit_memory_inherited_empty_bytes),
        ("diag_limit_file_first_empty_count", receipt.diag_limit_file_first_empty_count),
        ("diag_limit_file_first_empty_bytes", receipt.diag_limit_file_first_empty_bytes),
        ("diag_limit_file_inherited_empty_count", receipt.diag_limit_file_inherited_empty_count),
        ("diag_limit_file_inherited_empty_bytes", receipt.diag_limit_file_inherited_empty_bytes),
        ("diag_limit_operation_first_empty_count", receipt.diag_limit_operation_first_empty_count),
        ("diag_limit_operation_first_empty_bytes", receipt.diag_limit_operation_first_empty_bytes),
        ("diag_limit_operation_inherited_empty_count", receipt.diag_limit_operation_inherited_empty_count),
        ("diag_limit_operation_inherited_empty_bytes", receipt.diag_limit_operation_inherited_empty_bytes),
        ("diag_limit_descriptor_first_empty_count", receipt.diag_limit_descriptor_first_empty_count),
        ("diag_limit_descriptor_first_empty_bytes", receipt.diag_limit_descriptor_first_empty_bytes),
        ("diag_limit_descriptor_inherited_empty_count", receipt.diag_limit_descriptor_inherited_empty_count),
        ("diag_limit_descriptor_inherited_empty_bytes", receipt.diag_limit_descriptor_inherited_empty_bytes),
        ("diag_limit_memory_first_count", receipt.diag_limit_memory_first_count),
        ("diag_limit_memory_first_bytes", receipt.diag_limit_memory_first_bytes),
        ("diag_limit_memory_inherited_count", receipt.diag_limit_memory_inherited_count),
        ("diag_limit_memory_inherited_bytes", receipt.diag_limit_memory_inherited_bytes),
        ("diag_limit_file_first_count", receipt.diag_limit_file_first_count),
        ("diag_limit_file_first_bytes", receipt.diag_limit_file_first_bytes),
        ("diag_limit_file_inherited_count", receipt.diag_limit_file_inherited_count),
        ("diag_limit_file_inherited_bytes", receipt.diag_limit_file_inherited_bytes),
        ("diag_limit_operation_first_count", receipt.diag_limit_operation_first_count),
        ("diag_limit_operation_first_bytes", receipt.diag_limit_operation_first_bytes),
        ("diag_limit_operation_inherited_count", receipt.diag_limit_operation_inherited_count),
        ("diag_limit_operation_inherited_bytes", receipt.diag_limit_operation_inherited_bytes),
        ("diag_limit_descriptor_first_count", receipt.diag_limit_descriptor_first_count),
        ("diag_limit_descriptor_first_bytes", receipt.diag_limit_descriptor_first_bytes),
        ("diag_limit_descriptor_inherited_count", receipt.diag_limit_descriptor_inherited_count),
        ("diag_limit_descriptor_inherited_bytes", receipt.diag_limit_descriptor_inherited_bytes),
        ("diag_hints_0_count", receipt.diag_hints_0_count),
        ("diag_hints_0_bytes", receipt.diag_hints_0_bytes),
        ("diag_hints_1_count", receipt.diag_hints_1_count),
        ("diag_hints_1_bytes", receipt.diag_hints_1_bytes),
        ("diag_hints_2_count", receipt.diag_hints_2_count),
        ("diag_hints_2_bytes", receipt.diag_hints_2_bytes),
        ("diag_hints_3_count", receipt.diag_hints_3_count),
        ("diag_hints_3_bytes", receipt.diag_hints_3_bytes),
        ("diag_hints_4_count", receipt.diag_hints_4_count),
        ("diag_hints_4_bytes", receipt.diag_hints_4_bytes),
        ("diag_event_base_bytes", receipt.diag_event_base_bytes),
        ("diag_event_budget_bytes", receipt.diag_event_budget_bytes),
        ("diag_event_candidate_bytes", receipt.diag_event_candidate_bytes),
        ("diag_event_mixed_rejection_bytes", receipt.diag_event_mixed_rejection_bytes),
        ("diag_event_fetch_budget_count", receipt.diag_event_fetch_budget_count),
        ("diag_event_fetch_budget_bytes", receipt.diag_event_fetch_budget_bytes),
        ("diag_event_match_budget_count", receipt.diag_event_match_budget_count),
        ("diag_event_match_budget_bytes", receipt.diag_event_match_budget_bytes),
        ("diag_event_instruction_budget_count", receipt.diag_event_instruction_budget_count),
        ("diag_event_instruction_budget_bytes", receipt.diag_event_instruction_budget_bytes),
        ("diag_event_memory_budget_count", receipt.diag_event_memory_budget_count),
        ("diag_event_memory_budget_bytes", receipt.diag_event_memory_budget_bytes),
        ("diag_cursor_attached", receipt.diag_cursor_attached),
        ("diag_cursor_queries", receipt.diag_cursor_queries),
        ("diag_cursor_inherited", receipt.diag_cursor_inherited),
        ("diag_cursor_memory_limit", receipt.diag_cursor_memory_limit),
        ("diag_cursor_file_limit", receipt.diag_cursor_file_limit),
        ("diag_cursor_operation_limit", receipt.diag_cursor_operation_limit),
        ("diag_cursor_descriptor_limit", receipt.diag_cursor_descriptor_limit),
        ("diag_cursor_grants", receipt.diag_cursor_grants),
        ("diag_cursor_query_bytes", receipt.diag_cursor_query_bytes),
        ("diag_selected_pack_count", receipt.diag_selected_pack_count),
        ("diag_selected_pack_last_id", receipt.diag_selected_pack_last_id),
        ("diag_selected_pack_bytes", receipt.diag_selected_pack_bytes),
        ("diag_selected_pack_groups", receipt.diag_selected_pack_groups),
        ("diag_selected_pack_records", receipt.diag_selected_pack_records),
        ("diag_selected_unlocated_records", receipt.diag_selected_unlocated_records),
        ("diag_occurrence_preexisting_count", receipt.diag_occurrence_preexisting_count),
        ("diag_occurrence_preexisting_bytes", receipt.diag_occurrence_preexisting_bytes),
        ("diag_occurrence_preexisting_grants", receipt.diag_occurrence_preexisting_grants),
        ("diag_occurrence_missing_count", receipt.diag_occurrence_missing_count),
        ("diag_occurrence_missing_bytes", receipt.diag_occurrence_missing_bytes),
        ("diag_occurrence_missing_grants", receipt.diag_occurrence_missing_grants),
        ("diag_occurrence_duplicate_count", receipt.diag_occurrence_duplicate_count),
        ("diag_occurrence_duplicate_bytes", receipt.diag_occurrence_duplicate_bytes),
        ("diag_occurrence_duplicate_grants", receipt.diag_occurrence_duplicate_grants),
        ("diag_eligible_count", receipt.diag_eligible_count),
        ("diag_eligible_bytes", receipt.diag_eligible_bytes),
        ("diag_no_predecessor_count", receipt.diag_no_predecessor_count),
        ("diag_no_predecessor_bytes", receipt.diag_no_predecessor_bytes),
        ("diag_missing_span_count", receipt.diag_missing_span_count),
        ("diag_missing_span_bytes", receipt.diag_missing_span_bytes),
        ("diag_complete_empty_count", receipt.diag_complete_empty_count),
        ("diag_complete_empty_bytes", receipt.diag_complete_empty_bytes),
        ("diag_limited_empty_count", receipt.diag_limited_empty_count),
        ("diag_limited_empty_bytes", receipt.diag_limited_empty_bytes),
        ("diag_complete_hints_count", receipt.diag_complete_hints_count),
        ("diag_complete_hints_bytes", receipt.diag_complete_hints_bytes),
        ("diag_limited_hints_count", receipt.diag_limited_hints_count),
        ("diag_limited_hints_bytes", receipt.diag_limited_hints_bytes),
        ("diag_new_full_count", receipt.diag_new_full_count),
        ("diag_new_full_bytes", receipt.diag_new_full_bytes),
        ("diag_new_delta_count", receipt.diag_new_delta_count),
        ("diag_new_delta_bytes", receipt.diag_new_delta_bytes),
        ("diag_race_count", receipt.diag_race_count),
        ("diag_race_bytes", receipt.diag_race_bytes),
        ("diag_terminal_no_predecessor_count", receipt.diag_terminal_no_predecessor_count),
        ("diag_terminal_no_predecessor_bytes", receipt.diag_terminal_no_predecessor_bytes),
        ("diag_terminal_missing_span_count", receipt.diag_terminal_missing_span_count),
        ("diag_terminal_missing_span_bytes", receipt.diag_terminal_missing_span_bytes),
        ("diag_terminal_no_overlap_count", receipt.diag_terminal_no_overlap_count),
        ("diag_terminal_no_overlap_bytes", receipt.diag_terminal_no_overlap_bytes),
        ("diag_terminal_correspondence_limit_count", receipt.diag_terminal_correspondence_limit_count),
        ("diag_terminal_correspondence_limit_bytes", receipt.diag_terminal_correspondence_limit_bytes),
        ("diag_terminal_base_count", receipt.diag_terminal_base_count),
        ("diag_terminal_base_bytes", receipt.diag_terminal_base_bytes),
        ("diag_terminal_budget_count", receipt.diag_terminal_budget_count),
        ("diag_terminal_budget_bytes", receipt.diag_terminal_budget_bytes),
        ("diag_terminal_no_delta_count", receipt.diag_terminal_no_delta_count),
        ("diag_terminal_no_delta_bytes", receipt.diag_terminal_no_delta_bytes),
        ("diag_terminal_mixed_rejection_count", receipt.diag_terminal_mixed_rejection_count),
        ("diag_terminal_mixed_rejection_bytes", receipt.diag_terminal_mixed_rejection_bytes),
        ("diag_terminal_delta_count", receipt.diag_terminal_delta_count),
        ("diag_terminal_delta_bytes", receipt.diag_terminal_delta_bytes),
        ("diag_terminal_unknown_count", receipt.diag_terminal_unknown_count),
        ("diag_terminal_unknown_bytes", receipt.diag_terminal_unknown_bytes),
        ("diag_size_lt64_count", receipt.diag_size_lt64_count),
        ("diag_size_lt64_bytes", receipt.diag_size_lt64_bytes),
        ("diag_size_lt256_count", receipt.diag_size_lt256_count),
        ("diag_size_lt256_bytes", receipt.diag_size_lt256_bytes),
        ("diag_size_lt1024_count", receipt.diag_size_lt1024_count),
        ("diag_size_lt1024_bytes", receipt.diag_size_lt1024_bytes),
        ("diag_size_lt4096_count", receipt.diag_size_lt4096_count),
        ("diag_size_lt4096_bytes", receipt.diag_size_lt4096_bytes),
        ("diag_size_lt16384_count", receipt.diag_size_lt16384_count),
        ("diag_size_lt16384_bytes", receipt.diag_size_lt16384_bytes),
        ("diag_size_lt65536_count", receipt.diag_size_lt65536_count),
        ("diag_size_lt65536_bytes", receipt.diag_size_lt65536_bytes),
        ("diag_size_ge65536_count", receipt.diag_size_ge65536_count),
        ("diag_size_ge65536_bytes", receipt.diag_size_ge65536_bytes),
        ("diag_event_base", receipt.diag_event_base),
        ("diag_event_budget", receipt.diag_event_budget),
        ("diag_event_candidate", receipt.diag_event_candidate),
        ("diag_event_mixed_rejection", receipt.diag_event_mixed_rejection),
        ("diag_file_source_without_chunk", receipt.diag_file_source_without_chunk),
        ("diag_nonfile_chunk_count", receipt.diag_nonfile_chunk_count),
        ("diag_nonfile_chunk_bytes", receipt.diag_nonfile_chunk_bytes),
    ];
    format!(
        "{{{}}}",
        fields
            .into_iter()
            .map(|(key, value)| { format!("\"{key}\":{value}") })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn timed<T>(
    store: &LayerStackStore,
    phase: &str,
    action: impl FnOnce() -> AnyResult<T>,
) -> AnyResult<T> {
    let before = process_resource_snapshot()?;
    let physical_before = store.physical_storage_receipt();
    let start = Instant::now();
    let result = action();
    let elapsed = elapsed_ns(start);
    let physical = store.physical_storage_receipt().since(physical_before);
    let after = process_resource_snapshot()?;
    emit(
        "storage-smoke-phase",
        &[
            ("phase", quote(phase)),
            ("physical_storage", physical_json(physical)),
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
    let (page_size, page_count, freelist) =
        store.inspect_connection(|connection| -> AnyResult<(u64, u64, u64)> {
            let read = |name: &str| -> AnyResult<u64> {
                let value: i64 = connection.pragma_query_value(None, name, |row| row.get(0))?;
                Ok(value.try_into()?)
            };
            Ok((
                read("page_size")?,
                read("page_count")?,
                read("freelist_count")?,
            ))
        })??;
    let database = std::fs::metadata(store.path())?;
    emit(
        "storage-smoke-allocation",
        &[
            ("label", quote(label)),
            ("store_apparent_bytes", apparent.to_string()),
            ("store_allocated_bytes", allocated.to_string()),
            ("database_logical_bytes", database.len().to_string()),
            (
                "database_allocated_bytes",
                (database.blocks() * 512).to_string(),
            ),
            (
                "sidecar_logical_bytes",
                (apparent - database.len()).to_string(),
            ),
            (
                "sidecar_allocated_bytes",
                (allocated - database.blocks() * 512).to_string(),
            ),
            ("page_size_bytes", page_size.to_string()),
            ("page_count", page_count.to_string()),
            ("freelist_page_count", freelist.to_string()),
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

fn workload(
    store: &LayerStackStore,
    client: &Client,
    id: WorkspaceId,
    args: &[&str],
) -> AnyResult<OutputPage> {
    let argv = std::iter::once(WORKLOAD)
        .chain(args.iter().copied())
        .map(OsString::from)
        .collect();
    let output = timed(&store, "exec", || execute(client, id, argv))?;
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
    let status = timed(&store, "commit", || {
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
    if !performance && mode != "verification" && mode != "compatibility" {
        return Err("smoke mode".into());
    }
    if !matches!(
        case,
        "deepseek-five"
            | "deepseek-full"
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
            let source = if matches!(case, "deepseek-five" | "deepseek-full") {
                LayerStackInitialization::Empty
            } else {
                LayerStackInitialization::Directory(input.join("initial"))
            };
            let initialized = timed(&store, "init", || {
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
            let branch = timed(&store, "fork", || {
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
            let session = timed(&store, "mount", || {
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
                        &store,
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
                    if !(1..=if case == "deepseek-full" {
                        157
                    } else if case == "small-files" {
                        3
                    } else {
                        5
                    })
                        .contains(&index)
                    {
                        return Err("smoke step range".into());
                    }
                    let mut members = 0;
                    let mut execs = 0;
                    if matches!(case, "deepseek-five" | "deepseek-full") {
                        workload(
                            &store,
                            &client,
                            id,
                            &["storage-smoke-import", "/input/checkpoint", MOUNT],
                        )?;
                        execs = 1;
                    } else if case == "small-files" {
                        workload(
                            &store,
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
                            timed(&store, "sdk-edit", || {
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
                                &store,
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
                    let prefix = if mode == "compatibility" {
                        "compat"
                    } else {
                        "verify"
                    };
                    let fork = client
                        .fork_branch(EntityName::new(format!("{prefix}-{ordinal}"))?, source)?;
                    let session = timed(&store, "verify-mount", || {
                        Ok(client.create_workspace_session(request(fork, &container))?)
                    })?;
                    active = Some(session.id);
                    workload(
                        &store,
                        &client,
                        session.id,
                        &["storage-smoke-observe", MOUNT, "/input/observed.tsv"],
                    )?;
                    timed(&store, "verify-end", || {
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
        timed(&store, "end", || {
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
