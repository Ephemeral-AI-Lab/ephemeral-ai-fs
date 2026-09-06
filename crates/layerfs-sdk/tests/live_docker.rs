use layerfs_sdk::{
    Client, ContainerCreate, ContainerLimits, ContainerManager, CreateWorkspaceSession,
    EndWorkspaceMode, EntityName, ExecutionId, ExecutionTransport, LayerStackInitialization,
    LayerStackStore, LocalForkSource, NonEmpty, OutputPage, SdkError, WorkspaceCommitResult,
    WorkspaceError, WorkspaceId, WorkspacePlacement, WorkspaceProjection,
};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::time::{Duration, Instant};

type AnyResult<T> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn managed_container_lifecycle_and_disconnect_cleanup_are_exact() {
    if std::env::var_os("LAYERFS_LIVE_DOCKER").is_none() {
        return;
    }
    let image = std::env::var("LAYERFS_LIVE_DOCKER_IMAGE")
        .expect("LAYERFS_LIVE_DOCKER_IMAGE must name a prepared LayerFS runtime image");
    let root = temp();
    let manager = ContainerManager::open(root.join("containers")).unwrap();
    let name = format!(
        "layerfs-live-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    let proof = managed_proof(&manager, &name, &image, &root);
    let cleanup = cleanup_container(&manager, &name);
    match (proof, cleanup) {
        (Ok(()), Ok(())) => std::fs::remove_dir_all(root).unwrap(),
        (Err(proof), Ok(())) => panic!("managed lifecycle proof failed: {proof}"),
        (Ok(()), Err(cleanup)) => panic!("managed container cleanup failed: {cleanup}"),
        (Err(proof), Err(cleanup)) => {
            panic!("managed lifecycle proof failed: {proof}; cleanup also failed: {cleanup}")
        }
    }
}

fn managed_proof(
    manager: &ContainerManager,
    name: &str,
    image: &str,
    root: &Path,
) -> AnyResult<()> {
    require(
        Command::new("docker")
            .arg("info")
            .output()?
            .status
            .success(),
        "docker info",
    )?;
    manager.create(ContainerCreate {
        name: name.to_owned(),
        image: image.to_owned(),
        limits: ContainerLimits {
            memory_bytes: 2 * 1024 * 1024 * 1024,
            cpus: 2,
            pids: 256,
        },
    })?;
    let running = manager.start(name)?;
    let status = manager.status(name)?;
    require(
        status.running
            && !status.privileged
            && status.fuse_device
            && status.sys_admin
            && status.host_binds == 0,
        "managed container isolation",
    )?;

    let store_path = root.join("store.sqlite");
    let store = Arc::new(LayerStackStore::create(&store_path)?);
    let client = Client::connect_with_container(store.clone(), running.binding())?;
    let initialized = client
        .initialize_layerstack(EntityName::new("project")?, LayerStackInitialization::Empty)?;
    let attachment_branch = client.fork_branch(
        EntityName::new("attachment-failure")?,
        LocalForkSource::Layer {
            layer_id: initialized.genesis_layer_id,
        },
    )?;
    let attachment_root = format!("/workspace/layerfs-live-{}-attachment", std::process::id());
    let previous_attachment_failure =
        std::env::var_os("LAYERFS_WORKSPACE_INJECT_POST_ATTACH_FAILURE");
    std::env::set_var("LAYERFS_WORKSPACE_INJECT_POST_ATTACH_FAILURE", "1");
    let attachment = client.create_workspace_session(container_request(
        attachment_branch,
        &running.id,
        &attachment_root,
    ));
    match previous_attachment_failure {
        Some(value) => std::env::set_var("LAYERFS_WORKSPACE_INJECT_POST_ATTACH_FAILURE", value),
        None => std::env::remove_var("LAYERFS_WORKSPACE_INJECT_POST_ATTACH_FAILURE"),
    }
    require(
        matches!(
            attachment,
            Err(SdkError::Workspace(WorkspaceError::InvalidPlacement))
        ),
        "injected post-attachment failure",
    )?;
    require(
        client.active_workspace_count()? == 0,
        "post-attachment active Workspace cleanup",
    )?;
    wait_for(Duration::from_secs(5), || {
        container_clean(name, &attachment_root, "no-attachment-process").unwrap_or(false)
    })?;
    let lifecycle_branch = client.fork_branch(
        EntityName::new("lifecycle")?,
        LocalForkSource::Layer {
            layer_id: initialized.genesis_layer_id,
        },
    )?;
    let lifecycle_root = format!("/workspace/layerfs-live-{}-lifecycle", std::process::id());
    let lifecycle = client.create_workspace_session(container_request(
        lifecycle_branch,
        &running.id,
        &lifecycle_root,
    ))?;
    require(mounted(name, &lifecycle_root)?, "lifecycle FUSE mount")?;
    let (normal_execution, page) = execute(
        &client,
        lifecycle.id,
        ["/bin/sh", "-c", "printf managed > payload"],
    )?;
    require(
        page.receipt.as_ref().is_some_and(|receipt| {
            receipt.exit_code == Some(0) && receipt.transport == ExecutionTransport::Daemon
        }),
        "daemon execution receipt",
    )?;
    require(
        matches!(
            client.commit_workspace_session(lifecycle.id)?,
            WorkspaceCommitResult::Created { .. }
        ),
        "managed Commit",
    )?;
    client.end_workspace_session(lifecycle.id, EndWorkspaceMode::Clean)?;
    require(
        container_clean(name, &lifecycle_root, "no-lifecycle-process")?,
        "lifecycle cleanup",
    )?;

    let failure_branch = client.fork_branch(
        EntityName::new("disconnect")?,
        LocalForkSource::Layer {
            layer_id: initialized.genesis_layer_id,
        },
    )?;
    let failure_root = format!("/workspace/layerfs-live-{}-disconnect", std::process::id());
    let failure = client.create_workspace_session(container_request(
        failure_branch,
        &running.id,
        &failure_root,
    ))?;
    require(mounted(name, &failure_root)?, "disconnect FUSE mount")?;
    let spool = runtime_artifact("workspaces", &failure.id.to_string())?;
    require(spool.len() == 1, "Workspace spool evidence")?;
    let marker = format!("layerfs-disconnect-{}", failure.id);
    let previous = std::env::var_os("LAYERFS_EXEC_INJECT_DISCONNECT");
    std::env::set_var("LAYERFS_EXEC_INJECT_DISCONNECT", "1");
    let disconnected = execute_disconnected(&client, failure.id, &marker);
    match previous {
        Some(value) => std::env::set_var("LAYERFS_EXEC_INJECT_DISCONNECT", value),
        None => std::env::remove_var("LAYERFS_EXEC_INJECT_DISCONNECT"),
    }
    let (failed_execution, output_file) = disconnected?;
    wait_for(Duration::from_secs(5), || {
        matches!(client.active_execution_count(), Ok(0))
    })?;
    client.end_workspace_session(failure.id, EndWorkspaceMode::Discard)?;
    require(
        client.active_workspace_count()? == 0,
        "active Workspace cleanup",
    )?;
    require(!spool[0].exists(), "Workspace spool cleanup")?;
    wait_for(Duration::from_secs(5), || {
        container_clean(name, &failure_root, &marker).unwrap_or(false)
    })?;

    drop(client);
    require(!output_file.exists(), "output reader cleanup")?;
    let client = Client::connect_with_container(store.clone(), running.binding())?;
    let lease = client.create_workspace_session(container_request(
        failure_branch,
        &running.id,
        &failure_root,
    ))?;
    client.end_workspace_session(lease.id, EndWorkspaceMode::Discard)?;
    require(
        client.active_workspace_count()? == 0,
        "Branch lease release",
    )?;
    require(client.active_execution_count()? == 0, "execution cleanup")?;
    require(
        container_clean(name, &failure_root, &marker)?,
        "mount/process cleanup",
    )?;

    drop(client);
    drop(store);
    require(
        runtime_artifact("output", &format!("{normal_execution}.frames"))?.is_empty(),
        "successful output reader cleanup",
    )?;
    require(
        runtime_artifact("output", &format!("{failed_execution}.frames"))?.is_empty(),
        "failed output reader cleanup",
    )?;
    Ok(())
}

fn container_request(
    branch_id: layerfs_sdk::BranchId,
    container_id: &layerfs_sdk::ContainerId,
    root: &str,
) -> CreateWorkspaceSession {
    CreateWorkspaceSession {
        branch_id,
        placement: WorkspacePlacement::Container {
            container_id: container_id.clone(),
            root: PathBuf::from(root),
        },
        projection: Some(WorkspaceProjection::Fuse),
    }
}

fn execute<const N: usize>(
    client: &Client,
    workspace: WorkspaceId,
    argv: [&str; N],
) -> AnyResult<(ExecutionId, OutputPage)> {
    let execution = client.exec_workspace_session(
        workspace,
        NonEmpty::new(argv.into_iter().map(OsString::from).collect())?,
    )?;
    let reader = client.workspace_output(execution.id)?;
    let deadline = Instant::now() + Duration::from_secs(30);
    let mut after = 0;
    loop {
        let page = reader.read(after, true)?;
        if page.exited {
            return Ok((execution.id, page));
        }
        if Instant::now() >= deadline {
            return Err("execution output timeout".into());
        }
        after = page.next_sequence;
    }
}

fn execute_disconnected(
    client: &Client,
    workspace: WorkspaceId,
    marker: &str,
) -> AnyResult<(ExecutionId, PathBuf)> {
    let execution = client.exec_workspace_session(
        workspace,
        NonEmpty::new(vec![
            OsString::from("/bin/sh"),
            OsString::from("-c"),
            OsString::from("while :; do sleep 30; done"),
            OsString::from(marker),
        ])?,
    )?;
    let output = runtime_artifact("output", &format!("{}.frames", execution.id))?;
    require(output.len() == 1, "output spool evidence")?;
    let reader = client.workspace_output(execution.id)?;
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        match reader.read(0, true) {
            Err(WorkspaceError::InfrastructureLost) => {
                drop(reader);
                return Ok((execution.id, output[0].clone()));
            }
            Err(error) => return Err(format!("unexpected output failure: {error}").into()),
            Ok(_) if Instant::now() < deadline => {}
            Ok(_) => return Err("disconnect output timeout".into()),
        }
    }
}

