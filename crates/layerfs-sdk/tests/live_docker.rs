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
    let (_, primed) = execute(
        &client,
        lifecycle.id,
        [
            "/bin/sh",
            "-c",
            r#"
set -eu
 printf managed > meta-payload
 test "$(stat -c '%s:%h' meta-payload)" = 7:1
 ln meta-payload meta-alias
 test "$(stat -c '%s:%h' meta-payload)" = 7:2
 test "$(stat -c '%s:%h' meta-alias)" = 7:2
 test "$(stat -c %i meta-payload)" = "$(stat -c %i meta-alias)"
"#,
        ],
    )?;
    require(
        primed.receipt.is_some_and(|r| r.exit_code == Some(0)),
        "prime mounted metadata and hardlink attrs",
    )?;
    client.edit_workspace_file_range(layerfs_sdk::WorkspaceFileRangeEdit {
        workspace_id: lifecycle.id,
        path: "meta-payload".into(),
        start: 7,
        delete_len: 0,
        replacement: layerfs_sdk::WorkspaceFileReplacement::Inline(b"-sdk".to_vec()),
    })?;
    let (_, checked) = execute(
        &client,
        lifecycle.id,
        [
            "/bin/sh",
            "-c",
            r#"
set -eu
 test "$(stat -c '%s:%h' meta-payload)" = 11:2
 test "$(stat -c '%s:%h' meta-alias)" = 11:2
 test "$(stat -c %i meta-payload)" = "$(stat -c %i meta-alias)"
 test "$(cat meta-payload)" = managed-sdk; test "$(cat meta-alias)" = managed-sdk
 mv meta-payload meta-moved; test ! -e meta-payload
 printf fresh > meta-payload
 test "$(stat -c %s meta-payload)" = 5; test "$(cat meta-payload)" = fresh
 test "$(stat -c '%s:%h' meta-moved)" = 11:2; test "$(stat -c '%s:%h' meta-alias)" = 11:2
 test "$(stat -c %i meta-payload)" != "$(stat -c %i meta-alias)"
 exec 3<meta-alias; rm meta-alias
 test ! -e meta-alias; test "$(stat -c '%s:%h' meta-moved)" = 11:1
 test "$(cat <&3)" = managed-sdk; exec 3<&-
 rm meta-payload meta-moved
 test "$(cat payload)" = managed
"#,
        ],
    )?;
    require(
        checked.receipt.is_some_and(|r| r.exit_code == Some(0)),
        "immediate SDK/FUSE metadata, meta-alias, rename and retained-fd coherence",
    )?;
    eprintln!("live-metadata stage=sdk-resize-alias-rename-recreate-open-unlink-PASS");

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

    // A fresh owner makes committed payload eligible for immutable cache prefill.
    let committed = store.pin_branch(lifecycle_branch)?.root;
    let cold = client.create_workspace_session(container_request(
        lifecycle_branch,
        &running.id,
        &lifecycle_root,
    ))?;
    let (_, cold_read) = execute(
        &client,
        cold.id,
        [
            "python3",
            "-c",
            r#"
import time
with open('payload', 'rb') as source:
    time.sleep(0.02)  # Let best-effort post-OPEN prefill run before this read.
    assert source.read() == b'managed'
"#,
        ],
    )?;
    require(
        cold_read.receipt.is_some_and(|r| r.exit_code == Some(0)),
        "cold committed read-only open and read",
    )?;
    client.edit_workspace_file_range(layerfs_sdk::WorkspaceFileRangeEdit {
        workspace_id: cold.id,
        path: "payload".into(),
        start: 7,
        delete_len: 0,
        replacement: layerfs_sdk::WorkspaceFileReplacement::Inline(b"-sdk".to_vec()),
    })?;
    let (_, mapped) = execute(
        &client,
        cold.id,
        [
            "python3",
            "-c",
            r#"
import mmap, os
with open('payload', 'r+b') as source:
    assert os.fstat(source.fileno()).st_size == 11
    assert source.read() == b'managed-sdk'
    with mmap.mmap(source.fileno(), 0) as view:
        assert view[:] == b'managed-sdk'
        view[0:1] = b'M'
        view.flush()
        assert view[:] == b'Managed-sdk'
    source.seek(0)
    assert source.read() == b'Managed-sdk'
"#,
        ],
    )?;
    require(
        mapped.receipt.is_some_and(|r| r.exit_code == Some(0)),
        "cold prefill eligibility followed by SDK resize and writable mmap",
    )?;
    client.end_workspace_session(cold.id, EndWorkspaceMode::Discard)?;
    require(
        store.pin_branch(lifecycle_branch)?.root == committed,
        "cold proof leaves publication unchanged",
    )?;
    require(
        container_clean(name, &lifecycle_root, "no-cold-process")?,
        "cold proof cleanup",
    )?;
    eprintln!("live-prefill stage=cold-ro-sdk-resize-rw-mmap-discard-PASS");

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

