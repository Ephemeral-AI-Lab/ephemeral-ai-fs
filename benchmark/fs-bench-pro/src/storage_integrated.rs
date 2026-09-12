//! Ordinary schema10 format probe and correctness-only live smoke.
use super::*;
use layerfs_sdk::ContainerBinding;

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
    if schema != 10 || groups == 0 {
        return Err("linked ordinary schema10 format probe failed".into());
    }
    emit(
        "storage-format-probe",
        &[
            ("schema_version", schema.to_string()),
            ("metadata_groups", groups.to_string()),
            ("storage_policy", quote("ordinary")),
            ("sqlite_version", quote(rusqlite::version())),
            ("status", quote("PASS")),
        ],
    );
    Ok(())
}
fn check_script(small: &str, large: &str, after: bool) -> String {
    let mut script = format!("set -eu; test \"$(stat -f -c %t {MOUNT})\" = 65735546; test \"$(sha256sum {MOUNT}/large-a | cut -d ' ' -f1)\" = {large}; test \"$(sha256sum {MOUNT}/small-a | cut -d ' ' -f1)\" = {small}; test \"$(stat -c %h {MOUNT}/alias)\" = 2");
    if after {
        script.push_str(&format!("; test \"$(cat {MOUNT}/after)\" = after"));
    }
    script
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
    let destination = store.path().to_path_buf();
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
            ("compactions", "0".into()),
        ],
    );
    Ok(())
}
pub(super) fn dispatch(args: &[OsString]) -> AnyResult<()> {
    match args {
        [command, root] if command == "storage-format-probe" => probe(Path::new(root)),
        [command, root, container] if command == "storage-integration-smoke" => smoke(
            Path::new(root),
            &ContainerId(container.to_string_lossy().into_owned()),
        ),
        _ => Err("integrated storage command arguments".into()),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn ordinary_format_probe_does_not_create_compacted_storage() {
        let root = std::env::temp_dir().join(format!(
            "layerfs-ordinary-probe-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&root).unwrap();
        super::probe(&root).unwrap();
        let names = std::fs::read_dir(&root)
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(
            names,
            ["input", "store.sqlite"]
                .into_iter()
                .map(std::ffi::OsString::from)
                .collect()
        );
        let store = super::LayerStackStore::connect(root.join("store.sqlite")).unwrap();
        let compacted: bool = store
            .inspect_connection(|db| {
                db.query_row(
            "SELECT EXISTS(SELECT 1 FROM object_packs WHERE substr(data,1,6)=x'4c46434e5431')",
            [], |row| row.get(0))
            })
            .unwrap()
            .unwrap();
        assert!(!compacted);
        drop(store);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn integration_scripts_parse_with_and_without_following_mutations() {
        for after in [false, true] {
            for suffix in ["", "; printf changed > /workspace/storage-smoke/small-a"] {
                let script = super::check_script("0123456789", "abcdef0123", after) + suffix;
                assert!(std::process::Command::new("/bin/sh")
                    .args(["-n", "-c", &script])
                    .status()
                    .unwrap()
                    .success());
            }
        }
    }
}
