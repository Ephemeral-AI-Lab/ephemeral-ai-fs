//! Preregistered issue103 adapters: call public product operations only.
use super::*;
use layerfs_sdk::ContainerBinding;
use std::time::Duration;

fn receipt_json(r: &layerfs_sdk::CompactionReceipt) -> String {
    macro_rules! numeric { ($($field:ident),* $(,)?) => { vec![$((stringify!($field), r.$field.to_string())),*] }; }
    let mut fields = numeric![
        source_allocated_bytes,
        final_allocated_bytes,
        final_apparent_bytes,
        named_temporary_peak_bytes,
        original_objects_verified,
        added_owners_verified,
        whole_owners,
        small_full,
        small_prefix,
        whole_full,
        whole_prefix,
        native_slices,
        native_full,
        candidate_trials,
        max_depth,
        max_canonical_closure,
        max_encoded_closure,
        inventory_ns,
        owner_ns,
        encoding_ns,
        vacuum_ns,
        verification_ns,
        publication_ns,
        total_ns
    ];
    fields.extend([
        ("published", r.published.to_string()),
        ("cleanup_complete", r.cleanup_complete.to_string()),
        ("directory_synced", r.directory_synced.to_string()),
        (
            "publication_notes",
            format!(
                "[{}]",
                r.publication_notes
                    .iter()
                    .map(|v| quote(v))
                    .collect::<Vec<_>>()
                    .join(",")
            ),
        ),
    ]);
    format!(
        "{{{}}}",
        fields
            .iter()
            .map(|(k, v)| format!("{}:{v}", quote(k)))
            .collect::<Vec<_>>()
            .join(",")
    )
}
fn compact(
    store: &LayerStackStore,
    destination: &Path,
    limit: u64,
    trace_path: &Path,
) -> AnyResult<layerfs_sdk::CompactionReceipt> {
    let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let source_inode = std::fs::metadata(store.path())?.ino();
    let mut trace = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&trace_path)?;
    let trace_inode = trace.metadata()?.ino();
    let signal = stop.clone();
    let observer = std::thread::spawn(move || -> std::io::Result<(u64, u64, u64)> {
        use std::io::Write;
        let began = Instant::now();
        let mut peak = 0;
        let mut unlinked_peak = 0;
        let mut samples = 0;
        loop {
            let mut scanned = 0;
            let mut seen = std::collections::BTreeSet::new();
            let mut allocated = 0;
            let mut unlinked = 0;
            for entry in std::fs::read_dir("/dev/fd")? {
                scanned += 1;
                if scanned > 1024 {
                    return Err(std::io::Error::other("compaction FD observer bound"));
                }
                let entry = entry?;
                let fd = entry
                    .file_name()
                    .to_string_lossy()
                    .parse::<u32>()
                    .unwrap_or(0);
                if fd < 3 {
                    continue;
                }
                let Ok(m) = std::fs::metadata(entry.path()) else {
                    continue;
                };
                if !m.is_file()
                    || [source_inode, trace_inode].contains(&m.ino())
                    || !seen.insert((m.dev(), m.ino()))
                {
                    continue;
                }
                allocated += m.blocks() * 512;
                if m.nlink() == 0 {
                    unlinked += m.blocks() * 512;
                }
            }
            peak = peak.max(allocated);
            unlinked_peak = unlinked_peak.max(unlinked);
            samples += 1;
            writeln!(trace,"{{\"elapsed_ns\":{},\"open_temp_allocated_bytes\":{allocated},\"unlinked_allocated_bytes\":{unlinked}}}",began.elapsed().as_nanos())?;
            if signal.load(std::sync::atomic::Ordering::Acquire) {
                break;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        trace.flush()?;
        Ok((peak, unlinked_peak, samples))
    });
    let mut published_receipt = None;
    let result = timed(store, "compaction", || {
        let receipt = store.compact_into(
            destination,
            layerfs_sdk::CompactionOptions {
                temporary_byte_limit: limit,
            },
        )?;
        published_receipt = Some(receipt.clone());
        Ok(receipt)
    });
    stop.store(true, std::sync::atomic::Ordering::Release);
    let observed = observer.join().map_err(|_| "compaction observer panic");
    if let Some(receipt) = &published_receipt {
        emit("storage-compaction", &[("receipt", receipt_json(receipt))]);
    }
    let (peak, unlinked, samples) = observed??;
    emit(
        "storage-compaction-open-files",
        &[
            ("sampled_peak_allocated_bytes", peak.to_string()),
            (
                "sampled_unlinked_peak_allocated_bytes",
                unlinked.to_string(),
            ),
            ("samples", samples.to_string()),
            ("trace", quote(&trace_path.to_string_lossy())),
        ],
    );
    let receipt = result?;
    if !receipt.published
        || !receipt.cleanup_complete
        || !receipt.directory_synced
        || !receipt.publication_notes.is_empty()
    {
        return Err("compaction publication/cleanup incomplete; receipt retained".into());
    }
    Ok(receipt)
}
fn random(n: usize) -> Vec<u8> {
    let mut seed = 0x4101937du32;
    (0..n)
        .map(|_| {
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            seed as u8
        })
        .collect()
}
fn input(root: &Path) -> AnyResult<(PathBuf, String, String)> {
    let path = root.join("input");
    std::fs::create_dir(&path)?;
    let small = random(32768);
    let large = random(256 * 1024);
    let small_hash = workload_source::sdk_edit_common::sha256_hex(&small);
    let large_hash = workload_source::sdk_edit_common::sha256_hex(&large);
    for (name, mut bytes) in [
        ("small-a", small.clone()),
        ("small-b", small),
        ("large-a", large.clone()),
        ("large-b", large),
    ] {
        if name.ends_with('b') {
            bytes[1234] ^= 1;
        }
        std::fs::write(path.join(name), bytes)?;
    }
    std::fs::hard_link(path.join("small-a"), path.join("alias"))?;
    Ok((path, small_hash, large_hash))
}
fn initialized(
    root: &Path,
) -> AnyResult<(Arc<LayerStackStore>, layerfs_sdk::LayerId, String, String)> {
    let (input, small, large) = input(root)?;
    let store = Arc::new(LayerStackStore::create(root.join("store.sqlite"))?);
    let init = store.initialize_layerstack(
        EntityName::new("integrated")?,
        LayerStackInitialization::Directory(input),
    )?;
    Ok((store, init.genesis_layer_id, small, large))
}
fn probe(root: &Path) -> AnyResult<()> {
    let (store, layer, _, _) = initialized(root)?;
    let root_id = store.layer(layer)?.ok_or("probe layer")?.root_id;
    let bytes = layerfs_layerstack_store::ObjectSource::read_object(&*store, root_id)?;
    if bytes.get(13..21) != Some(b"LFS6FSR\0") {
        return Err("linked namespace is not compact".into());
    }
    let (schema, groups) = store.inspect_connection(|db| {
        Ok::<_, rusqlite::Error>((
            db.pragma_query_value(None, "user_version", |r| r.get::<_, i64>(0))?,
            db.query_row("SELECT count(*) FROM metadata_value_groups", [], |r| {
                r.get::<_, i64>(0)
            })?,
        ))
    })??;
    let r = compact(
        &store,
        &root.join("compacted.sqlite"),
        4 * 1024 * 1024 * 1024,
        &root.join("probe-open-files.jsonl"),
    )?;
    if schema != 10
        || groups == 0
        || r.whole_prefix == 0
        || r.small_prefix == 0
        || r.native_slices == 0
    {
        return Err("linked integrated format probe failed".into());
    }
    emit(
        "storage-format-probe",
        &[
            ("schema_version", schema.to_string()),
            ("metadata_groups", groups.to_string()),
            ("content_version", "107".into()),
            ("sqlite_version", quote(rusqlite::version())),
            ("status", quote("PASS")),
        ],
    );
    Ok(())
}
fn check_script(small: &str, large: &str, after: bool) -> String {
    format!("set -eu; test \"$(stat -f -c %t {MOUNT})\" = 65735546; test \"$(sha256sum {MOUNT}/large-a | cut -d ' ' -f1)\" = {large}; test \"$(sha256sum {MOUNT}/small-a | cut -d ' ' -f1)\" = {small}; test \"$(stat -c %h {MOUNT}/alias)\" = 2; {}", if after {format!("test \"$(cat {MOUNT}/after)\" = after")} else {String::new()})
}
fn exec_session(
    store: Arc<LayerStackStore>,
    binding: &ContainerBinding,
    container: &ContainerId,
    branch: BranchId,
    script: String,
    commit_change: bool,
) -> AnyResult<Option<layerfs_sdk::CommitId>> {
    let client = benchmark_client(store.clone(), Some(binding))?;
    let session = client.create_workspace_session(request(branch, container))?;
    let result = (|| -> AnyResult<Option<layerfs_sdk::CommitId>> {
        let output = timed(&store, "integration-exec", || {
            execute(
                &client,
                session.id,
                vec!["/bin/sh".into(), "-c".into(), script.into()],
            )
        })?;
        emit(
            "integration-exec",
            &[("receipt", quote(&format!("{:?}", output.receipt)))],
        );
        if !commit_change {
            return Ok(None);
        }
        let status = timed(&store, "integration-commit", || {
            Ok(client.commit_workspace_session_with_status(session.id)?)
        })?;
        if status.presentation_failed {
            return Err("integration Commit presentation failure".into());
        }
        match status.result {
            WorkspaceCommitResult::Created { commit_id, .. } => Ok(Some(commit_id)),
            other => Err(format!("expected Created: {other:?}").into()),
        }
    })();
    let cleanup = client.end_workspace_session(
        session.id,
        if result.is_ok() {
            EndWorkspaceMode::Clean
        } else {
            EndWorkspaceMode::Discard
        },
    );
    let value = result?;
    cleanup?;
    Ok(value)
}
fn smoke(root: &Path, container: &ContainerId) -> AnyResult<()> {
    let binding = benchmark_container_binding(root, container)?.ok_or("authenticated binding")?;
    let (store, layer, small, large) = initialized(root)?;
    let branch = store.fork_branch(
        EntityName::new("main")?,
        LocalForkSource::Layer { layer_id: layer },
    )?;
    let script = check_script(&small, &large, false)
        + &format!("; printf changed > {MOUNT}/small-a; test \"$(cat {MOUNT}/alias)\" = changed");
    let head =
        exec_session(store.clone(), &binding, container, branch, script, true)?.ok_or("commit")?;
    let destination = root.join("compacted.sqlite");
    compact(
        &store,
        &destination,
        4 * 1024 * 1024 * 1024,
        &root.join("smoke-open-files.jsonl"),
    )?;
    drop(store);
    let store = Arc::new(LayerStackStore::connect(&destination)?);
    let old = store.fork_branch(
        EntityName::new("old")?,
        LocalForkSource::Layer { layer_id: layer },
    )?;
    exec_session(
        store.clone(),
        &binding,
        container,
        old,
        check_script(&small, &large, false),
        false,
    )?;
    let fork = store.fork_branch(
        EntityName::new("fork")?,
        LocalForkSource::Branch {
            branch_id: branch,
            commit_id: head,
        },
    )?;
    let changed = workload_source::sdk_edit_common::sha256_hex(b"changed");
    exec_session(
        store.clone(),
        &binding,
        container,
        fork,
        check_script(&changed, &large, false) + &format!("; printf after > {MOUNT}/after"),
        true,
    )?;
    drop(store);
    let store = Arc::new(LayerStackStore::connect(destination)?);
    exec_session(
        store,
        &binding,
        container,
        fork,
        check_script(&changed, &large, true),
        false,
    )?;
    emit(
        "storage-integration-smoke",
        &[
            ("status", quote("PASS")),
            ("exec_calls", "4".into()),
            ("created_commits", "2".into()),
            ("compactions", "1".into()),
        ],
    );
    Ok(())
}
pub(super) fn dispatch(args: &[OsString]) -> AnyResult<()> {
    match args {
        [command, source, destination, limit, trace] if command == "storage-compact" => {
            let store = LayerStackStore::connect(Path::new(source))?;
            compact(
                &store,
                Path::new(destination),
                limit.to_str().ok_or("limit")?.parse()?,
                Path::new(trace),
            )?;
            Ok(())
        }
        [command, root] if command == "storage-format-probe" => probe(Path::new(root)),
        [command, root, container] if command == "storage-integration-smoke" => smoke(
            Path::new(root),
            &ContainerId(container.to_string_lossy().into_owned()),
        ),
        _ => Err("integrated storage command arguments".into()),
    }
}
