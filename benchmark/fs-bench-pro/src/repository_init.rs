//! Real-source Init diagnostic; independent full-content verification is separate.
use super::*;
use layerfs_content::{filesystem, tree::inode::InodeKind, CanonicalPath};
use std::collections::{BTreeMap, BTreeSet};
use std::os::unix::fs::MetadataExt;

struct DigestSink {
    hash: workload_source::Sha256,
    bytes: u64,
}

impl Write for DigestSink {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.hash.update(bytes);
        self.bytes += bytes.len() as u64;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub(super) fn run(args: &[OsString]) -> AnyResult<()> {
    let [_, root, input, mode, oracle] = args else {
        return Err("repository-init ROOT INPUT performance|verification ORACLE.tsv".into());
    };
    let root = Path::new(root);
    let input = Path::new(input);
    let mut expected = BTreeMap::new();
    for line in std::fs::read_to_string(oracle)?.lines() {
        let fields = line.splitn(4, '\t').collect::<Vec<_>>();
        if fields.len() != 4 || !valid_digest(fields[0]) {
            return Err("repository oracle record".into());
        }
        CanonicalPath::new(fields[3])?;
        let _: u32 = fields[2].parse()?;
        if expected
            .insert(
                fields[3].to_owned(),
                (fields[0].to_owned(), fields[1].parse::<u64>()?),
            )
            .is_some()
        {
            return Err("duplicate repository oracle path".into());
        }
    }
    if expected.is_empty() {
        return Err("empty repository oracle".into());
    }
    let logical_bytes: u64 = expected.values().map(|row| row.1).sum();
    if mode == "performance" {
        if !input.is_dir() {
            return Err("repository input directory".into());
        }
        std::fs::create_dir(root)?;
        let store_path = root.join("store.sqlite");
        let store = Arc::new(LayerStackStore::create(&store_path)?);
        let client = Client::connect(store.clone())?;
        let before = process_resource_snapshot()?;
        let started = Instant::now();
        let initialized = client.initialize_layerstack(
            EntityName::new("real-source")?,
            LayerStackInitialization::Directory(input.to_owned()),
        )?;
        let elapsed = elapsed_ns(started);
        let after = process_resource_snapshot()?;
        let receipts = store.take_layerstack_initialization_receipts();
        let [scan] = receipts.as_slice() else {
            return Err("Init receipt count".into());
        };
        if scan.scanned_files != expected.len() as u64 || scan.scanned_bytes != logical_bytes {
            return Err("repository Init scan mismatch".into());
        }
        let layer = store
            .layer(initialized.genesis_layer_id)?
            .ok_or("missing initialized layer")?;
        std::fs::write(root.join("layer-id"), layer.id.to_string())?;
        std::fs::write(root.join("root-id"), layer.root_id.to_string())?;
        let canonical = store.canonical_storage()?;
        let physical = store.physical_storage_receipt();
        let candidate = operation_candidate(
            &client.monitor_snapshot()?,
            OperationFamily::LayerStackInitialize,
        )?;
        if client.active_workspace_count()? != 0 || client.active_execution_count()? != 0 {
            return Err("unexpected Init runtime activity".into());
        }
        drop(client);
        drop(store);
        let metadata = std::fs::metadata(store_path)?;
        println!("{{\"schema\":\"real-source-init-v1\",\"mode\":\"performance\",\"admission_eligible\":false,\"public_api_call_count\":1,\"verification_bytes\":0,\"layerstack_init_ns\":{elapsed},\"files\":{},\"logical_bytes\":{logical_bytes},\"canonical_objects\":{},\"canonical_bytes\":{},\"allocated_bytes\":{},\"apparent_bytes\":{},\"initialization_disk_read_bytes\":{},\"initialization_disk_write_bytes\":{},\"user_cpu_ns\":{},\"system_cpu_ns\":{},\"peak_rss_bytes\":{},\"swaps\":{},\"reused_objects\":{},\"full_selected\":{},\"delta_selected\":{},\"selected_encoded_bytes\":{},\"status\":\"PASS\"}}",
            expected.len(), canonical.objects, canonical.encoded_bytes, metadata.blocks()*512, metadata.len(),
            after.disk_read_bytes.saturating_sub(before.disk_read_bytes), after.disk_write_bytes.saturating_sub(before.disk_write_bytes),
            after.user_cpu_ns.saturating_sub(before.user_cpu_ns), after.system_cpu_ns.saturating_sub(before.system_cpu_ns),
            after.peak_resident_bytes, after.swaps.saturating_sub(before.swaps), candidate.reused_objects,
            physical.full_selected, physical.delta_selected, physical.selected_encoded_bytes);
        return Ok(());
    }
    if mode != "verification" {
        return Err("repository Init mode".into());
    }
    let started = Instant::now();
    let store = LayerStackStore::connect(root.join("store.sqlite"))?;
    let id = std::fs::read_to_string(root.join("layer-id"))?.parse::<layerfs_sdk::LayerId>()?;
    let layer = store.layer(id)?.ok_or("reopened layer missing")?;
    if layer.root_id.to_string() != std::fs::read_to_string(root.join("root-id"))? {
        return Err("reopened root mismatch".into());
    }
    let snapshot = store.snapshot_reader(layer.root_id);
    let reader = layerfs_layerstack_store::CoreReader(&snapshot);
    let mut pending = vec![String::new()];
    let mut found = BTreeSet::new();
    let mut verified_bytes = 0;
    while let Some(directory) = pending.pop() {
        let path = if directory.is_empty() {
            CanonicalPath::root()
        } else {
            CanonicalPath::new(&directory)?
        };
        let mut after = None;
        loop {
            let (page, _) = filesystem::list(
                &reader,
                layer.root_id,
                &path,
                after.as_ref(),
                256,
                64 * 1024,
            )?;
            for (name, _) in page.entries {
                let name = std::str::from_utf8(name.as_bytes())?;
                let relative = if directory.is_empty() {
                    name.to_owned()
                } else {
                    format!("{directory}/{name}")
                };
                let path = CanonicalPath::new(&relative)?;
                let (stat, _) = filesystem::stat(&reader, layer.root_id, &path)?;
                if stat.kind == InodeKind::Directory {
                    pending.push(relative);
                    continue;
                }
                let wanted = expected.get(&relative).ok_or("unexpected stored file")?;
                let mut sink = DigestSink {
                    hash: workload_source::Sha256::new(),
                    bytes: 0,
                };
                filesystem::stream(&reader, layer.root_id, &path, &mut sink)?;
                if sink.bytes != wanted.1 || workload_source::hex(&sink.hash.finish()) != wanted.0 {
                    return Err(format!("stored file mismatch: {relative}").into());
                }
                verified_bytes += sink.bytes;
                if !found.insert(relative) {
                    return Err("duplicate stored file".into());
                }
            }
            after = page.continuation;
            if after.is_none() {
                break;
            }
        }
    }
    if found.len() != expected.len() || verified_bytes != logical_bytes {
        return Err("stored namespace coverage".into());
    }
    println!("{{\"schema\":\"real-source-init-v1\",\"mode\":\"verification\",\"fresh_reopen\":true,\"files_verified\":{},\"bytes_verified\":{verified_bytes},\"verification_ns\":{},\"status\":\"PASS\"}}", found.len(), elapsed_ns(started));
    Ok(())
}
