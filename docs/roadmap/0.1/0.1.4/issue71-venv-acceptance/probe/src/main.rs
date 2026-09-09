use layerfs_sdk::{
    Client, EntityName, LayerStackInitialization, LayerStackStore, Query, QueryKind,
};
use std::os::unix::fs::MetadataExt;
use std::{path::Path, sync::Arc, time::Instant};
fn usage() -> libc::rusage {
    let mut r = std::mem::MaybeUninit::uninit();
    assert_eq!(
        unsafe { libc::getrusage(libc::RUSAGE_SELF, r.as_mut_ptr()) },
        0
    );
    unsafe { r.assume_init() }
}
fn seconds(t: libc::timeval) -> f64 {
    t.tv_sec as f64 + t.tv_usec as f64 / 1e6
}
fn disk(p: &Path) -> std::io::Result<(u64, u64)> {
    let mut total = (0, 0);
    for e in std::fs::read_dir(p)? {
        let e = e?;
        let m = e.metadata()?;
        let v = if m.is_dir() {
            disk(&e.path())?
        } else {
            (m.len(), m.blocks() * 512)
        };
        total.0 += v.0;
        total.1 += v.1;
    }
    Ok(total)
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() == 5 && args[1] == "workspace" {
        return workspace(
            Path::new(&args[2]),
            Path::new(&args[3]),
            args[4].to_str().ok_or("image")?,
        );
    }
    if args.len() == 4 && args[1] == "verify" {
        return verify(Path::new(&args[2]), Path::new(&args[3]));
    }
    assert_eq!(args.len(), 3);
    let source = Path::new(&args[1]);
    let root = Path::new(&args[2]);
    assert!(source.is_dir());
    assert!(!root.exists(), "output must be absent");
    assert!(
        !root
            .parent()
            .ok_or("output parent")?
            .canonicalize()?
            .starts_with(source.canonicalize()?),
        "output must be outside source"
    );
    let setup = Instant::now();
    std::fs::create_dir(root)?;
    let store = Arc::new(LayerStackStore::create(root.join("store.sqlite"))?);
    let client = Client::connect(store.clone())?;
    println!("baseline_store_bytes={:?}", disk(root)?);
    let name = EntityName::new("torch-venv")?;
    let source = LayerStackInitialization::Directory(source.to_path_buf());
    println!("setup_seconds={:.9}", setup.elapsed().as_secs_f64());
    let before = usage();
    let start = Instant::now();
    let result = client.initialize_layerstack(name, source)?;
    let elapsed = start.elapsed().as_secs_f64();
    let after = usage();
    println!("init_wall_seconds={elapsed:.9}");
    println!(
        "init_user_cpu_seconds={:.6}",
        seconds(after.ru_utime) - seconds(before.ru_utime)
    );
    println!(
        "init_system_cpu_seconds={:.6}",
        seconds(after.ru_stime) - seconds(before.ru_stime)
    );
    println!("process_peak_rss_bytes={}", after.ru_maxrss);
    println!("live_store_bytes={:?}", disk(root)?);
    println!("result={result:?}");
    println!(
        "import_receipts={:?}",
        store.take_layerstack_initialization_receipts()
    );
    let layer = store.layer(result.genesis_layer_id)?.unwrap();
    assert_eq!(
        client
            .query(Query::new(QueryKind::LayerStacks))?
            .items
            .len(),
        1
    );
    assert_eq!(client.query(Query::new(QueryKind::Layers))?.items.len(), 1);
    println!("verification=one_stack_one_genesis_layer");
    drop(client);
    drop(store);
    println!("closed_store_bytes={:?}", disk(root)?);
    let reopened = LayerStackStore::connect(root.join("store.sqlite"))?;
    assert_eq!(reopened.layer(result.genesis_layer_id)?.unwrap(), layer);
    println!("verification=persisted_genesis_layer_matches_after_reopen");
    Ok(())
}