fn runtime_artifact(directory: &str, name: &str) -> std::io::Result<Vec<PathBuf>> {
    let root = std::env::temp_dir().join("layerfs-runtime");
    if !root.is_dir() {
        return Ok(Vec::new());
    }
    Ok(std::fs::read_dir(root)?
        .filter_map(Result::ok)
        .map(|entry| entry.path().join(directory).join(name))
        .filter(|path| path.exists())
        .collect())
}

fn mounted(container: &str, root: &str) -> std::io::Result<bool> {
    docker_status(container, ["findmnt", "-rn", "-M", root])
}

fn container_clean(container: &str, root: &str, marker: &str) -> std::io::Result<bool> {
    let mount = mounted(container, root)?;
    let pid_output = Command::new("docker")
        .args([
            "exec",
            container,
            "find",
            "/tmp",
            "-maxdepth",
            "1",
            "-name",
            "layerfs-execution-*.pid",
            "-print",
        ])
        .output()?;
    let pid_files = pid_output.status.success() && pid_output.stdout.is_empty();
    let processes = Command::new("docker")
        .args(["top", container, "-eo", "pid,args"])
        .output()?;
    if !processes.status.success() {
        return Err(std::io::Error::other("docker top"));
    }
    let execution = pid_files
        && !String::from_utf8_lossy(&processes.stdout)
            .split_ascii_whitespace()
            .any(|argument| argument == marker);
    let helper = docker_status(
        container,
        [
            "/bin/sh",
            "-c",
            "for file in /proc/[0-9]*/comm; do [ \"$(cat \"$file\" 2>/dev/null)\" = layerfs-fuse ] && exit 1; done; exit 0",
        ],
    )?;
    if mount || !execution || !helper {
        eprintln!(
            "container cleanup residue: mount={mount} process_or_pid={} pid_status={} pid_files={:?} pid_error={:?} helper={} processes={:?}",
            !execution,
            pid_output.status,
            String::from_utf8_lossy(&pid_output.stdout),
            String::from_utf8_lossy(&pid_output.stderr),
            !helper,
            String::from_utf8_lossy(&processes.stdout),
        );
    }
    Ok(!mount && execution && helper)
}

