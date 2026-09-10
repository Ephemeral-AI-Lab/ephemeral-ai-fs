use layerfs_content::{
    filesystem::{self, ContentChange},
    CanonicalPath, ObjectId,
};
use layerfs_layerstack_store::{
    apply_changes, CommitOutcome, CompactionOptions, CoreReader, EntityName,
    LayerStackInitialization, LayerStackStore, LocalForkSource,
};
use std::path::PathBuf;

fn folder() -> PathBuf {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    loop {
        let path = std::env::temp_dir().join(format!(
            "layerfs-compaction-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        match std::fs::create_dir(&path) {
            Ok(()) => return path,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => (),
            Err(error) => panic!("{error}"),
        }
    }
}
fn random(length: usize) -> Vec<u8> {
    let mut n = 0x19138919u32;
    (0..length)
        .map(|_| {
            n ^= n << 13;
            n ^= n >> 17;
            n ^= n << 5;
            n as u8
        })
        .collect()
}
fn read(store: &LayerStackStore, root: ObjectId, path: &str) -> Vec<u8> {
    let reader = store.snapshot_reader(root);
    let mut bytes = Vec::new();
    filesystem::stream(
        &CoreReader(&reader),
        root,
        &CanonicalPath::new(path).unwrap(),
        &mut bytes,
    )
    .unwrap();
    bytes
}

#[test]
fn public_compaction_preserves_history_hardlinks_and_future_writes() {
    let folder = folder();
    let input = folder.join("input");
    std::fs::create_dir(&input).unwrap();
    let mut small = random(32768);
    let mut large = random(256 * 1024);
    std::fs::write(input.join("small"), &small).unwrap();
    std::fs::hard_link(input.join("small"), input.join("alias")).unwrap();
    std::fs::write(input.join("large"), &large).unwrap();
    std::fs::write(input.join("over-limit"), vec![b'x'; 2 * 1024 * 1024 + 1]).unwrap();
    let path = folder.join("source.sqlite");
    let source = LayerStackStore::create(&path).unwrap();
    let initial = source
        .initialize_layerstack(
            EntityName::new("demo").unwrap(),
            LayerStackInitialization::Directory(input),
        )
        .unwrap();
    let genesis = source
        .layer(initial.genesis_layer_id)
        .unwrap()
        .unwrap()
        .root_id;
    let branch = source
        .fork_branch(
            EntityName::new("main").unwrap(),
            LocalForkSource::Layer {
                layer_id: initial.genesis_layer_id,
            },
        )
        .unwrap();
    let mut history = vec![(genesis, small.clone(), large.clone())];
    let mut last_commit = None;
    for step in 1..=5 {
        small[step * 271] ^= step as u8;
        large[step * 1234] ^= step as u8;
        let pinned = source.pin_branch(branch).unwrap();
        let candidate = apply_changes(
            &pinned.reader,
            pinned.root,
            &[
                ContentChange::Write {
                    path: "small".into(),
                    bytes: small.clone(),
                    mode: 0o640,
                },
                ContentChange::Write {
                    path: "large".into(),
                    bytes: large.clone(),
                    mode: 0o644,
                },
            ],
            [step as u8; 32],
        )
        .unwrap();
        let CommitOutcome::Committed {
            root_id, commit_id, ..
        } = source
            .commit_candidate(
                &pinned.branch,
                pinned.root,
                pinned.branch.base_layer_id,
                candidate,
            )
            .unwrap()
        else {
            panic!("commit required")
        };
        history.push((root_id, small.clone(), large.clone()));
        last_commit = Some(commit_id);
    }
    let before = std::fs::read(&path).unwrap();
    let output = folder.join("compacted.sqlite");
    let receipt = source
        .compact_into(&output, CompactionOptions::default())
        .unwrap();
    eprintln!("compaction receipt: {receipt:?}");
    assert!(receipt.published && receipt.cleanup_complete && receipt.directory_synced);
    assert!(
        receipt.small_prefix > 0
            && receipt.whole_prefix > 0
            && receipt.native_slices > 0
            && receipt.native_full > 0
    );
    assert_eq!(
        std::fs::read(&path).unwrap(),
        before,
        "source is preserved exactly"
    );
    assert!(
        source
            .compact_into(&output, CompactionOptions::default())
            .is_err(),
        "destination cannot be overwritten"
    );
    let compacted = LayerStackStore::connect(&output).unwrap();
    for (root, small, large) in &history {
        assert_eq!(read(&compacted, *root, "small"), *small);
        assert_eq!(read(&compacted, *root, "alias"), *small);
        assert_eq!(read(&compacted, *root, "large"), *large);
        let reader = compacted.snapshot_reader(*root);
        assert_eq!(
            filesystem::stat(
                &CoreReader(&reader),
                *root,
                &CanonicalPath::new("alias").unwrap()
            )
            .unwrap()
            .0
            .namespace_ref_count,
            2
        );
    }
    let fork = compacted
        .fork_branch(
            EntityName::new("fork").unwrap(),
            LocalForkSource::Branch {
                branch_id: branch,
                commit_id: last_commit.unwrap(),
            },
        )
        .unwrap();
    let pinned = compacted.pin_branch(fork).unwrap();
    let mut future_large = large.clone();
    future_large[4567] ^= 19;
    let candidate = apply_changes(
        &pinned.reader,
        pinned.root,
        &[ContentChange::Write {
            path: "large".into(),
            bytes: future_large.clone(),
            mode: 0o600,
        }],
        [91; 32],
    )
    .unwrap();
    let CommitOutcome::Committed { root_id, .. } = compacted
        .commit_candidate(
            &pinned.branch,
            pinned.root,
            pinned.branch.base_layer_id,
            candidate,
        )
        .unwrap()
    else {
        panic!("commit required")
    };
    drop(pinned);
    drop(compacted);
    let reopened = LayerStackStore::connect(&output).unwrap();
    assert_eq!(read(&reopened, root_id, "large"), future_large);
    assert_eq!(read(&reopened, genesis, "large"), history[0].2);
    let repeated = folder.join("compacted-again.sqlite");
    let repeated_receipt = reopened
        .compact_into(&repeated, CompactionOptions::default())
        .unwrap();
    assert!(
        repeated_receipt.published
            && repeated_receipt.cleanup_complete
            && repeated_receipt.directory_synced
    );
    let repeated = LayerStackStore::connect(repeated).unwrap();
    assert_eq!(read(&repeated, root_id, "large"), future_large);
    assert_eq!(read(&repeated, genesis, "large"), history[0].2);
    drop(repeated);
    drop(reopened);
    drop(source);
    std::fs::remove_dir_all(folder).unwrap();
}

#[test]
fn compaction_budget_failure_does_not_publish_or_change_source() {
    let folder = folder();
    let path = folder.join("source.sqlite");
    let source = LayerStackStore::create(&path).unwrap();
    source
        .initialize_layerstack(
            EntityName::new("demo").unwrap(),
            LayerStackInitialization::Empty,
        )
        .unwrap();
    let before = std::fs::read(&path).unwrap();
    let output = folder.join("compacted.sqlite");
    assert!(source
        .compact_into(
            &output,
            CompactionOptions {
                temporary_byte_limit: 4096
            }
        )
        .is_err());
    assert!(!output.exists());
    assert_eq!(std::fs::read(&path).unwrap(), before);
    drop(source);
    std::fs::remove_dir_all(folder).unwrap();
}

#[test]
fn compaction_failure_boundaries_preserve_source_and_report_published_outcome() {
    let folder = folder();
    let path = folder.join("source.sqlite");
    let source = LayerStackStore::create(&path).unwrap();
    let initial = source
        .initialize_layerstack(
            EntityName::new("demo").unwrap(),
            LayerStackInitialization::Empty,
        )
        .unwrap();
    let branch = source
        .fork_branch(
            EntityName::new("main").unwrap(),
            LocalForkSource::Layer {
                layer_id: initial.genesis_layer_id,
            },
        )
        .unwrap();
    let pinned = source.pin_branch(branch).unwrap();
    let candidate = apply_changes(
        &pinned.reader,
        pinned.root,
        &[ContentChange::Write {
            path: "file".into(),
            bytes: random(16384),
            mode: 0o640,
        }],
        [3; 32],
    )
    .unwrap();
    source
        .commit_candidate(
            &pinned.branch,
            pinned.root,
            pinned.branch.base_layer_id,
            candidate,
        )
        .unwrap();
    drop(pinned);
    let before = std::fs::read(&path).unwrap();
    for point in [10, 11, 12, 14] {
        let output = folder.join(format!("failed-{point}.sqlite"));
        layerfs_layerstack_store::set_transaction_failure_at(Some(u64::MAX - point));
        let result = source.compact_into(&output, CompactionOptions::default());
        layerfs_layerstack_store::set_transaction_failure_at(None);
        assert!(result.is_err(), "point={point}");
        assert!(!output.exists());
        assert_eq!(std::fs::read(&path).unwrap(), before);
        assert_eq!(
            std::fs::read_dir(&folder).unwrap().count(),
            1,
            "owned work files must be cleaned"
        );
    }
    let output = folder.join("published.sqlite");
    layerfs_layerstack_store::set_transaction_failure_at(Some(u64::MAX - 13));
    let result = source.compact_into(&output, CompactionOptions::default());
    layerfs_layerstack_store::set_transaction_failure_at(None);
    let receipt = result.unwrap();
    assert!(receipt.published && receipt.cleanup_complete && !receipt.directory_synced);
    assert!(!receipt.publication_notes.is_empty());
    let published = LayerStackStore::connect(&output).unwrap();
    assert_eq!(
        published.branch(branch).unwrap(),
        source.branch(branch).unwrap()
    );
    assert_eq!(std::fs::read(&path).unwrap(), before);
    drop(published);
    drop(source);
    std::fs::remove_dir_all(folder).unwrap();
}

#[test]
fn compaction_uses_four_kib_pages_for_a_supported_large_page_source() {
    let folder = folder();
    let path = folder.join("source.sqlite");
    let source = LayerStackStore::create(&path).unwrap();
    source
        .initialize_layerstack(
            EntityName::new("demo").unwrap(),
            LayerStackInitialization::Empty,
        )
        .unwrap();
    drop(source);
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection
        .execute_batch("PRAGMA page_size=65536; VACUUM;")
        .unwrap();
    drop(connection);
    let source = LayerStackStore::connect(&path).unwrap();
    let before = std::fs::read(&path).unwrap();
    let output = folder.join("compacted.sqlite");
    let receipt = source
        .compact_into(&output, CompactionOptions::default())
        .unwrap();
    assert!(receipt.published && receipt.directory_synced);
    let output = LayerStackStore::connect(&output).unwrap();
    assert_eq!(
        output
            .inspect_connection(|connection| connection
                .pragma_query_value(None, "page_size", |row| row.get::<_, i64>(0))
                .unwrap())
            .unwrap(),
        4096
    );
    assert_eq!(std::fs::read(&path).unwrap(), before);
    drop(output);
    drop(source);
    std::fs::remove_dir_all(folder).unwrap();
}

#[test]
fn compaction_rejects_corrupt_dependencies_before_publication() {
    let folder = folder();
    let path = folder.join("source.sqlite");
    let source = LayerStackStore::create(&path).unwrap();
    let initial = source
        .initialize_layerstack(
            EntityName::new("demo").unwrap(),
            LayerStackInitialization::Empty,
        )
        .unwrap();
    let branch = source
        .fork_branch(
            EntityName::new("main").unwrap(),
            LocalForkSource::Layer {
                layer_id: initial.genesis_layer_id,
            },
        )
        .unwrap();
    let pinned = source.pin_branch(branch).unwrap();
    let candidate = apply_changes(
        &pinned.reader,
        pinned.root,
        &[ContentChange::Write {
            path: "file".into(),
            bytes: random(16384),
            mode: 0o640,
        }],
        [4; 32],
    )
    .unwrap();
    let CommitOutcome::Committed { root_id, .. } = source
        .commit_candidate(
            &pinned.branch,
            pinned.root,
            pinned.branch.base_layer_id,
            candidate,
        )
        .unwrap()
    else {
        panic!("commit required")
    };
    drop(pinned);
    let reader = source.snapshot_reader(root_id);
    let payload = filesystem::stat(
        &CoreReader(&reader),
        root_id,
        &CanonicalPath::new("file").unwrap(),
    )
    .unwrap()
    .0
    .content_root;
    drop(reader);
    source.inspect_connection(|connection| {
        let (pack, mut bytes): (i64, Vec<u8>) = connection.query_row("SELECT p.pack_id,p.data FROM object_packs p JOIN objects o USING(pack_id) WHERE o.object_id=?1", [payload.as_bytes().as_slice()], |row| Ok((row.get(0)?, row.get(1)?))).unwrap();
        *bytes.last_mut().unwrap() ^= 1;
        connection.execute("UPDATE object_packs SET data=?1 WHERE pack_id=?2", rusqlite::params![bytes, pack]).unwrap();
    }).unwrap();
    let corrupt_source = std::fs::read(&path).unwrap();
    let output = folder.join("compacted.sqlite");
    assert!(source
        .compact_into(&output, CompactionOptions::default())
        .is_err());
    assert!(!output.exists());
    assert_eq!(std::fs::read(&path).unwrap(), corrupt_source);
    assert_eq!(std::fs::read_dir(&folder).unwrap().count(), 1);
    drop(source);
    std::fs::remove_dir_all(folder).unwrap();
}

#[test]
fn compaction_mid_rewrite_quota_failure_preserves_original() {
    let folder = folder();
    let input = folder.join("input");
    std::fs::create_dir(&input).unwrap();
    std::fs::write(input.join("large"), random(2 * 1024 * 1024)).unwrap();
    let path = folder.join("source.sqlite");
    let source = LayerStackStore::create(&path).unwrap();
    source
        .initialize_layerstack(
            EntityName::new("demo").unwrap(),
            LayerStackInitialization::Directory(input),
        )
        .unwrap();
    let before = std::fs::read(&path).unwrap();
    let output = folder.join("compacted.sqlite");
    // Source fits its 3-MiB share; adding replacement packs to the private copy
    // exceeds it. This exercises an actual SQLite quota failure after construction.
    assert!(before.len() < 3 * 1024 * 1024);
    let result = source.compact_into(
        &output,
        CompactionOptions {
            temporary_byte_limit: 12 * 1024 * 1024,
        },
    );
    assert!(
        matches!(
            result,
            Err(layerfs_layerstack_store::StoreError::Database(_))
        ),
        "{result:?}"
    );
    assert!(!output.exists());
    assert_eq!(std::fs::read(&path).unwrap(), before);
    assert_eq!(std::fs::read_dir(&folder).unwrap().count(), 2);
    drop(source);
    std::fs::remove_dir_all(folder).unwrap();
}
