use layerfs_sdk::{
    Client, ContainerId, CreateWorkspaceSession, EndWorkspaceMode, EntityName, ExecutionTransport,
    LayerStackStore, LocalForkSource, NonEmpty, StorageReceipt, WorkspacePlacement,
    WorkspaceProjection,
};
use serde_json::{json, Value};
use std::{
    ffi::OsString,
    io::{self, BufRead, Write},
    path::PathBuf,
    sync::Arc,
    time::Instant,
};
type AnyResult<T> = Result<T, Box<dyn std::error::Error>>;
include!("resources.rs");
fn emit(kind: &str, value: Value) -> AnyResult<()> {
    println!("{}", json!({"kind":kind,"value":value}));
    io::stdout().flush()?;
    Ok(())
}
fn command(expected: &str) -> AnyResult<()> {
    let mut line = String::new();
    io::stdin().lock().read_line(&mut line)?;
    if line.trim() != expected {
        return Err("probe control protocol".into());
    }
    Ok(())
}
fn physical(p: layerfs_layerstack_store::PhysicalStorageReceipt) -> Value {
    json!({"group_fetches":p.group_fetches,"blob_ranges":p.blob_ranges,"encoded_read_bytes":p.encoded_read_bytes,"decoded_read_bytes":p.decoded_read_bytes,"decompression_calls":p.decompression_calls,"base_fetches":p.base_fetches,
    "native_record_fetches":p.native_record_fetches,"native_request_bytes":p.native_request_bytes,"native_parser_bytes":p.native_parser_bytes,"native_raw_decoded_bytes":p.native_raw_decoded_bytes,"native_decode_calls":p.native_decode_calls,"native_decode_ns":p.native_decode_ns,"native_dependency_edges":p.native_dependency_edges,
    "native_depth_0":p.native_depth_0,"native_depth_1":p.native_depth_1,"native_depth_2":p.native_depth_2,"native_depth_3":p.native_depth_3,"native_depth_4":p.native_depth_4})
}
fn session_fuse(client: &Client) -> AnyResult<Value> {
    let mut reads = 0u64;
    let mut bytes = 0u64;
    let mut receipts = 0u64;
    for operation in &client.monitor_snapshot()?.operations {
        if operation.operation.family != layerfs_sdk::OperationFamily::WorkspaceEnd {
            continue;
        }
        for receipt in &operation.storage {
            if let StorageReceipt::WorkspaceRead(receipt) = receipt {
                receipts += 1;
                reads = reads
                    .checked_add(receipt.kernel_read_requests)
                    .ok_or("FUSE count overflow")?;
                bytes = bytes
                    .checked_add(receipt.kernel_read_bytes)
                    .ok_or("FUSE bytes overflow")?;
            }
        }
    }
    Ok(json!({"receipt_count":receipts,"present":receipts>0,
        "read_requests":if receipts>0{Some(reads)}else{None},
        "read_bytes":if receipts>0{Some(bytes)}else{None},
        "write_requests":null,"write_bytes":null,
        "scope":"whole session: setup mount proof + read + digest + end",
        "read_status":if receipts>0{"measured"}else{"unavailable"},
        "write_status":"unavailable: public write receipt requires Commit; probe never Commits"}))
}
fn execute(
    client: &Client,
    id: layerfs_sdk::WorkspaceId,
    argv: Vec<OsString>,
) -> AnyResult<(layerfs_sdk::OutputPage, u64, u64)> {
    let start = Instant::now();
    let execution = client.exec_workspace_session(id, NonEmpty::new(argv)?)?;
    let reader = client.workspace_output(execution.id)?;
    let mut output = reader.read(0, true)?;
    let mut reads = 1;
    while !output.exited {
        let next = reader.read(output.next_sequence, true)?;
        reads += 1;
        output.chunks.extend(next.chunks);
        output.next_sequence = next.next_sequence;
        output.truncated |= next.truncated;
        output.exited = next.exited;
        output.receipt = next.receipt;
    }
    let elapsed = u64::try_from(start.elapsed().as_nanos())?;
    Ok((output, reads, elapsed))
}
fn output_text(output: &layerfs_sdk::OutputPage) -> AnyResult<String> {
    let receipt = output.receipt.as_ref().ok_or("missing execution receipt")?;
    if !output.exited
        || output.truncated
        || receipt.exit_code != Some(0)
        || receipt.transport != ExecutionTransport::Daemon
        || receipt.daemon_timing.is_none()
        || receipt.docker_engine_calls != 0
        || !receipt.timing_balanced()
    {
        return Err("invalid daemon execution receipt".into());
    }
    let raw = output
        .chunks
        .iter()
        .flat_map(|c| c.bytes.iter().copied())
        .collect::<Vec<_>>();
    if raw.len() > 4096 {
        return Err("probe output bound".into());
    }
    Ok(String::from_utf8(raw)?)
}
fn main() {
    if let Err(error) = run() {
        let _ = emit("error", json!({"message":error.to_string()}));
        std::process::exit(1);
    }
}
fn run() -> AnyResult<()> {
    if !cfg!(target_os = "macos") {
        return Err("host must be macOS".into());
    }
    let a = std::env::args().skip(1).collect::<Vec<_>>();
    let [root, container, branch, commit, row_id, case, operation, digest] = a.as_slice() else {
        return Err("ROOT CONTAINER BRANCH COMMIT ROW CASE range|full SHA256".into());
    };
    let length = match case.as_str() {
        "sdk-text-32k" => 32768u64,
        "sdk-binary-8m" => 8388608,
        _ => return Err("case".into()),
    };
    let (offset, count) = match operation.as_str() {
        "range" => (3 * length / 4 - 2048, 4096),
        "full" => (0, length),
        _ => return Err("operation".into()),
    };
    if digest.len() != 64 || !digest.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("expected digest".into());
    }
    let root = PathBuf::from(root);
    let setup = Instant::now();
    let manager = layerfs_sdk::ContainerManager::open(
        root.parent()
            .ok_or("copy parent")?
            .join("container-control"),
    )?;
    let binding = manager.connect(container)?.binding();
    let store = Arc::new(LayerStackStore::connect(root.join("store.sqlite"))?);
    let client = Client::connect_with_container(store.clone(), binding)?;
    let fork = client.fork_branch(
        EntityName::new(format!("read-{row_id}"))?,
        LocalForkSource::Branch {
            branch_id: branch.parse()?,
            commit_id: commit.parse()?,
        },
    )?;
    let session = client.create_workspace_session(CreateWorkspaceSession {
        branch_id: fork,
        placement: WorkspacePlacement::Container {
            container_id: ContainerId(container.clone()),
            root: "/workspace/storage-smoke".into(),
        },
        projection: Some(WorkspaceProjection::Fuse),
    })?;
    if session.projection != WorkspaceProjection::Fuse {
        return Err("non-FUSE session".into());
    }
    let id = session.id;
    let result = (|| -> AnyResult<()> {
        // Setup-only mount proof does not open the target file.
        let proof = r#"found=0; while IFS= read -r line; do read -r -a fields <<< "$line"; if [[ ${fields[4]} == /workspace/storage-smoke ]]; then for ((i=6;i<${#fields[@]};i++)); do if [[ ${fields[i]} == - ]]; then [[ ${fields[i+1]} == fuse || ${fields[i+1]} == fuse.* ]] || exit 1; ((found+=1)); break; fi; done; fi; done < /proc/self/mountinfo; [[ $found == 1 ]]"#;
        let (out, _, _) = execute(
            &client,
            id,
            vec!["/bin/bash".into(), "-c".into(), proof.into()],
        )?;
        output_text(&out)?;
        emit(
            "ready",
            json!({"setup_ns":u64::try_from(setup.elapsed().as_nanos())?,"workspace_id":id.to_string(),"fork_branch_id":fork.to_string(),"checkpoint_index":3,"mount":"/workspace/storage-smoke","file_length_bytes":length,"offset_bytes":offset,"requested_bytes":count,"mount_proof":"PASS"}),
        )?;
        for phase in ["read", "digest"] {
            let sink = if phase == "read" {
                "/usr/bin/wc -c"
            } else {
                "/usr/bin/sha256sum"
            };
            let shell=format!("/usr/bin/dd if=/workspace/storage-smoke/file bs=65536 iflag=skip_bytes,count_bytes,fullblock skip={offset} count={count} status=none | {sink}");
            let argv = vec![
                "/bin/bash".into(),
                "-o".into(),
                "pipefail".into(),
                "-c".into(),
                shell.into(),
            ];
            command(phase)?;
            let before = process_resource_snapshot()?;
            let p = store.physical_storage_receipt();
            let action = execute(&client, id, argv);
            let after = process_resource_snapshot()?;
            let delta = store.physical_storage_receipt().since(p);
            let (out, reads, elapsed) = action?;
            // Terminal timer was stopped inside execute, before all checks below.
            let text = output_text(&out)?;
            let correct = if phase == "read" {
                text.trim().parse::<u64>()? == count
            } else {
                text.split_whitespace().next() == Some(digest.as_str())
            };
            let resources_ok = after.resident_bytes <= 8 * 1024 * 1024 * 1024
                && after.peak_resident_bytes <= 8 * 1024 * 1024 * 1024
                && after.swaps == before.swaps;
            emit(
                phase,
                json!({"elapsed_ns":elapsed,"physical_storage":physical(delta),"expected_bytes":count,"output":text.trim(),"correct":correct,"public_exec_count":1,"workspace_output_reader_count":1,"output_read_count":reads,"shell_process_count":1,"dd_process_count":1,"sink_process_count":1,"execution_receipt":format!("{:?}",out.receipt),"fuse_read_requests":null,"fuse_read_bytes":null,"fuse_write_requests":null,"fuse_write_bytes":null,"fuse_status":"unavailable per action: read metrics published only at session end; write metrics require Commit","host_cpu_ns":after.user_cpu_ns.checked_sub(before.user_cpu_ns).ok_or("CPU regression")?+after.system_cpu_ns.checked_sub(before.system_cpu_ns).ok_or("CPU regression")?,"host_rss_bytes":after.resident_bytes,"host_lifetime_peak_rss_bytes":after.peak_resident_bytes,"host_footprint_bytes":after.physical_footprint_bytes,"host_disk_read_bytes":after.disk_read_bytes.checked_sub(before.disk_read_bytes).ok_or("IO regression")?,"host_disk_write_bytes":after.disk_write_bytes.checked_sub(before.disk_write_bytes).ok_or("IO regression")?,"resource_status":if resources_ok{"PASS"}else{"FAIL"}}),
            )?;
            if !correct || !resources_ok || elapsed > 30_000_000_000 {
                return Err("read correctness/resource/time bound".into());
            }
        }
        command("end")?;
        Ok(())
    })();
    let end = Instant::now();
    let cleanup = client.end_workspace_session(
        id,
        if result.is_ok() {
            EndWorkspaceMode::Clean
        } else {
            EndWorkspaceMode::Discard
        },
    );
    let clean = cleanup.is_ok()
        && client.active_workspace_count()? == 0
        && client.active_execution_count()? == 0;
    let visible=client.query(layerfs_sdk::Query::new(layerfs_sdk::QueryKind::Branches).limit(512))?.items.into_iter().any(|item|matches!(item,layerfs_sdk::QueryItem::Branch(b) if b.id==fork && b.head_commit_id==Some(commit.parse().unwrap())));
    let session_fuse = session_fuse(&client)?;
    let metrics_present = session_fuse["present"].as_bool() == Some(true);
    drop(client);
    drop(store);
    emit(
        "closed",
        json!({"cleanup_ok":clean,"success":result.is_ok(),"unchanged_head":visible,"commit_call_count":0,"session_fuse":session_fuse,"read_only_evidence":"successful Clean end + unchanged fork head + fixed read-only commands; not a measured zero-write counter","cleanup_ns":u64::try_from(end.elapsed().as_nanos())?}),
    )?;
    result?;
    cleanup?;
    if !clean || !visible || !metrics_present {
        return Err("read probe cleanup/head".into());
    }
    Ok(())
}