fn docker_status<const N: usize>(container: &str, args: [&str; N]) -> std::io::Result<bool> {
    Ok(Command::new("docker")
        .arg("exec")
        .arg(container)
        .args(args)
        .output()?
        .status
        .success())
}

fn wait_for(timeout: Duration, mut condition: impl FnMut() -> bool) -> AnyResult<()> {
    let deadline = Instant::now() + timeout;
    while !condition() {
        if Instant::now() >= deadline {
            return Err("cleanup timeout".into());
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    Ok(())
}

fn cleanup_container(manager: &ContainerManager, name: &str) -> AnyResult<()> {
    let inspect = Command::new("docker")
        .args([
            "inspect",
            "-f",
            "{{index .Config.Labels \"dev.layerfs.managed\"}}",
            name,
        ])
        .output()?;
    if !inspect.status.success() {
        return Ok(());
    }
    require(
        String::from_utf8_lossy(&inspect.stdout).trim() == "true",
        "refusing to remove an unmanaged container",
    )?;
    manager.stop(name)?;
    manager.remove(name)?;
    require(
        !Command::new("docker")
            .args(["inspect", name])
            .output()?
            .status
            .success(),
        "container removal",
    )
}

fn require(condition: bool, message: &'static str) -> AnyResult<()> {
    if condition {
        Ok(())
    } else {
        Err(message.into())
    }
}

fn temp() -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "layerfs-sdk-v4-docker-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&path).unwrap();
    path
}

#[test]
fn running_commands_and_dirty_mappings_continue_across_commit() {
    run_live_cut(false);
}

#[test]
fn ordinary_writes_queue_during_mapped_commit() {
    run_live_cut(true);
}

fn run_live_cut(ordinary_writes: bool) {
    if std::env::var_os("LAYERFS_LIVE_DOCKER").is_none() {
        return;
    }
    let image = std::env::var("LAYERFS_LIVE_DOCKER_IMAGE").unwrap();
    let root = temp();
    let manager = ContainerManager::open(root.join("containers")).unwrap();
    let name = format!("layerfs-live-cut-{}", std::process::id());
    eprintln!("focused live cut container={name}");
    let result = live_cut_check(&manager, &name, &image, &root, ordinary_writes);
    let cleanup = cleanup_container(&manager, &name);
    match (result, cleanup) {
        (Ok(()), Ok(())) => std::fs::remove_dir_all(root).unwrap(),
        (result, cleanup) => panic!("live cut result={result:?}; cleanup={cleanup:?}"),
    }
}

fn live_cut_check(
    manager: &ContainerManager,
    name: &str,
    image: &str,
    root: &Path,
    ordinary_writes: bool,
) -> AnyResult<()> {
    use std::io::Write;
    manager.create(ContainerCreate {
        name: name.to_owned(),
        image: image.to_owned(),
        limits: ContainerLimits {
            memory_bytes: 2 * 1024 * 1024 * 1024,
            cpus: 2,
            pids: 256,
        },
    })?;
    let running = manager.start(name)?;
    // A focused ordinary mmap/fd client, compiled during test preparation.
    // It does not implement filesystem/storage behavior or benchmark mutation.
    let program = r#"
#include <assert.h>
#include <fcntl.h>
#include <stdio.h>
#include <string.h>
#include <sys/mman.h>
#include <sys/stat.h>
#include <time.h>
#include <unistd.h>
int main(int argc, char **argv) {
    assert(argc == 4);
    int fd = open(argv[1], O_RDWR); assert(fd >= 0);
    struct stat before, after; assert(fstat(fd, &before) == 0);
    char cwd[4096], later[4096]; assert(getcwd(cwd, sizeof cwd));
    assert(lseek(fd, 19, SEEK_SET) == 19);
    volatile unsigned char *p = mmap(0, 4096, PROT_READ | PROT_WRITE, MAP_SHARED, fd, 0);
    assert((void *)p != MAP_FAILED);
    p[0] = 'A'; p[4095] = 'B';
    puts("ready"); fflush(stdout);
    struct timespec delay = {0, 1000000};
    while (access(argv[2], F_OK) != 0) {
        p[128]++;
#ifdef ORDINARY_WRITES
        assert(pwrite(fd, "Q", 1, 129) == 1);
#else
        nanosleep(&delay, 0);
#endif
    }
    assert(p[0] == 'A' && p[4095] == 'B' && p[777] == 'S');
    assert(fstat(fd, &after) == 0 && before.st_ino == after.st_ino && before.st_dev == after.st_dev);
    assert(lseek(fd, 0, SEEK_CUR) == 19);
    assert(getcwd(later, sizeof later) && strcmp(cwd, later) == 0);
    p[0] = 'C'; p[4095] = 'D';
    assert(pwrite(fd, "F", 1, 2048) == 1);
    puts("after"); fflush(stdout);
    while (access(argv[3], F_OK) != 0) {
        p[128]++;
#ifdef ORDINARY_WRITES
        assert(pwrite(fd, "Q", 1, 129) == 1);
#else
        nanosleep(&delay, 0);
#endif
    }
    assert(p[0] == 'C' && p[4095] == 'D' && p[2048] == 'F');
    assert(munmap((void *)p, 4096) == 0); assert(close(fd) == 0);
    puts("done"); return 0;
}
"#;
    let mut compiler = Command::new("docker")
        .args([
            "exec",
            "-i",
            name,
            "cc",
            "-O2",
            "-x",
            "c",
            "-",
            "-o",
            "/var/tmp/layerfs-cut-check",
        ])
        .args(if ordinary_writes {
            vec!["-DORDINARY_WRITES"]
        } else {
            vec![]
        })
        .stdin(std::process::Stdio::piped())
        .spawn()?;
    compiler
        .stdin
        .take()
        .ok_or("compiler stdin")?
        .write_all(program.as_bytes())?;
    require(compiler.wait()?.success(), "compile focused mmap client")?;
    let store = Arc::new(LayerStackStore::create(root.join("store.sqlite"))?);
    let client = Client::connect_with_container(store.clone(), running.binding())?;
    let initialized =
        client.initialize_layerstack(EntityName::new("cut")?, LayerStackInitialization::Empty)?;
    let branch = client.fork_branch(
        EntityName::new("main")?,
        LocalForkSource::Layer {
            layer_id: initialized.genesis_layer_id,
        },
    )?;
    let placement = format!("/workspace/cut-{}", std::process::id());
    let session =
        client.create_workspace_session(container_request(branch, &running.id, &placement))?;
    let peer_branch = client.fork_branch(
        EntityName::new("peer")?,
        LocalForkSource::Layer {
            layer_id: initialized.genesis_layer_id,
        },
    )?;
    let peer = client.create_workspace_session(container_request(
        peer_branch,
        &running.id,
        &format!("{placement}-peer"),
    ))?;
    require(
        docker_status(
            name,
            ["/bin/sh", "-c", "! ps -e -o comm= | grep -qx layerfs-fuse"],
        )?,
        "daemon owns both mounts without helper processes",
    )?;
    let (_, prepared) = execute(&client, session.id, ["/bin/sh", "-c", "dd if=/dev/zero of=held-a bs=4096 count=1 2>/dev/null; dd if=/dev/zero of=held-b bs=4096 count=1 2>/dev/null"])?;
    require(
        prepared
            .receipt
            .is_some_and(|receipt| receipt.exit_code == Some(0)),
        "prepare mapped files",
    )?;
    let go = format!("/var/tmp/cut-{}-go", session.id);
    let done = format!("/var/tmp/cut-{}-done", session.id);
    let mut executions = Vec::new();
    for file in ["held-a", "held-b"] {
        let execution = client.exec_workspace_session(
            session.id,
            NonEmpty::new(vec![
                OsString::from("/var/tmp/layerfs-cut-check"),
                OsString::from(file),
                OsString::from(&go),
                OsString::from(&done),
            ])?,
        )?;
        executions.push(execution.id);
    }
    for id in &executions {
        wait_live_marker(&client, *id, "ready")?;
    }
    let (_, peer_write) = execute(&client, peer.id, ["/bin/sh", "-c", "set -e; printf peer > isolated; mkdir names; touch names/a; ln names/a names/b; mv names/b names/c; test $(ls names | wc -l) -eq 2; rm -r names"])?;
    require(
        peer_write
            .receipt
            .is_some_and(|receipt| receipt.exit_code == Some(0)),
        "peer namespace operations",
    )?;
    client.commit_workspace_session(peer.id)?;
    client.end_workspace_session(peer.id, EndWorkspaceMode::Clean)?;
    eprintln!("live-cut stage=peer-ended-with-primary-mappings-live");
    eprintln!("live-cut stage=first-commit");
    require(
        client.active_execution_count()? == 2,
        "two commands live before Commit",
    )?;
    require(
        matches!(
            client.commit_workspace_session(session.id)?,
            WorkspaceCommitResult::Created { .. }
        ),
        "Commit with running mappings",
    )?;
    require(
        client.active_execution_count()? == 2,
        "commands survive first Commit",
    )?;
    let first_root = store.pin_branch(branch)?.root;
    check_mapped_snapshot(&store, first_root, false)?;
    for path in ["held-a", "held-b"] {
        eprintln!("live-cut stage=sdk-edit path={path}");
        let edit = layerfs_sdk::WorkspaceFileRangeEdit {
            workspace_id: session.id,
            path: path.into(),
            start: 777,
            delete_len: 1,
            replacement: layerfs_sdk::WorkspaceFileReplacement::Inline(vec![b'S']),
        };
        if path == "held-a" {
            let rendezvous = std::sync::Barrier::new(2);
            std::thread::scope(|scope| -> AnyResult<()> {
                let committing = scope.spawn(|| {
                    rendezvous.wait();
                    client.commit_workspace_session(session.id)
                });
                rendezvous.wait();
                let edited = client.edit_workspace_file_range(edit);
                committing
                    .join()
                    .map_err(|_| "concurrent Commit panicked")??;
                edited?;
                Ok(())
            })?;
        } else {
            client.edit_workspace_file_range(edit)?;
        }
    }
    require(
        docker_status(name, ["touch", go.as_str()])?,
        "release after-cut writes",
    )?;
    eprintln!("live-cut stage=second-commit");
    for id in &executions {
        wait_live_marker(&client, *id, "after")?;
    }
    require(
        matches!(
            client.commit_workspace_session(session.id)?,
            WorkspaceCommitResult::Created { .. }
        ),
        "second live Commit",
    )?;
    require(
        client.active_execution_count()? == 2,
        "commands survive second Commit",
    )?;
    check_mapped_snapshot(&store, store.pin_branch(branch)?.root, true)?;
    check_mapped_snapshot(&store, first_root, false)?;
    require(
        docker_status(name, ["touch", done.as_str()])?,
        "release retained handles",
    )?;
    for id in executions {
        wait_for(Duration::from_secs(5), || {
            client
                .workspace_output(id)
                .is_ok_and(|reader| reader.read(0, false).is_ok_and(|page| page.exited))
        })?;
        let page = client.workspace_output(id)?.read(0, false)?;
        require(
            page.receipt
                .is_some_and(|receipt| receipt.exit_code == Some(0)),
            "mapping/fd/cwd client assertions",
        )?;
    }
    // The continuously updated counter can be dirty after the second cut.
    client.commit_workspace_session(session.id)?;
    client.end_workspace_session(session.id, EndWorkspaceMode::Clean)?;
    require(!mounted(name, &placement)?, "same mount eventually cleaned")?;
    Ok(())
}

fn wait_live_marker(client: &Client, id: ExecutionId, marker: &str) -> AnyResult<()> {
    let reader = client.workspace_output(id)?;
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let page = reader.read(0, false)?;
        let bytes: Vec<u8> = page
            .chunks
            .into_iter()
            .flat_map(|chunk| chunk.bytes)
            .collect();
        if String::from_utf8_lossy(&bytes).contains(marker) {
            return Ok(());
        }
        if page.exited || Instant::now() >= deadline {
            return Err(format!(
                "live client marker {marker}: {}",
                String::from_utf8_lossy(&bytes)
            )
            .into());
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn check_mapped_snapshot(
    store: &LayerStackStore,
    root: layerfs_content::ObjectId,
    later: bool,
) -> AnyResult<()> {
    let reader = store.snapshot_reader(root);
    for path in ["held-a", "held-b"] {
        let mut bytes = Vec::new();
        layerfs_content::filesystem::read_range(
            &layerfs_layerstack_store::CoreReader(&reader),
            root,
            &layerfs_content::CanonicalPath::new(path)?,
            0..4096,
            &mut bytes,
        )?;
        require(bytes.len() == 4096, "mapped snapshot length")?;
        for (index, byte) in bytes.into_iter().enumerate() {
            let expected = match index {
                777 if later => b'S',
                0 => {
                    if later {
                        b'C'
                    } else {
                        b'A'
                    }
                }
                4095 => {
                    if later {
                        b'D'
                    } else {
                        b'B'
                    }
                }
                2048 if later => b'F',
                128 | 129 => continue,
                _ => 0,
            };
            require(byte == expected, "mapped snapshot exact stable bytes")?;
        }
    }
    Ok(())
}
