use layerfs_content::filesystem::ContentChange;
use layerfs_layerstack_store::{
    apply_changes, AddLayerResult, CommitOutcome, EntityName, LayerStackInitialization,
    LayerStackInitializationReceipt, LayerStackStore, LocalForkSource, ObjectSource, StoreError,
};
use std::collections::BTreeSet;

#[test]
fn exact_v10_schema_runtime_and_old_schema_rejection() {
    let root = temp("schema");
    let path = root.join("store.sqlite");
    let store = LayerStackStore::create(&path).unwrap();
    assert_eq!(store_files(&root), vec!["store.sqlite"]);
    drop(store);
    let connection =
        rusqlite::Connection::open_with_flags(&path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
            .unwrap();
    assert_eq!(pragma(&connection, "application_id"), 0x4c46_534c);
    assert_eq!(pragma(&connection, "user_version"), 10);
    assert_eq!(pragma(&connection, "page_size"), 4096);

    let tables = connection
        .prepare(
            "SELECT name,ncol,wr,strict FROM pragma_table_list \
             WHERE schema='main' AND name NOT LIKE 'sqlite_%' ORDER BY name",
        )
        .unwrap()
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)?,
            ))
        })
        .unwrap()
        .collect::<rusqlite::Result<Vec<_>>>()
        .unwrap();
    assert_eq!(
        tables,
        vec![
            ("branches".to_owned(), 5, 1, 1),
            ("commits".to_owned(), 4, 1, 1),
            ("layer_stacks".to_owned(), 3, 1, 1),
            ("layers".to_owned(), 6, 1, 1),
            ("metadata_value_groups".to_owned(), 5, 0, 1),
            ("object_packs".to_owned(), 2, 0, 1),
            ("objects".to_owned(), 5, 1, 1),
            ("scope_allocator".to_owned(), 2, 1, 1),
            ("workspace_stages".to_owned(), 3, 1, 1),
        ]
    );
    assert_eq!(tables.iter().map(|table| table.1).sum::<i64>(), 35);
    let indexes = connection
        .prepare(
            "SELECT name FROM sqlite_schema \
             WHERE type='index' AND name NOT LIKE 'sqlite_%' ORDER BY name",
        )
        .unwrap()
        .query_map([], |row| row.get::<_, String>(0))
        .unwrap()
        .collect::<rusqlite::Result<BTreeSet<_>>>()
        .unwrap();
    assert_eq!(
        indexes,
        BTreeSet::from([
            "branch_identity".to_owned(),
            "branch_names".to_owned(),
            "layer_identity".to_owned(),
            "layer_stack_names".to_owned(),
            "layers_child".to_owned(),
            "layers_genesis".to_owned(),
            "layers_source".to_owned(),
        ])
    );
    drop(connection);

    let old = root.join("old.sqlite");
    let connection = rusqlite::Connection::open(&old).unwrap();
    connection
        .execute_batch(
            "PRAGMA application_id=1279677260; PRAGMA user_version=3; \
             CREATE TABLE store(singleton INTEGER PRIMARY KEY) STRICT;",
        )
        .unwrap();
    drop(connection);
    assert!(matches!(
        LayerStackStore::connect(&old),
        Err(StoreError::WrongStoreSchema)
    ));
    let connection = rusqlite::Connection::open(&old).unwrap();
    assert_eq!(pragma(&connection, "user_version"), 3);
    assert_eq!(
        connection
            .query_row(
                "SELECT count(*) FROM sqlite_schema WHERE name='store'",
                [],
                |row| { row.get::<_, i64>(0) }
            )
            .unwrap(),
        1
    );

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn one_store_initialize_fork_commit_add_and_dedup_are_atomic() {
    let root = temp("lifecycle");
    let store = LayerStackStore::create(root.join("store.sqlite")).unwrap();
    let initialized = store
        .initialize_layerstack(
            EntityName::new("demo").unwrap(),
            LayerStackInitialization::Empty,
        )
        .unwrap();
    let branch_id = store
        .fork_branch(
            EntityName::new("main").unwrap(),
            LocalForkSource::Layer {
                layer_id: initialized.genesis_layer_id,
            },
        )
        .unwrap();
    let pinned = store.pin_branch(branch_id).unwrap();
    let namespace = layerfs_content::filesystem::namespace(
        &layerfs_layerstack_store::CoreReader(&pinned.reader),
        pinned.root,
    )
    .unwrap();
    assert!(namespace.scope.is_some());
    assert_eq!(
        &layerfs_content::decode_bytes_object(
            &pinned
                .reader
                .read_object(namespace.inode_table_root)
                .unwrap()
        )
        .unwrap()[..8],
        b"LFS6INT\0"
    );
    let built = apply_changes(
        &pinned.reader,
        pinned.root,
        &[ContentChange::Write {
            path: "hello".to_owned(),
            bytes: b"world".to_vec(),
            mode: 0o644,
        }],
        [7; 32],
    )
    .unwrap();
    let commit_id = match store
        .commit_candidate(
            &pinned.branch,
            pinned.root,
            pinned.branch.base_layer_id,
            built,
        )
        .unwrap()
    {
        CommitOutcome::Committed {
            commit_id,
            candidate_objects,
            inserted_objects,
            reused_objects,
            candidate_bytes,
            inserted_bytes,
            reused_bytes,
            ..
        } => {
            assert_eq!(candidate_objects, inserted_objects + reused_objects);
            assert_eq!(candidate_bytes, inserted_bytes + reused_bytes);
            commit_id
        }
        outcome => panic!("unexpected Commit outcome: {outcome:?}"),
    };
    assert_eq!(
        store.branch(branch_id).unwrap().unwrap().head_commit_id,
        Some(commit_id)
    );
    assert!(store.commit(commit_id).unwrap().is_some());
    let layer_id = match store.add_layer(branch_id).unwrap() {
        AddLayerResult::Added { layer_id } => layer_id,
        outcome => panic!("unexpected Add outcome: {outcome:?}"),
    };
    assert_eq!(
        store.add_layer(branch_id).unwrap(),
        AddLayerResult::UpToDate { layer_id }
    );
    let counts = store.store_counts().unwrap();
    assert_eq!(counts.layer_stacks, 1);
    assert_eq!(counts.layers, 2);
    assert_eq!(counts.branches, 1);
    assert_eq!(counts.commits, 1);
    assert_eq!(store.canonical_storage().unwrap().objects, counts.objects);

    drop(store);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn directory_initialization_receipt_counts_scanned_files_and_bytes() {
    let root = temp("initialization-receipt");
    let source = root.join("source");
    std::fs::create_dir(&source).unwrap();
    std::fs::write(source.join("first"), b"one").unwrap();
    std::fs::write(source.join("second"), b"twenty").unwrap();
    let store = LayerStackStore::create(root.join("store.sqlite")).unwrap();
    store.take_layerstack_initialization_receipts();
    let initialized = store
        .initialize_layerstack(
            EntityName::new("receipt").unwrap(),
            LayerStackInitialization::Directory(source),
        )
        .unwrap();
    assert_eq!(
        store.take_layerstack_initialization_receipts(),
        vec![LayerStackInitializationReceipt {
            layer_stack_id: initialized.layer_stack_id,
            scanned_files: 2,
            scanned_bytes: 9,
            source_passes: 1,
        }]
    );

    drop(store);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn compact_framing_public_lifecycle_and_file_boundaries() {
    use layerfs_content::{filesystem, CanonicalPath};
    use layerfs_layerstack_store::CoreReader;
    let root = temp("compact-framing-lifecycle");
    let source = root.join("source");
    std::fs::create_dir(&source).unwrap();
    let files: Vec<_> = [0, 1, 131071, 131072, 131073, 2 * 1024 * 1024 + 1]
        .into_iter()
        .map(|length| {
            let name = format!("file-{length}");
            let bytes: Vec<u8> = (0..length).map(|i| (i % 251) as u8).collect();
            std::fs::write(source.join(&name), &bytes).unwrap();
            (name, bytes)
        })
        .collect();
    std::fs::hard_link(source.join("file-131071"), source.join("alias")).unwrap();
    let path = root.join("store.sqlite");
    let store = LayerStackStore::create(&path).unwrap();
    let initialized = store
        .initialize_layerstack(
            EntityName::new("framing").unwrap(),
            LayerStackInitialization::Directory(source),
        )
        .unwrap();
    let genesis = store
        .layer(initialized.genesis_layer_id)
        .unwrap()
        .unwrap()
        .root_id;
    {
        let reader = store.snapshot_reader(genesis);
        assert!(filesystem::namespace(&CoreReader(&reader), genesis)
            .unwrap()
            .scope
            .is_some());
        let indexed = store
            .inspect_connection(|db| {
                db.prepare("SELECT object_id FROM objects")
                    .unwrap()
                    .query_map([], |row| row.get::<_, Vec<u8>>(0))
                    .unwrap()
                    .map(|row| layerfs_content::ObjectId::from_bytes(&row.unwrap()).unwrap())
                    .collect::<BTreeSet<_>>()
            })
            .unwrap();
        let mut reached = BTreeSet::new();
        let mut pending = vec![genesis];
        while let Some(id) = pending.pop() {
            if !reached.insert(id) {
                continue;
            }
            let bytes = reader.read_object(id).unwrap();
            layerfs_content::authenticate_identity(&bytes, id).unwrap();
            let value = layerfs_content::decode_bytes_object(&bytes).unwrap();
            assert!(!value.starts_with(b"LFS4INO\0") && !value.starts_with(b"LFS4DIR\0"));
            pending
                .extend(layerfs_content::object::references::referenced_objects(&bytes).unwrap());
        }
        assert_eq!(
            reached, indexed,
            "initialization must admit only final reachable objects"
        );
    }
    let branch = store
        .fork_branch(
            EntityName::new("main").unwrap(),
            LocalForkSource::Layer {
                layer_id: initialized.genesis_layer_id,
            },
        )
        .unwrap();
    store
        .inspect_connection(|db| {
            assert!(db
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM object_packs WHERE substr(data,9,4)=x'04000000')",
                    [],
                    |r| r.get::<_, bool>(0)
                )
                .unwrap());
            assert!(db
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM object_packs WHERE substr(data,9,4)=x'06000000')",
                    [],
                    |r| r.get::<_, bool>(0)
                )
                .unwrap());
            assert!(
                db.query_row("SELECT count(*) FROM metadata_value_groups", [], |r| r
                    .get::<_, i64>(0))
                    .unwrap()
                    > 0
            );
        })
        .unwrap();
    drop(store);
    let store = LayerStackStore::connect(&path).unwrap();
    let before = store.snapshot_reader(genesis);
    for (name, expected) in &files {
        let mut actual = Vec::new();
        filesystem::stream(
            &CoreReader(&before),
            genesis,
            &CanonicalPath::from_bytes(name.as_bytes()).unwrap(),
            &mut actual,
        )
        .unwrap();
        assert_eq!(&actual, expected);
    }
    let alias = CanonicalPath::from_bytes(b"alias").unwrap();
    assert_eq!(
        filesystem::stat(&CoreReader(&before), genesis, &alias)
            .unwrap()
            .0
            .namespace_ref_count,
        2
    );
    let pinned = store.pin_branch(branch).unwrap();
    let candidate = apply_changes(
        &pinned.reader,
        pinned.root,
        &[ContentChange::Write {
            path: "file-1".into(),
            bytes: b"changed".to_vec(),
            mode: 0o600,
        }],
        [31; 32],
    )
    .unwrap();
    let (commit, committed_root) = match store
        .commit_candidate(
            &pinned.branch,
            pinned.root,
            pinned.branch.base_layer_id,
            candidate,
        )
        .unwrap()
    {
        CommitOutcome::Committed {
            commit_id, root_id, ..
        } => (commit_id, root_id),
        outcome => panic!("unexpected outcome: {outcome:?}"),
    };
    let fork = store
        .fork_branch(
            EntityName::new("fork").unwrap(),
            LocalForkSource::Branch {
                branch_id: branch,
                commit_id: commit,
            },
        )
        .unwrap();
    let pinned_fork = store.pin_branch(fork).unwrap();
    let candidate = apply_changes(
        &pinned_fork.reader,
        pinned_fork.root,
        &[ContentChange::Write {
            path: "file-1".into(),
            bytes: b"fork only".to_vec(),
            mode: 0o640,
        }],
        [32; 32],
    )
    .unwrap();
    assert!(matches!(
        store
            .commit_candidate(
                &pinned_fork.branch,
                pinned_fork.root,
                pinned_fork.branch.base_layer_id,
                candidate
            )
            .unwrap(),
        CommitOutcome::Committed { .. }
    ));
    drop(pinned_fork);
    drop(pinned);
    drop(before);
    drop(store);
    let store = LayerStackStore::connect(&path).unwrap();
    for (state, expected) in [
        (genesis, b"\0".as_slice()),
        (committed_root, b"changed".as_slice()),
        (
            store.pin_branch(fork).unwrap().root,
            b"fork only".as_slice(),
        ),
    ] {
        let reader = store.snapshot_reader(state);
        let mut actual = Vec::new();
        filesystem::stream(
            &CoreReader(&reader),
            state,
            &CanonicalPath::from_bytes(b"file-1").unwrap(),
            &mut actual,
        )
        .unwrap();
        assert_eq!(actual, expected);
    }
    assert_eq!(
        store.canonical_storage().unwrap().objects,
        store.store_counts().unwrap().objects
    );
    drop(store);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn entity_names_enforce_the_exact_boundary() {
    assert!(EntityName::new("a").is_ok());
    assert!(EntityName::new("a".repeat(63)).is_ok());
    for invalid in ["", "A", "-a", "a-", "a/b", "a b", "é"] {
        assert!(EntityName::new(invalid).is_err(), "accepted {invalid:?}");
    }
    assert!(EntityName::new("a".repeat(64)).is_err());
}