fn verify(source: &Path, database: &Path) -> Result<(), Box<dyn std::error::Error>> {
    use layerfs_content::{filesystem, CanonicalPath};
    use std::io::{Read, Write};
    use std::os::unix::ffi::OsStrExt;
    struct CompareFile(std::fs::File, u64);
    impl Write for CompareFile {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            let mut expected = vec![0; bytes.len()];
            self.0.read_exact(&mut expected)?;
            if expected != bytes {
                return Err(std::io::Error::other("content mismatch"));
            }
            self.1 += bytes.len() as u64;
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let store = LayerStackStore::connect(database)?;
    let stack = store
        .layer_stack_by_name(&EntityName::new(
            std::env::var("VERIFY_STACK_NAME").unwrap_or_else(|_| "torch-venv".to_owned()),
        )?)?
        .unwrap();
    let layer = store.layer(stack.head_layer_id)?.unwrap();
    let snapshot = store.snapshot_reader(layer.root_id);
    let snapshot = if let Ok(branch) = std::env::var("VERIFY_BRANCH") {
        store.pin_branch(branch.parse()?)?.reader
    } else {
        snapshot
    };
    let root_id = snapshot.root();
    let reader = layerfs_layerstack_store::CoreReader(&snapshot);
    let mut directories = vec![std::path::PathBuf::new()];
    let (mut files, mut bytes, mut links, mut dirs) = (0, 0, 0, 0);
    while let Some(relative) = directories.pop() {
        let path = if relative.as_os_str().is_empty() {
            CanonicalPath::root()
        } else {
            CanonicalPath::new(relative.to_str().unwrap())?
        };
        let (directory_entry, _) = filesystem::stat(&reader, root_id, &path)?;
        verify_metadata(
            &reader,
            directory_entry,
            &std::fs::symlink_metadata(source.join(&relative))?,
        )?;
        let expected = std::fs::read_dir(source.join(&relative))?
            .map(|e| Ok(e?.file_name().as_bytes().to_vec()))
            .collect::<std::io::Result<std::collections::BTreeSet<_>>>()?;
        let mut actual = std::collections::BTreeSet::new();
        let mut after = None;
        loop {
            let (page, _) =
                filesystem::list(&reader, root_id, &path, after.as_ref(), 256, 1024 * 1024)?;
            actual.extend(
                page.entries
                    .iter()
                    .map(|(name, _)| name.as_bytes().to_vec()),
            );
            after = page.continuation;
            if after.is_none() {
                break;
            }
        }
        assert_eq!(
            actual,
            expected,
            "directory entries at {}",
            relative.display()
        );
        for name in expected {
            let child = relative.join(std::ffi::OsStr::from_bytes(&name));
            let native = source.join(&child);
            let metadata = native.symlink_metadata()?;
            let path = CanonicalPath::new(child.to_str().unwrap())?;
            let (entry, _) = filesystem::stat(&reader, root_id, &path)?;
            verify_metadata(&reader, entry, &metadata)?;
            if metadata.is_dir() {
                assert_eq!(
                    entry.kind,
                    layerfs_content::tree::inode::InodeKind::Directory
                );
                directories.push(child);
                dirs += 1;
            } else if metadata.file_type().is_symlink() {
                let (target, _) = filesystem::readlink(&reader, root_id, &path)?;
                assert_eq!(target, std::fs::read_link(native)?.as_os_str().as_bytes());
                links += 1;
            } else {
                let mut sink = CompareFile(std::fs::File::open(native)?, 0);
                filesystem::stream(&reader, root_id, &path, &mut sink)?;
                assert_eq!(
                    sink.0.read(&mut [0])?,
                    0,
                    "truncated file {}",
                    child.display()
                );
                files += 1;
                bytes += sink.1;
            }
        }
    }
    println!("full_verification files={files} bytes={bytes} symlinks={links} directories={dirs}");
    Ok(())
}

fn verify_metadata(
    reader: &impl layerfs_content::object::access::ObjectRead,
    entry: layerfs_content::filesystem::Stat,
    native: &std::fs::Metadata,
) -> Result<(), Box<dyn std::error::Error>> {
    use layerfs_content::{
        file::rope::{read_all_bounded, FileStateRoot},
        tree::metadata::{metadata_lookup, MetadataKey},
    };
    let read = |name: &[u8], limit| -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let value = metadata_lookup(
            reader,
            entry.metadata_root,
            &MetadataKey::new("portable".into(), name.to_vec())?,
        )?
        .ok_or("missing metadata")?;
        let mut bytes = Vec::new();
        read_all_bounded(
            reader,
            FileStateRoot(value.value_file_root),
            limit,
            &mut bytes,
        )?;
        Ok(bytes)
    };
    let expected_mode = if native.file_type().is_symlink() {
        0o777
    } else {
        native.mode() & if native.is_dir() { 0o1777 } else { 0o777 }
    };
    assert_eq!(read(b"mode", 4)?, expected_mode.to_be_bytes());
    let mut mtime = native.mtime().to_be_bytes().to_vec();
    mtime.extend_from_slice(&(native.mtime_nsec() as u32).to_be_bytes());
    assert_eq!(read(b"mtime", 12)?, mtime);
    Ok(())
}