/// Pinned package workflow over the live daemon/FUSE surface: a clean install of
/// hash-pinned wheels into the workspace, then a representative pinned update,
/// each followed by Commit, reopen and content/metadata/module verification.
#[test]
fn pinned_package_install_and_update_survive_commit_and_reopen() {
    if std::env::var_os("LAYERFS_LIVE_DOCKER").is_none() {
        return;
    }
    let image = std::env::var("LAYERFS_LIVE_DOCKER_IMAGE").unwrap();
    let root = temp();
    let manager = ContainerManager::open(root.join("containers")).unwrap();
    let name = format!("layerfs-pkg-{}", std::process::id());
    eprintln!("pinned package container={name}");
    let result = pinned_package_workflow(&manager, &name, &image, &root);
    if result.is_err() {
        if let Ok(logs) = Command::new("docker").args(["logs", name.as_str()]).output() {
            eprintln!("daemon stdout: {}", String::from_utf8_lossy(&logs.stdout));
            eprintln!("daemon stderr: {}", String::from_utf8_lossy(&logs.stderr));
        }
    }
    let cleanup = cleanup_container(&manager, &name);
    match (result, cleanup) {
        (Ok(()), Ok(())) => std::fs::remove_dir_all(root).unwrap(),
        (result, cleanup) => panic!("pinned package result={result:?}; cleanup={cleanup:?}"),
    }
}

/// Pinned pip bootstrap and the two hash-pinned package sets. Acquisition is a
/// separate, untimed setup phase: the artifacts are fetched once into the
/// container, verified by SHA256, and never mixed into product timing.
const PINNED_PIP: &str = "https://files.pythonhosted.org/packages/8a/6a/19e9fe04fca059ccf770861c7d5721ab4c2aebc539889e97c7977528a53b/pip-24.0-py3-none-any.whl";
const PINNED_PIP_SHA256: &str = "ba0d021a166865d2265246961bec0152ff124de910c5cc39f1156ce3fa7c69dc";
const PINNED_PROJECT: &str = r#"import idna
import packaging.version
import six

def versions():
    return (idna.__version__, packaging.__version__, six.__version__)

def normalize(value):
    return idna.encode(value).decode("ascii")

if __name__ == "__main__":
    print("application", *versions(), normalize("bücher.example"))
"#;
const PINNED_SET_A: &str = "\
--require-hashes
idna==3.6 --hash=sha256:c05567e9c24a6b9faaa835c4821bad0590fbb9d5779e7caa6e1cc4978e7eb24f
packaging==23.2 --hash=sha256:8c491190033a9af7e1d931d0b5dacc2ef47509b34dd0de67ed209b5203fc88c7
six==1.16.0 --hash=sha256:8abb2f1d86890a2dfb989f9a77cfcfd3e47c2a354b01111771326f8aa26e0254
";
const PINNED_SET_B: &str = "\
--require-hashes
idna==3.7 --hash=sha256:82fee1fc78add43492d3a1898bfa6d8a904cc97d8427f683ed8e798d07761aa0
packaging==24.0 --hash=sha256:2ddfb553fdf02fb784c234c7ba6ccc288296ceabec964ad2eae3777778130bc5
six==1.17.0 --hash=sha256:4721f391ed90541fddacab5acf947aa0d3dc7d27b2e1e8eda2be8970586c3274
";

