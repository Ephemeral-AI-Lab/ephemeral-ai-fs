//! #116 Phase 1 capacity probe: bounded, public-API reproductions.
//!
//! Explicit selection only:
//! `cargo +1.85.1 test -p layerfs-workspace --release --test issue116_capacity \
//!    -- --ignored --nocapture --test-threads=1`
//!
//! Fixture preparation (source files) happens before the session opens and is
//! not part of any measured edit. The probe reports the exact rejection point
//! and public error for the per-file edit-count budget, the pending-workspace
//! piece budget and pending directory width. It never changes product behavior.

use layerfs_layerstack_store::{
    EntityName, LayerStackInitialization, LayerStackStore, LocalForkSource,
};
use layerfs_workspace::{
    CreateWorkspaceSession, EndWorkspaceMode, WorkspaceFileRangeEdit, WorkspaceFileReplacement,
    WorkspacePlacement, WorkspaceProjection, WorkspaceSession, Workspaces,
};

fn fixture(
    label: &str,
    build: impl FnOnce(&std::path::Path),
) -> (std::path::PathBuf, Workspaces, layerfs_layerstack_store::BranchId) {
    let root = std::env::temp_dir().join(format!(
        "layerfs-issue116-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let source = root.join("source");
    std::fs::create_dir_all(&source).unwrap();
    build(&source);
    let store = LayerStackStore::create(root.join("store.sqlite")).unwrap();
    let layer = store
        .initialize_layerstack(
            EntityName::new("project").unwrap(),
            LayerStackInitialization::Directory(source),
        )
        .unwrap()
        .genesis_layer_id;
    let branch = store
        .fork_branch(
            EntityName::new("main").unwrap(),
            LocalForkSource::Layer { layer_id: layer },
        )
        .unwrap();
    let workspaces = Workspaces::new(root.join("runtime"), store.clone()).unwrap();
    (root, workspaces, branch)
}

fn source_files(source: &std::path::Path, count: usize, bytes: usize) {
    let payload = vec![3u8; bytes];
    for index in 0..count {
        std::fs::write(source.join(format!("f{index}")), &payload).unwrap();
    }
}

fn open_session(
    root: &std::path::Path,
    workspaces: &Workspaces,
    branch: layerfs_layerstack_store::BranchId,
    name: &str,
) -> WorkspaceSession {
    workspaces
        .create_workspace_session(CreateWorkspaceSession {
            branch_id: branch,
            placement: WorkspacePlacement::Host {
                root: root.join(name),
            },
            projection: Some(WorkspaceProjection::Materialize),
        })
        .unwrap()
}

fn edit(
    workspaces: &Workspaces,
    session: &WorkspaceSession,
    path: &str,
    start: u64,
    delete_len: u64,
    replacement: WorkspaceFileReplacement,
) -> layerfs_workspace::WorkspaceResult<()> {
    workspaces.edit_workspace_file_range(WorkspaceFileRangeEdit {
        workspace_id: session.id,
        path: path.into(),
        start,
        delete_len,
        replacement,
    })
}

fn finish(root: std::path::PathBuf, workspaces: Workspaces, session: WorkspaceSession) {
    let _ = workspaces.end_workspace_session(session.id, EndWorkspaceMode::Discard);
    drop(workspaces);
    let _ = std::fs::remove_dir_all(root);
}

/// A: repeated overwrite of the same four bytes in one 4 KiB file.
#[test]
#[ignore = "issue116 phase-1 capacity probe"]
fn probe_a_repeated_overwrite_of_one_file() {
    let (root, workspaces, branch) = fixture("probe-a", |source| source_files(source, 1, 4096));
    let session = open_session(&root, &workspaces, branch, "mount");
    let mut accepted = 0u32;
    let mut rejection = None;
    for index in 0..10_000u32 {
        let value = (index & 0xff) as u8;
        match edit(
            &workspaces,
            &session,
            "f0",
            0,
            4,
            WorkspaceFileReplacement::Inline(vec![value; 4]),
        ) {
            Ok(()) => accepted += 1,
            Err(error) => {
                rejection = Some(format!("{error:?}"));
                break;
            }
        }
    }
    println!("PROBE A accepted={accepted} rejection={rejection:?}");
    assert!(rejection.is_none(), "pending edits must not be rejected by a count");
    finish(root, workspaces, session);
}

/// B: repeated append-shaped growth of one empty file through the public SDK route.
#[test]
#[ignore = "issue116 phase-1 capacity probe"]
fn probe_b_repeated_append_growth_of_one_file() {
    let (root, workspaces, branch) = fixture("probe-b", |source| source_files(source, 1, 0));
    let session = open_session(&root, &workspaces, branch, "mount");
    let payload = vec![9u8; 4096];
    let mut accepted = 0u32;
    let mut offset = 0u64;
    let mut rejection = None;
    for _ in 0..300_000u32 {
        match edit(
            &workspaces,
            &session,
            "f0",
            offset,
            0,
            WorkspaceFileReplacement::Inline(payload.clone()),
        ) {
            Ok(()) => {
                accepted += 1;
                offset += payload.len() as u64;
            }
            Err(error) => {
                rejection = Some(format!("{error:?}"));
                break;
            }
        }
    }
    println!("PROBE B accepted={accepted} final_len={offset} rejection={rejection:?}");
    finish(root, workspaces, session);
}

/// C: many distinct pre-existing files each changed once in one pending workspace.
#[test]
#[ignore = "issue116 phase-1 capacity probe"]
fn probe_c_many_changed_files() {
    let started = std::time::Instant::now();
    let (root, workspaces, branch) = fixture("probe-c", |source| source_files(source, 2_000, 4096));
    println!("PROBE C fixture_ns={}", started.elapsed().as_nanos());
    let session = open_session(&root, &workspaces, branch, "mount");
    let mut accepted = 0u32;
    let mut rejection = None;
    let mut last = std::time::Instant::now();
    for index in 0..2_000u32 {
        let path = format!("f{index}");
        match edit(
            &workspaces,
            &session,
            &path,
            0,
            1,
            WorkspaceFileReplacement::Inline(vec![1]),
        ) {
            Ok(()) => {
                accepted += 1;
                if accepted % 100 == 0 {
                    println!(
                        "PROBE C accepted={accepted} batch_ms={}",
                        last.elapsed().as_millis()
                    );
                    last = std::time::Instant::now();
                }
            }
            Err(error) => {
                rejection = Some(format!("{error:?}"));
                break;
            }
        }
    }
    println!(
        "PROBE C accepted={accepted} total_ms={} rejection={rejection:?}",
        started.elapsed().as_millis()
    );
    finish(root, workspaces, session);
}

/// D: one genuinely wide single directory, every entry changed once.
#[test]
#[ignore = "issue116 phase-1 capacity probe"]
fn probe_d_wide_single_directory() {
    let started = std::time::Instant::now();
    let (root, workspaces, branch) = fixture("probe-d", |source| {
        std::fs::create_dir_all(source.join("wide")).unwrap();
        let payload = vec![4u8; 4096];
        for index in 0..2_000 {
            std::fs::write(source.join("wide").join(format!("f{index}")), &payload).unwrap();
        }
    });
    println!("PROBE D fixture_ns={}", started.elapsed().as_nanos());
    let session = open_session(&root, &workspaces, branch, "mount");
    let mut accepted = 0u32;
    let mut rejection = None;
    let mut last = std::time::Instant::now();
    for index in 0..2_000u32 {
        let path = format!("wide/f{index}");
        match edit(
            &workspaces,
            &session,
            &path,
            0,
            1,
            WorkspaceFileReplacement::Inline(vec![1]),
        ) {
            Ok(()) => {
                accepted += 1;
                if accepted % 100 == 0 {
                    println!(
                        "PROBE D accepted={accepted} batch_ms={}",
                        last.elapsed().as_millis()
                    );
                    last = std::time::Instant::now();
                }
            }
            Err(error) => {
                rejection = Some(format!("{error:?}"));
                break;
            }
        }
    }
    println!(
        "PROBE D accepted={accepted} total_ms={} rejection={rejection:?}",
        started.elapsed().as_millis()
    );
    finish(root, workspaces, session);
}

/// E: live pending-set charges after N distinct changed files.
///
/// The charges are read from the public workspace detail counters through the
/// same public session used by the SDK route, so the numbers are the product's
/// own accounting rather than a re-derivation.
#[test]
#[ignore = "issue116 phase-1 capacity probe"]
fn probe_e_pending_set_charges() {
    for files in [250usize, 500, 1_000] {
        let started = std::time::Instant::now();
        let (root, workspaces, branch) = fixture("probe-e", |source| source_files(source, files, 4096));
        let fixture_ns = started.elapsed().as_nanos();
        let session = open_session(&root, &workspaces, branch, "mount");
        let mut accepted = 0u32;
        let mut rejection = None;
        for index in 0..files {
            let path = format!("f{index}");
            match edit(
                &workspaces,
                &session,
                &path,
                0,
                1,
                WorkspaceFileReplacement::Inline(vec![1]),
            ) {
                Ok(()) => accepted += 1,
                Err(error) => {
                    rejection = Some(format!("{error:?}"));
                    break;
                }
            }
        }
        println!(
            "PROBE E files={files} accepted={accepted} fixture_ms={} total_ms={} rejection={rejection:?}",
            fixture_ns / 1_000_000,
            started.elapsed().as_millis()
        );
        finish(root, workspaces, session);
    }
}
/// F: measured per-edit cost of the public SDK route at two pending-set sizes.
#[test]
#[ignore = "issue116 phase-1 capacity probe"]
fn probe_f_edit_cost() {
    for files in [1usize, 200] {
        let (root, workspaces, branch) = fixture("probe-f", |source| source_files(source, files, 4096));
        let session = open_session(&root, &workspaces, branch, "mount");
        let mut accepted = 0u32;
        let started = std::time::Instant::now();
        for index in 0..20u32 {
            let path = format!("f{}", index as usize % files);
            match edit(&workspaces, &session, &path, 0, 1, WorkspaceFileReplacement::Inline(vec![1])) {
                Ok(()) => accepted += 1,
                Err(error) => {
                    println!("PROBE F files={files} rejected_at={index} error={error:?}");
                    break;
                }
            }
        }
        println!(
            "PROBE F files={files} accepted={accepted} total_ms={} per_edit_ms={}",
            started.elapsed().as_millis(),
            started.elapsed().as_millis() / u128::from(accepted.max(1))
        );
        finish(root, workspaces, session);
    }
}
