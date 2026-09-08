use super::*;
use crate::objects::{CheckedOutputAdmission, PreparedAdmission};
use crate::{CoreReader, ObjectBuffer};
use layerfs_content::file::rope::{self, FileStateRoot};
use layerfs_content::ObjectId;

fn folder() -> PathBuf {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    loop {
        let path = std::env::temp_dir().join(format!(
            "layerfs-format-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        match std::fs::create_dir(&path) {
            Ok(()) => return path,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => panic!("{error}"),
        }
    }
}

fn publish_file(db: &StoreDb, bytes: &[u8]) -> ObjectId {
    let built = ObjectBuffer::build_complete_file(bytes, bytes.len() as u64).unwrap();
    let root = built.root_id;
    let mut owner = CheckedOutputAdmission::new(db).unwrap();
    owner.admit(built.objects).unwrap();
    let finished = owner.finish().unwrap();
    let mut statement_number = finished.statement_number;
    PreparedAdmission::prepare_missing(db, finished.final_batch)
        .unwrap()
        .publish(db, &mut statement_number, |_, _, _| Ok(()))
        .unwrap();
    root
}

fn read_file(db: &StoreDb, root: ObjectId, expected: &[u8]) {
    let canonical = db.read_object_row(root).unwrap();
    layerfs_content::authenticate_identity(&canonical, root).unwrap();
    let mut actual = Vec::new();
    rope::read_all(&CoreReader(db), FileStateRoot(root), &mut actual).unwrap();
    assert_eq!(actual, expected);
}

fn pack_versions(db: &StoreDb) -> BTreeSet<Vec<u8>> {
    db.reader()
        .unwrap()
        .prepare("SELECT DISTINCT substr(data,9,4) FROM object_packs")
        .unwrap()
        .query_map([], |row| row.get(0))
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap()
}

#[test]
fn legacy_format_reads_authenticates_and_writes_without_promotion() {
    let directory = folder();
    let path = directory.join("legacy.sqlite");
    let connection = Connection::open(&path).unwrap();
    connection.execute_batch(statements::schema::V6).unwrap();
    drop(connection);
    let first = vec![b'a'; 8192];
    let second = vec![b'b'; 8192];
    let db = StoreDb::connect(&path).unwrap();
    assert!(!db.native_format());
    let first_root = publish_file(&db, &first);
    read_file(&db, first_root, &first);
    assert_eq!(
        pack_versions(&db),
        BTreeSet::from([1u32.to_le_bytes().to_vec()])
    );
    drop(db);
    let db = StoreDb::connect(&path).unwrap();
    let second_root = publish_file(&db, &second);
    read_file(&db, first_root, &first);
    read_file(&db, second_root, &second);
    assert_eq!(
        pack_versions(&db),
        BTreeSet::from([1u32.to_le_bytes().to_vec()])
    );
    assert_eq!(
        db.reader()
            .unwrap()
            .pragma_query_value(None, "user_version", |row| row.get::<_, i64>(0))
            .unwrap(),
        LEGACY_SCHEMA_VERSION
    );
    drop(db);
    let db = StoreDb::connect(&path).unwrap();
    read_file(&db, first_root, &first);
    read_file(&db, second_root, &second);
    drop(db);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn native_format_reopens_writes_and_retains_mixed_dependency_reads() {
    let directory = folder();
    let path = directory.join("native.sqlite");
    let first = vec![b'a'; 8192];
    let second = vec![b'b'; 8192];
    let db = StoreDb::create(&path).unwrap();
    assert!(db.native_format());
    let first_root = publish_file(&db, &first);
    // File-state/map structure is legacy; authenticated file chunks are native.
    assert_eq!(
        pack_versions(&db),
        BTreeSet::from([1u32.to_le_bytes().to_vec(), 2u32.to_le_bytes().to_vec()])
    );
    read_file(&db, first_root, &first);
    drop(db);
    let db = StoreDb::connect(&path).unwrap();
    assert!(db.native_format());
    let second_root = publish_file(&db, &second);
    assert_ne!(first_root, second_root);
    let count: i64 = db
        .reader()
        .unwrap()
        .query_row("SELECT count(*) FROM objects", [], |row| row.get(0))
        .unwrap();
    assert_eq!(publish_file(&db, &first), first_root);
    assert_eq!(
        db.reader()
            .unwrap()
            .query_row("SELECT count(*) FROM objects", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        count
    );
    read_file(&db, first_root, &first);
    read_file(&db, second_root, &second);
    drop(db);
    let db = StoreDb::connect(&path).unwrap();
    read_file(&db, first_root, &first);
    read_file(&db, second_root, &second);
    drop(db);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn unsupported_versions_and_native_header_downgrade_reject_without_mutation() {
    let directory = folder();
    for version in [LEGACY_SCHEMA_VERSION, SCHEMA_VERSION + 1, 0] {
        let path = directory.join(format!("{version}.sqlite"));
        let db = StoreDb::create(&path).unwrap();
        publish_file(&db, &vec![b'n'; 8192]);
        assert!(pack_versions(&db).contains(&2u32.to_le_bytes().to_vec()));
        drop(db);
        // Disposable malformed fixtures only: this is not a supported conversion.
        let connection = Connection::open(&path).unwrap();
        connection
            .pragma_update(None, "user_version", version)
            .unwrap();
        drop(connection);
        let before = std::fs::read(&path).unwrap();
        assert!(matches!(
            StoreDb::connect(&path),
            Err(StoreError::WrongStoreSchema)
        ));
        assert_eq!(std::fs::read(&path).unwrap(), before);
        assert!(!appended(&path, "-journal").exists());
        assert!(!appended(&path, "-wal").exists());
        assert!(!appended(&path, "-shm").exists());
    }
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn legacy_open_contract_rejects_native_store_before_writer_configuration() {
    let directory = folder();
    let path = directory.join("native.sqlite");
    let db = StoreDb::create(&path).unwrap();
    publish_file(&db, &vec![b'n'; 8192]);
    drop(db);
    let before = std::fs::read(&path).unwrap();
    let connection = Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
    // The released legacy open contract requires exactly user_version6. Test
    // that boundary independently of the new dual-version preflight.
    assert!(matches!(
        verify_schema(&connection, LEGACY_SCHEMA_VERSION),
        Err(StoreError::WrongStoreSchema)
    ));
    drop(connection);
    assert_eq!(std::fs::read(&path).unwrap(), before);
    std::fs::remove_dir_all(directory).unwrap();
}