/// Run one shell command in the session and return its captured output; a
/// non-zero exit fails with the exact command output for diagnosis.
fn run_package_command(
    client: &Client,
    session: WorkspaceId,
    body: &str,
    label: &'static str,
) -> AnyResult<String> {
    let execution = client.exec_workspace_session(
        session,
        NonEmpty::new(vec![
            OsString::from("/bin/sh"),
            OsString::from("-c"),
            OsString::from(body),
        ])?,
    )?;
    let reader = client.workspace_output(execution.id)?;
    let deadline = Instant::now() + Duration::from_secs(120);
    let mut after = 0;
    let mut collected = Vec::new();
    let receipt = loop {
        let page = reader.read(after, true)?;
        for chunk in &page.chunks {
            collected.extend_from_slice(&chunk.bytes);
        }
        if page.exited {
            break page.receipt;
        }
        if Instant::now() >= deadline {
            return Err("pinned package command timeout".into());
        }
        after = page.next_sequence;
    };
    let text = String::from_utf8_lossy(&collected).into_owned();
    require(
        receipt.is_some_and(|receipt| receipt.exit_code == Some(0)),
        label,
    )
    .map_err(|error| -> Box<dyn std::error::Error> {
        format!("{error}: {}", text.trim()).into()
    })?;
    Ok(text)
}

/// One shell-safe single-quoted word, so exact fixture bytes reach the mount.
fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

/// Container-side probe: run the application through the installed vendor tree,
/// then hash the installed module inventory for comparison across reopen.
fn package_probe(expected: &str) -> String {
    format!(
        r#"import hashlib, pathlib, subprocess, sys
out = subprocess.run([sys.executable, 'app.py'], capture_output=True, text=True, env={{'PYTHONPATH': 'vendor'}})
print('module-run', out.returncode, out.stdout.strip(), out.stderr.strip())
assert out.returncode == 0, out.stderr
assert out.stdout.startswith('application {expected} '), out.stdout
root = pathlib.Path('vendor')
files = sorted(str(p.relative_to(root)) for p in root.rglob('*.py'))
digest = hashlib.sha256()
for name in files:
    digest.update(name.encode())
    digest.update((root / name).read_bytes())
print('module-files', len(files), digest.hexdigest())
"#
    )
}