fn workspace(archive: &Path, root: &Path, image: &str) -> Result<(), Box<dyn std::error::Error>> {
    use layerfs_sdk::{
        ContainerCreate, ContainerLimits, ContainerManager, CreateWorkspaceSession,
        EndWorkspaceMode, LocalForkSource, NonEmpty, WorkspaceCommitResult, WorkspacePlacement,
        WorkspaceProjection,
    };
    use std::{ffi::OsString, process::Command};
    assert!(archive.is_file());
    assert!(!root.exists(), "output must be absent");
    let setup = Instant::now();
    std::fs::create_dir(root)?;
    let manager = ContainerManager::open(root.join("containers"))?;
    let name = format!(
        "layerfs-issue71-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    );
    println!("container={name}");
    manager.create(ContainerCreate {
        name: name.clone(),
        image: image.to_owned(),
        limits: ContainerLimits {
            memory_bytes: 2 * 1024 * 1024 * 1024,
            cpus: 2,
            pids: 256,
        },
    })?;
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(
        || -> Result<(), Box<dyn std::error::Error>> {
            assert!(Command::new("docker")
                .args(["update", "--memory-swap", "2147483648", &name])
                .status()?
                .success());
            let running = manager.start(&name)?;
            let status = manager.status(&name)?;
            assert!(
                status.running
                    && status.fuse_device
                    && status.sys_admin
                    && !status.privileged
                    && status.host_binds == 0
            );
            let delivery = Instant::now();
            assert!(Command::new("docker")
                .args(["cp"])
                .arg(archive)
                .arg(format!("{name}:/input.tar"))
                .status()?
                .success());
            println!(
                "archive_delivery_seconds={:.9}",
                delivery.elapsed().as_secs_f64()
            );
            let store = Arc::new(LayerStackStore::create(root.join("store.sqlite"))?);
            let client = Client::connect_with_container(store.clone(), running.binding())?;
            let initialized = client.initialize_layerstack(
                EntityName::new("torch-venv")?,
                LayerStackInitialization::Empty,
            )?;
            let branch = client.fork_branch(
                EntityName::new("main")?,
                LocalForkSource::Layer {
                    layer_id: initialized.genesis_layer_id,
                },
            )?;
            println!("setup_seconds={:.9}", setup.elapsed().as_secs_f64());
            let lifecycle = Instant::now();
            let created = Instant::now();
            let session = client.create_workspace_session(CreateWorkspaceSession {
                branch_id: branch,
                placement: WorkspacePlacement::Container {
                    container_id: running.id.clone(),
                    root: "/workspace/project".into(),
                },
                projection: Some(WorkspaceProjection::Fuse),
            })?;
            println!("create_seconds={:.9}", created.elapsed().as_secs_f64());
            let before = usage();
            let exec_started = Instant::now();
            let execution = client.exec_workspace_session(
                session.id,
                NonEmpty::new(vec![
                    OsString::from("tar"),
                    OsString::from("--blocking-factor=2048"),
                    OsString::from("--no-same-owner"),
                    OsString::from("-xpf"),
                    OsString::from("/input.tar"),
                    OsString::from("-C"),
                    OsString::from("/workspace/project"),
                ])?,
            )?;
            let reader = client.workspace_output(execution.id)?;
            let mut after = 0;
            loop {
                let page = reader.read(after, true)?;
                assert!(!page.truncated, "execution output was truncated");
                for chunk in &page.chunks {
                    std::io::Write::write_all(&mut std::io::stderr(), &chunk.bytes)?;
                }
                if page.exited {
                    let receipt = page.receipt.ok_or("missing execution receipt")?;
                    assert_eq!(receipt.exit_code, Some(0));
                    assert_eq!(receipt.transport, layerfs_sdk::ExecutionTransport::Daemon);
                    break;
                }
                if exec_started.elapsed() > std::time::Duration::from_secs(300) {
                    return Err("Exec deadline".into());
                }
                after = page.next_sequence;
            }
            let exec = exec_started.elapsed().as_secs_f64();
            println!("exec_seconds={exec:.9}");
            let commit_started = Instant::now();
            let commit = client.commit_workspace_session_with_status(session.id)?;
            assert!(matches!(
                commit.result,
                WorkspaceCommitResult::Created { .. }
            ));
            assert!(
                !commit.presentation_failed,
                "committed data but failed live checkpoint"
            );
            let commit_seconds = commit_started.elapsed().as_secs_f64();
            let after = usage();
            println!("commit_seconds={commit_seconds:.9}");
            println!("exec_commit_seconds={:.9}", exec + commit_seconds);
            println!(
                "host_exec_commit_user_cpu_seconds={:.6}",
                seconds(after.ru_utime) - seconds(before.ru_utime)
            );
            println!(
                "host_exec_commit_system_cpu_seconds={:.6}",
                seconds(after.ru_stime) - seconds(before.ru_stime)
            );
            println!("process_peak_rss_bytes={}", after.ru_maxrss);
            let resources = Command::new("docker").args(["exec", &name, "/bin/sh", "-c", "cat /sys/fs/cgroup/memory.peak /sys/fs/cgroup/memory.swap.current /sys/fs/cgroup/memory.events"]).output()?;
            assert!(resources.status.success());
            let resources = String::from_utf8(resources.stdout)?;
            println!("container_memory_resources={resources:?}");
            let mut lines = resources.lines();
            assert!(lines.next().ok_or("memory peak")?.parse::<u64>()? <= 2 * 1024 * 1024 * 1024);
            assert_eq!(lines.next(), Some("0"));
            assert!(resources.lines().any(|line| line == "oom 0"));
            assert!(resources.lines().any(|line| line == "oom_kill 0"));
            let pin = store.pin_branch(branch)?;
            println!("branch={branch}");
            println!("commit={:?}", pin.branch.head_commit_id);
            println!("root={:?}", pin.root);
            drop(pin);
            client.end_workspace_session(session.id, EndWorkspaceMode::Clean)?;
            assert_eq!(client.active_workspace_count()?, 0);
            assert_eq!(client.active_execution_count()?, 0);
            println!("lifecycle_seconds={:.9}", lifecycle.elapsed().as_secs_f64());
            drop(client);
            drop(store);
            Ok(())
        },
    ));
    let stop = manager.stop(&name);
    let remove = manager.remove(&name);
    if let Err(error) = &stop {
        eprintln!("container stop error: {error}");
    }
    if let Err(error) = &remove {
        eprintln!("container removal error: {error}");
    }
    match result {
        Ok(result) => result?,
        Err(panic) => std::panic::resume_unwind(panic),
    }
    stop?;
    remove?;
    println!("cleanup=removed_owned_container");
    Ok(())
}