#[test]
fn no_op_commit_writes_nothing_and_every_publication_statement_rolls_back() {
    let root = temp("commit-atomicity");
    let path = root.join("store.sqlite");
    let store = LayerStackStore::create(&path).unwrap();
    let initialized = store
        .initialize_layerstack(
            EntityName::new("demo").unwrap(),
            LayerStackInitialization::Empty,
        )
        .unwrap();
    let branch_id = store
        .fork_branch(
            EntityName::new("main").unwrap(),
            LocalForkSource::Layer {
                layer_id: initialized.genesis_layer_id,
            },
        )
        .unwrap();
    let pinned = store.pin_branch(branch_id).unwrap();
    let before_counts = store.store_counts().unwrap();
    let before_version = store.data_version().unwrap();
    let before_bytes = std::fs::metadata(&path).unwrap().len();
    let unchanged = apply_changes(&pinned.reader, pinned.root, &[], [1; 32]).unwrap();
    assert_eq!(
        store
            .commit_candidate(
                &pinned.branch,
                pinned.root,
                pinned.branch.base_layer_id,
                unchanged,
            )
            .unwrap(),
        CommitOutcome::UpToDate {
            root_id: pinned.root
        }
    );
    assert_eq!(store.store_counts().unwrap(), before_counts);
    assert_eq!(store.data_version().unwrap(), before_version);
    assert_eq!(std::fs::metadata(&path).unwrap().len(), before_bytes);
    assert_eq!(store_files(&root), vec!["store.sqlite"]);

    // The two logical publication statements have stable fault sentinels;
    // pack, pool catalogue and locator insertion can batch independently of object count.
    for statement in [1, 2, 3, u64::MAX - 1, u64::MAX - 2] {
        let candidate = apply_changes(
            &pinned.reader,
            pinned.root,
            &[ContentChange::Write {
                path: "hello".to_owned(),
                bytes: b"world".to_vec(),
                mode: 0o644,
            }],
            [2; 32],
        )
        .unwrap();
        layerfs_layerstack_store::set_transaction_failure_at(Some(statement));
        let result = store.commit_candidate(
            &pinned.branch,
            pinned.root,
            pinned.branch.base_layer_id,
            candidate,
        );
        layerfs_layerstack_store::set_transaction_failure_at(None);
        assert!(matches!(
            result,
            Err(StoreError::Integrity("injected transaction failure"))
        ));
        assert_eq!(store.store_counts().unwrap(), before_counts);
        assert_eq!(store.branch(branch_id).unwrap().unwrap(), pinned.branch);
    }

    drop(store);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn concurrent_commit_has_one_cas_winner() {
    let root = temp("concurrent-commit");
    let store = std::sync::Arc::new(LayerStackStore::create(root.join("store.sqlite")).unwrap());
    let initialized = store
        .initialize_layerstack(
            EntityName::new("demo").unwrap(),
            LayerStackInitialization::Empty,
        )
        .unwrap();
    let branch_id = store
        .fork_branch(
            EntityName::new("main").unwrap(),
            LocalForkSource::Layer {
                layer_id: initialized.genesis_layer_id,
            },
        )
        .unwrap();
    let pinned = store.pin_branch(branch_id).unwrap();
    let pinned_root = pinned.root;
    let candidates = [b"one".as_slice(), b"two".as_slice()].map(|bytes| {
        apply_changes(
            &pinned.reader,
            pinned_root,
            &[ContentChange::Write {
                path: "winner".to_owned(),
                bytes: bytes.to_vec(),
                mode: 0o644,
            }],
            [3; 32],
        )
        .unwrap()
    });
    let threads = candidates.map(|candidate| {
        let store = store.clone();
        let branch = pinned.branch.clone();
        std::thread::spawn(move || {
            store.commit_candidate(&branch, pinned_root, branch.base_layer_id, candidate)
        })
    });
    let outcomes = threads.map(|thread| thread.join().unwrap());
    assert_eq!(
        outcomes
            .iter()
            .filter(|outcome| matches!(outcome, Ok(CommitOutcome::Committed { .. })))
            .count(),
        1
    );
    assert_eq!(
        outcomes
            .iter()
            .filter(|outcome| matches!(outcome, Err(StoreError::CommitHeadMoved { .. })))
            .count(),
        1
    );
    assert_eq!(store.store_counts().unwrap().commits, 1);

    drop(store);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn visible_missing_and_same_length_corrupt_objects_are_integrity_errors() {
    let root = temp("object-integrity");
    let path = root.join("store.sqlite");
    let store = LayerStackStore::create(&path).unwrap();
    let initialized = store
        .initialize_layerstack(
            EntityName::new("demo").unwrap(),
            LayerStackInitialization::Empty,
        )
        .unwrap();
    let branch_id = store
        .fork_branch(
            EntityName::new("main").unwrap(),
            LocalForkSource::Layer {
                layer_id: initialized.genesis_layer_id,
            },
        )
        .unwrap();
    let pinned = store.pin_branch(branch_id).unwrap();
    let visible_root = pinned.root;
    let canonical = pinned.reader.read_object(visible_root).unwrap();
    drop(pinned);
    drop(store);
    // A valid RAW singleton keeps framing and canonical length intact. Verify
    // its positive control before changing only an authenticated payload byte.
    let mut packed = b"LFPACK\0\0\x01\0\0\0\x01\0\0\0".to_vec();
    for value in [
        32,
        canonical.len() + 9,
        canonical.len() + 9,
        0,
        1,
        canonical.len() + 1,
    ] {
        packed.extend_from_slice(&(value as u32).to_le_bytes());
    }
    packed.push(0);
    packed.extend_from_slice(&canonical);
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection
        .execute("INSERT INTO object_packs(data) VALUES(?1)", [&packed])
        .unwrap();
    let pack_id = connection.last_insert_rowid();
    connection
        .execute(
            "UPDATE objects SET pack_id=?1,group_number=0,record_number=0 WHERE object_id=?2",
            rusqlite::params![pack_id, visible_root.as_bytes().as_slice()],
        )
        .unwrap();
    drop(connection);
    let store = LayerStackStore::connect(&path).unwrap();
    assert_eq!(
        store
            .snapshot_reader(visible_root)
            .read_object(visible_root)
            .unwrap(),
        canonical
    );
    drop(store);
    *packed.last_mut().unwrap() ^= 1;
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection
        .execute(
            "UPDATE object_packs SET data=?1 WHERE pack_id=?2",
            rusqlite::params![packed, pack_id],
        )
        .unwrap();
    drop(connection);
    let store = LayerStackStore::connect(&path).unwrap();
    assert_eq!(
        store
            .snapshot_reader(visible_root)
            .read_object(visible_root),
        Err(StoreError::Integrity("object identity"))
    );
    drop(store);
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection
        .execute(
            "DELETE FROM objects WHERE object_id=?1",
            [visible_root.as_bytes().as_slice()],
        )
        .unwrap();
    drop(connection);
    assert!(matches!(
        LayerStackStore::connect(&path),
        Err(StoreError::Integrity("foreign key check"))
    ));

    std::fs::remove_dir_all(root).unwrap();
}

fn pragma(connection: &rusqlite::Connection, name: &str) -> i64 {
    connection
        .pragma_query_value(None, name, |row| row.get(0))
        .unwrap()
}

fn temp(label: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "layerfs-v4-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&path).unwrap();
    path
}

fn store_files(root: &std::path::Path) -> Vec<String> {
    let mut files = std::fs::read_dir(root)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect::<Vec<_>>();
    files.sort();
    files
}