fn pinned_package_workflow(
    manager: &ContainerManager,
    name: &str,
    image: &str,
    root: &Path,
) -> AnyResult<()> {
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
    // Untimed setup: pinned pip wheel, verified by SHA256, then a local pip
    // bootstrap outside the workspace. Wheels only, so no build backend and no
    // package lifecycle script ever executes.
    let bootstrap = format!(
        "set -e; cd /var/tmp; \
         if [ ! -f pip.whl ]; then curl -fsSL '{PINNED_PIP}' -o pip.whl; fi; \
         echo '{PINNED_PIP_SHA256}  pip.whl' > pip.sha256; sha256sum -c pip.sha256; \
         python3 pip.whl/pip --version"
    );
    let boot = Command::new("docker")
        .args(["exec", name, "/bin/sh", "-c", bootstrap.as_str()])
        .output()?;
    require(
        boot.status.success(),
        "pinned pip bootstrap outside the workspace",
    )?;
    eprintln!(
        "pinned package stage=setup pip={}",
        String::from_utf8_lossy(&boot.stdout).trim()
    );
    let store = Arc::new(LayerStackStore::create(root.join("store.sqlite"))?);
    let client = Client::connect_with_container(store.clone(), running.binding())?;
    let initialized = client.initialize_layerstack(
        EntityName::new("pinned-package")?,
        LayerStackInitialization::Empty,
    )?;
    let branch = client.fork_branch(
        EntityName::new("main")?,
        LocalForkSource::Layer {
            layer_id: initialized.genesis_layer_id,
        },
    )?;
    let placement = format!("/workspace/pinned-{}", std::process::id());
    let mut session =
        client.create_workspace_session(container_request(branch, &running.id, &placement))?;
    // Single-quoted payloads keep every fixture byte exact without heredocs or
    // command substitution, and write ordinary files through the mount.
    let write = |path: &str, body: &str| -> String {
        format!("printf '%s' {} > {}", shell_quote(body), shell_quote(path))
    };
    run_package_command(
        &client,
        session.id,
        &format!(
            "set -e; mkdir -p project; {}; {}",
            write("project/app.py", PINNED_PROJECT),
            write("project/requirements.txt", PINNED_SET_A)
        ),
        "pinned project fixture",
    )?;
    let install = |session: WorkspaceId, requirements: &str, extra: &str| -> AnyResult<()> {
        let body = format!(
            "set -e; cd project; {}; \
             python3 /var/tmp/pip.whl/pip install {extra} --require-hashes --only-binary=:all: \
             --no-input --disable-pip-version-check --target vendor -r requirements.txt",
            write("requirements.txt", requirements)
        );
        let output = run_package_command(&client, session, &body, "pinned package installation")?;
        eprintln!("pinned package stage=install output={}", output.trim());
        Ok(())
    };
    let verify = |session: WorkspaceId, label: &str, expected: &str| -> AnyResult<()> {
        let probe = package_probe(expected);
        let body = format!(
            "set -e; cd project; {}; python3 /var/tmp/package-probe.py",
            write("/var/tmp/package-probe.py", &probe)
        );
        let text = run_package_command(
            &client,
            session,
            &body,
            "pinned module and application verification",
        )?;
        eprintln!("pinned package stage=verify label={label} output={}", text.trim());
        Ok(())
    };
    let commit = |session: WorkspaceId| -> AnyResult<()> {
        require(
            matches!(
                client.commit_workspace_session(session)?,
                WorkspaceCommitResult::Created { .. }
            ),
            "pinned package Commit",
        )?;
        Ok(())
    };
    // Clean installation of the pinned set, then Commit, reopen and verify.
    install(session.id, PINNED_SET_A, "--no-cache-dir")?;
    verify(session.id, "installed-in-session", "3.6")?;
    commit(session.id)?;
    let installed_root = store.pin_branch(branch)?.root;
    client.end_workspace_session(session.id, EndWorkspaceMode::Clean)?;
    session = client.create_workspace_session(container_request(branch, &running.id, &placement))?;
    verify(session.id, "installed-after-reopen", "3.6")?;
    verify_package_snapshot(&store, installed_root, "3.6")?;
    // Representative pinned update, then Commit, reopen and verify again.
    install(session.id, PINNED_SET_B, "--upgrade --no-cache-dir")?;
    verify(session.id, "updated-in-session", "3.7")?;
    commit(session.id)?;
    let updated_root = store.pin_branch(branch)?.root;
    require(updated_root != installed_root, "pinned update changed the root")?;
    client.end_workspace_session(session.id, EndWorkspaceMode::Clean)?;
    session = client.create_workspace_session(container_request(branch, &running.id, &placement))?;
    verify(session.id, "updated-after-reopen", "3.7")?;
    verify_package_snapshot(&store, updated_root, "3.7")?;
    client.end_workspace_session(session.id, EndWorkspaceMode::Clean)?;
    Ok(())
}

/// One bounded page of a committed directory, by name, from the frozen Store.
fn committed_directory_names(
    store: &LayerStackStore,
    root: layerfs_content::ObjectId,
    path: &layerfs_content::CanonicalPath,
) -> AnyResult<Vec<String>> {
    let reader = store.snapshot_reader(root);
    let (page, _) = layerfs_content::filesystem::list(
        &layerfs_layerstack_store::CoreReader(&reader),
        root,
        path,
        None,
        4096,
        1024 * 1024,
    )?;
    Ok(page
        .entries
        .iter()
        .map(|(name, _)| String::from_utf8_lossy(name.as_bytes()).into_owned())
        .collect())
}

/// Host-side, container-independent verification of the committed package tree:
/// exact application module bytes, installed module inventory and metadata.
fn verify_package_snapshot(
    store: &LayerStackStore,
    root: layerfs_content::ObjectId,
    expected_versions: &str,
) -> AnyResult<()> {
    let reader = store.snapshot_reader(root);
    let path = layerfs_content::CanonicalPath::new("project/app.py")?;
    let (stat, _) = layerfs_content::filesystem::stat(
        &layerfs_layerstack_store::CoreReader(&reader),
        root,
        &path,
    )?;
    let length = layerfs_content::file::content::length(
        &layerfs_layerstack_store::CoreReader(&reader),
        layerfs_content::file::content::FileContentRoot(stat.content_root),
    )?;
    let mut bytes = Vec::new();
    layerfs_content::filesystem::read_range(
        &layerfs_layerstack_store::CoreReader(&reader),
        root,
        &path,
        0..length,
        &mut bytes,
    )?;
    let text = String::from_utf8(bytes)?;
    require(
        text == PINNED_PROJECT,
        "committed application bytes are exact",
    )?;
    require(
        matches!(stat.kind, layerfs_content::tree::inode::InodeKind::RegularFile),
        "committed module kind",
    )?;
    let vendor = layerfs_content::CanonicalPath::new("project/vendor")?;
    let names = committed_directory_names(&store, root, &vendor)?;
    for expected in ["idna", "packaging", "six"] {
        require(
            names.iter().any(|name| name.starts_with(expected)),
            "installed distribution present in the committed tree",
        )
        .map_err(|error| -> Box<dyn std::error::Error> {
            format!("{error}: {names:?}").into()
        })?;
    }
    eprintln!(
        "pinned package stage=committed-snapshot versions={expected_versions} entries={} bytes={} refs={}",
        names.len(),
        length,
        stat.namespace_ref_count
    );
    Ok(())
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
    if result.is_err() {
        if let Ok(logs) = Command::new("docker")
            .args(["logs", name.as_str()])
            .output()
        {
            eprintln!("daemon stdout: {}", String::from_utf8_lossy(&logs.stdout));
            eprintln!("daemon stderr: {}", String::from_utf8_lossy(&logs.stderr));
        }
    }
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
    /* Keep this coherence check inside the existing 4096-edit generation budget. */
    struct timespec delay = {0, 1000000};
    struct timespec phase_start, phase_end;
    unsigned long spins = 0, barrier_spins = 0;
    assert(clock_gettime(CLOCK_MONOTONIC, &phase_start) == 0);
    while (access(argv[2], F_OK) != 0) {
        p[128]++;
#ifdef ORDINARY_WRITES
        assert(pwrite(fd, "Q", 1, 129) == 1);
#endif
        nanosleep(&delay, 0);
        spins++;
    }
    assert(clock_gettime(CLOCK_MONOTONIC, &phase_end) == 0);
    double pre_seconds = (phase_end.tv_sec - phase_start.tv_sec)
        + (phase_end.tv_nsec - phase_start.tv_nsec) / 1e9;
    if (p[0] != 'A' || p[4095] != 'B' || p[777] != 'S') {
        unsigned char ordinary[4096] = {0};
        ssize_t count = pread(fd, ordinary, sizeof ordinary, 0);
        fprintf(stderr, "after SDK boundary path=%s mapped[0,4095,777]=%u,%u,%u pread_count=%zd pread[0,4095,777]=%u,%u,%u expected=65,66,83\n",
                argv[1], p[0], p[4095], p[777], count, ordinary[0], ordinary[4095], ordinary[777]);
    }
    assert(p[0] == 'A' && p[4095] == 'B' && p[777] == 'S');
    assert(fstat(fd, &after) == 0 && before.st_ino == after.st_ino && before.st_dev == after.st_dev);
    assert(lseek(fd, 0, SEEK_CUR) == 19);
    assert(getcwd(later, sizeof later) && strcmp(cwd, later) == 0);
    p[0] = 'C'; p[4095] = 'D';
    assert(pwrite(fd, "F", 1, 2048) == 1);
    puts("after"); fflush(stdout);
    assert(clock_gettime(CLOCK_MONOTONIC, &phase_start) == 0);
    while (access(argv[3], F_OK) != 0) {
        p[128]++;
#ifdef ORDINARY_WRITES
        assert(pwrite(fd, "Q", 1, 129) == 1);
#endif
        nanosleep(&delay, 0);
        barrier_spins++;
    }
    assert(clock_gettime(CLOCK_MONOTONIC, &phase_end) == 0);
    double barrier_seconds = (phase_end.tv_sec - phase_start.tv_sec)
        + (phase_end.tv_nsec - phase_start.tv_nsec) / 1e9;
    assert(p[0] == 'C' && p[4095] == 'D' && p[2048] == 'F');
    printf("live %s pre_spins=%lu pre_seconds=%.6f barrier_spins=%lu barrier_seconds=%.6f\n",
           argv[1], spins, pre_seconds, barrier_spins, barrier_seconds);
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
    let mut edit_ns = 0_u128;
    for path in ["held-a", "held-b"] {
        eprintln!("live-cut stage=sdk-edit path={path}");
        let edit = layerfs_sdk::WorkspaceFileRangeEdit {
            workspace_id: session.id,
            path: path.into(),
            start: 777,
            delete_len: 1,
            replacement: layerfs_sdk::WorkspaceFileReplacement::Inline(vec![b'S']),
        };
        let edit_started = Instant::now();
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
        edit_ns += edit_started.elapsed().as_nanos();
    }
    require(
        docker_status(name, ["touch", go.as_str()])?,
        "release after-cut writes",
    )?;
    eprintln!("live-cut stage=second-commit");
    for id in &executions {
        if let Err(error) = wait_live_marker(&client, *id, "after") {
            let diagnostic_commit = client.commit_workspace_session(session.id);
            eprintln!("after failed boundary diagnostic Commit: {diagnostic_commit:?}");
            if diagnostic_commit.is_ok() {
                let root = store.pin_branch(branch)?.root;
                let reader = store.snapshot_reader(root);
                for path in ["held-a", "held-b"] {
                    let mut bytes = Vec::new();
                    let read = layerfs_content::filesystem::read_range(
                        &layerfs_layerstack_store::CoreReader(&reader),
                        root,
                        &layerfs_content::CanonicalPath::new(path)?,
                        777..778,
                        &mut bytes,
                    );
                    eprintln!("after failed boundary committed path={path} byte777={bytes:?} read={read:?}");
                }
            }
            return Err(error);
        }
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
    let mut live_measurements = Vec::new();
    for id in &executions {
        wait_for(Duration::from_secs(5), || {
            client
                .workspace_output(*id)
                .is_ok_and(|reader| reader.read(0, false).is_ok_and(|page| page.exited))
        })?;
        let page = client.workspace_output(*id)?.read(0, false)?;
        require(
            page.receipt
                .is_some_and(|receipt| receipt.exit_code == Some(0)),
            "mapping/fd/cwd client assertions",
        )?;
        let sample = page
            .chunks
            .iter()
            .flat_map(|chunk| String::from_utf8_lossy(&chunk.bytes).into_owned().into_bytes())
            .collect::<Vec<u8>>();
        let text = String::from_utf8_lossy(&sample).into_owned();
        if let Some(line) = text.lines().find(|line| line.starts_with("live ")) {
            live_measurements.push(line.to_owned());
        }
    }
    require(
        live_measurements.len() == 2,
        "both running commands reported barrier throughput",
    )?;
    eprintln!("live-cut edit_ns={edit_ns} measurements={live_measurements:?}");
    // Compact pending form and its conversion on the live path, with the two
    // commands still owning their mappings, descriptors and directories.
    live_compact_and_conversion_proof(&client, session.id)?;
    // The continuously updated counter can be dirty after the second cut.
    client.commit_workspace_session(session.id)?;
    client.end_workspace_session(session.id, EndWorkspaceMode::Clean)?;
    require(!mounted(name, &placement)?, "same mount eventually cleaned")?;
    Ok(())
}

/// Bounded live proof of the compact pending form and its promotion path:
/// one equal-length overwrite of a committed file charges exactly one compact
/// descriptor, and a second edit of the same file converts it to the rooted
/// three-node representation with the same exact contents.
fn live_compact_and_conversion_proof(
    client: &Client,
    session: layerfs_sdk::WorkspaceId,
) -> AnyResult<()> {
    let _commit_diagnostics = layerfs_sdk::capture_workspace_commit_diagnostics()?;
    let (_, prepared) = execute(
        client,
        session,
        ["/bin/sh", "-c", "set -e; printf 0123456789abcdef > compact"],
    )?;
    require(
        prepared
            .receipt
            .is_some_and(|receipt| receipt.exit_code == Some(0)),
        "compact proof fixture",
    )?;
    require(
        matches!(
            client.commit_workspace_session(session)?,
            WorkspaceCommitResult::Created { .. }
        ),
        "compact proof base Commit",
    )?;
    layerfs_layerstack_store::take_workspace_commit_diagnostics();
    // One equal-length overwrite of the committed base: the bounded form.
    client.edit_workspace_file_range(layerfs_sdk::WorkspaceFileRangeEdit {
        workspace_id: session,
        path: "compact".into(),
        start: 4,
        delete_len: 2,
        replacement: layerfs_sdk::WorkspaceFileReplacement::Inline(b"XY".to_vec()),
    })?;
    require(
        matches!(
            client.commit_workspace_session(session)?,
            WorkspaceCommitResult::Created { .. }
        ),
        "compact proof single-splice Commit",
    )?;
    let bounded = layerfs_layerstack_store::take_workspace_commit_diagnostics();
    let bounded = bounded.last().ok_or("compact Commit diagnostics")?;
    require(bounded.edit_count == 1, "one compact changed file")?;
    require(
        bounded.edit_piece_logical_charge == 64,
        "one compact descriptor is charged instead of three nodes",
    )?;
    require(
        bounded.edit_piece_count >= 2,
        "the bounded form still reports its base and splice pieces",
    )?;
    require(
        bounded.edit_piece_count <= 3,
        "the bounded form stays inside the three-piece logical bound",
    )?;
    // A second and third edit of the same pending file keep the tree
    // representation: the bounded form converts before the second splice.
    for (start, bytes) in [(10_u64, b"ZW"), (1_u64, b"QR")] {
        client.edit_workspace_file_range(layerfs_sdk::WorkspaceFileRangeEdit {
            workspace_id: session,
            path: "compact".into(),
            start,
            delete_len: 2,
            replacement: layerfs_sdk::WorkspaceFileReplacement::Inline(bytes.to_vec()),
        })?;
    }
    require(
        matches!(
            client.commit_workspace_session(session)?,
            WorkspaceCommitResult::Created { .. }
        ),
        "compact proof conversion Commit",
    )?;
    let converted = layerfs_layerstack_store::take_workspace_commit_diagnostics();
    let converted = converted.last().ok_or("conversion Commit diagnostics")?;
    require(
        converted.edit_count == 2,
        "two counted edits of the converted file",
    )?;
    require(
        converted.edit_piece_logical_charge == 5 * 128,
        "the second pending splice materializes the rooted representation",
    )?;
    require(
        converted.edit_piece_count == 5,
        "five logical pieces after the second splice",
    )?;
    eprintln!(
        "live-cut compact_proof bounded_charge={} bounded_pieces={} converted_charge={} converted_pieces={}",
        bounded.edit_piece_logical_charge,
        bounded.edit_piece_count,
        converted.edit_piece_logical_charge,
        converted.edit_piece_count
    );
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
        if String::from_utf8_lossy(&bytes)
            .lines()
            .any(|line| line == marker)
        {
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
