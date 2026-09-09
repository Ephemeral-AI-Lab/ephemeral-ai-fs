use super::*;

struct Fixture {
    db: crate::schema::StoreDb,
    root: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = loop {
            let root = std::env::temp_dir().join(format!(
                "layerfs-comparison-reuse-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match std::fs::create_dir(&root) {
                Ok(()) => break root,
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("{error}"),
            }
        };
        Self {
            db: crate::schema::StoreDb::create(root.join("store.sqlite")).unwrap(),
            root,
        }
    }

    fn retain(&self, objects: Vec<AuthenticatedCanonicalObject>) {
        let mut owner = CheckedOutputAdmission::new(&self.db).unwrap();
        owner.admit_page(objects).unwrap();
        let finished = owner.finish().unwrap();
        PreparedAdmission::prepare_missing(&self.db, finished.final_batch)
            .unwrap()
            .publish(&self.db, &mut 0, |_, _, _| Ok(()))
            .unwrap();
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn chunk(seed: u64) -> AuthenticatedCanonicalObject {
    let mut state = seed + 1;
    let raw = (0..32_768)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state as u8
        })
        .collect::<Vec<_>>();
    let mut object = AuthenticatedCanonicalObject::new(
        layerfs_content::file::extent_codec::encode_chunk_object(&raw).unwrap(),
        None,
    )
    .unwrap();
    object.1.diagnostic = diagnostic::FILE | diagnostic::COMPLETE;
    object
}

#[test]
fn comparison_reuse_reads_once_and_checks_every_occurrence_and_location() {
    let fixture = Fixture::new();
    let original = chunk(1);
    fixture.retain(vec![original.clone()]);
    let mut owner = CheckedOutputAdmission::new_for_initialization(&fixture.db).unwrap();

    let before = fixture.db.physical_storage_receipt();
    owner.admit_page(vec![original.clone()]).unwrap();
    owner.flush().unwrap();
    assert!(
        fixture
            .db
            .physical_storage_receipt()
            .since(before)
            .native_decode_calls
            > 0
    );
    assert_eq!(owner.compared.len(), 1);

    let before = fixture.db.physical_storage_receipt();
    owner.admit_page(vec![original.clone()]).unwrap();
    owner.flush().unwrap();
    assert_eq!(
        fixture
            .db
            .physical_storage_receipt()
            .since(before)
            .native_decode_calls,
        0
    );
    assert_eq!(owner.checked.candidate_objects, 1);
    assert_eq!(owner.checked.reused_objects, 1);
    assert_eq!(owner.diagnostics.collision_checks, 2);

    // A stale physical association must force storage validation, even if its
    // retained operand is deliberately corrupted through this private test seam.
    let retained = owner.compared.get_mut(&original.id).unwrap();
    retained.0.record += 1;
    retained.1[0] ^= 1;
    let before = fixture.db.physical_storage_receipt();
    owner.admit_page(vec![original.clone()]).unwrap();
    owner.flush().unwrap();
    assert!(
        fixture
            .db
            .physical_storage_receipt()
            .since(before)
            .native_decode_calls
            > 0
    );
    assert_eq!(owner.compared[&original.id].1, original.bytes);

    let mut collision = original.clone();
    collision.0.bytes = chunk(2).0.bytes;
    assert_eq!(collision.bytes.len(), original.bytes.len());
    let session = owner.session();
    assert!(matches!(
        session.resolve(
            owner
                .admit_page(vec![collision])
                .and_then(|_| owner.flush())
        ),
        Err(StoreError::Integrity("object collision"))
    ));
    assert_eq!(
        fixture.db.read_object_row(original.id).unwrap(),
        original.bytes
    );
}

#[test]
fn comparison_reuse_authenticates_storage_before_retaining_an_operand() {
    let fixture = Fixture::new();
    let original = chunk(3);
    let other = chunk(4);
    fixture.retain(vec![original.clone(), other.clone()]);
    let other_location = fixture.db.object_locations(&[other.id]).unwrap()[&other.id];
    // Test-only corruption: the locator names another valid, same-length frame.
    fixture
        .db
        .writer()
        .unwrap()
        .execute(
            "UPDATE objects SET pack_id=?1,group_number=?2,record_number=?3 WHERE object_id=?4",
            rusqlite::params![
                other_location.pack,
                other_location.group as i64,
                other_location.record as i64,
                original.id.as_bytes().as_slice()
            ],
        )
        .unwrap();
    let mut owner = CheckedOutputAdmission::new_for_initialization(&fixture.db).unwrap();
    let session = owner.session();
    assert!(session
        .resolve(owner.admit_page(vec![original]).and_then(|_| owner.flush()))
        .is_err());
    assert!(owner.compared.is_empty());
    assert_eq!(owner.compared_bytes, 0);
}

#[test]
fn comparison_reuse_bounds_resident_operands_and_eviction_keeps_exact_checks() {
    let fixture = Fixture::new();
    let mut owner = CheckedOutputAdmission::new_for_initialization(&fixture.db).unwrap();
    let first = chunk(10);
    let mut saw_eviction = false;
    for seed in 10..90 {
        let object = chunk(seed);
        owner.admit_page(vec![object.clone()]).unwrap();
        owner.flush().unwrap();
        owner.admit_page(vec![object]).unwrap();
        owner.flush().unwrap();
        assert!(owner.compared_bytes <= COMPARISON_REUSE_BYTES);
        assert_eq!(
            owner.compared_bytes,
            owner
                .compared
                .values()
                .map(|(_, bytes)| bytes.capacity() + COMPARISON_REUSE_ENTRY_BYTES)
                .sum::<usize>()
        );
        saw_eviction |= !owner.compared.contains_key(&first.id);
    }
    assert!(saw_eviction);
    assert!(!owner.compared.contains_key(&first.id));
    let before = fixture.db.physical_storage_receipt();
    owner.admit_page(vec![first.clone()]).unwrap();
    owner.flush().unwrap();
    assert!(
        fixture
            .db
            .physical_storage_receipt()
            .since(before)
            .native_decode_calls
            > 0
    );
    assert_eq!(owner.compared[&first.id].1, first.bytes);
    owner.abort().unwrap();
}

#[test]
fn comparison_reuse_single_operand_boundary_reserves_a_complete_tree_node() {
    let fixture = Fixture::new();
    let mut owner = CheckedOutputAdmission::new_for_initialization(&fixture.db).unwrap();
    let payload_limit = COMPARISON_REUSE_BYTES - COMPARISON_REUSE_ENTRY_BYTES;
    let first = ObjectId::for_bytes(b"comparison allocation boundary");
    let second = ObjectId::for_bytes(b"comparison allocation overflow");
    let location = read::Location {
        canonical_length: payload_limit,
        pack: 1,
        group: 0,
        record: 0,
    };
    // This private seam tests allocation accounting, not canonical admission.
    let bytes = vec![0; payload_limit];
    assert_eq!(bytes.capacity(), payload_limit);
    owner.remember_comparison(first, location, bytes);
    assert_eq!(owner.compared.len(), 1);
    assert_eq!(owner.compared_bytes, COMPARISON_REUSE_BYTES);

    owner.remember_comparison(second, location, vec![0; payload_limit + 1]);
    assert_eq!(owner.compared.len(), 1);
    assert!(owner.compared.contains_key(&first));
    assert_eq!(owner.compared_bytes, COMPARISON_REUSE_BYTES);

    // Spare Vec capacity is owned memory even when its logical length is tiny.
    let mut spare_capacity = Vec::with_capacity(payload_limit + 1);
    spare_capacity.push(0);
    owner.remember_comparison(second, location, spare_capacity);
    assert!(!owner.compared.contains_key(&second));
    assert_eq!(owner.compared_bytes, COMPARISON_REUSE_BYTES);

    owner.remember_comparison(second, location, vec![0]);
    assert_eq!(owner.compared.len(), 1);
    assert!(!owner.compared.contains_key(&first));
    assert_eq!(owner.compared_bytes, COMPARISON_REUSE_ENTRY_BYTES + 1);
    owner.abort().unwrap();
}

#[test]
fn comparison_reuse_collision_rolls_back_pending_and_committed_private_batches() {
    let fixture = Fixture::new();
    let baseline = chunk(100);
    fixture.retain(vec![baseline.clone()]);
    let committed = chunk(101);
    let pending = chunk(102);
    let mut owner = CheckedOutputAdmission::new_for_initialization(&fixture.db).unwrap();
    owner.admit_page(vec![committed.clone()]).unwrap();
    owner.flush().unwrap();
    owner.commit_pending().unwrap();
    assert!(fixture.db.reader().unwrap().is_autocommit());
    owner
        .admit_page(vec![committed.clone(), pending.clone()])
        .unwrap();
    owner.flush().unwrap();
    assert!(!fixture.db.reader().unwrap().is_autocommit());
    assert!(owner.compared.contains_key(&committed.id));
    let mut collision = committed.clone();
    collision.0.bytes = pending.bytes.clone();
    let session = owner.session();
    assert!(matches!(
        session.resolve(
            owner
                .admit_page(vec![collision])
                .and_then(|_| owner.flush())
        ),
        Err(StoreError::Integrity("object collision"))
    ));
    assert!(fixture.db.reader().unwrap().is_autocommit());
    assert!(fixture
        .db
        .object_locations(&[committed.id, pending.id])
        .unwrap()
        .is_empty());
    assert_eq!(
        fixture.db.read_object_row(baseline.id).unwrap(),
        baseline.bytes
    );
    assert!(matches!(
        owner.admit_page(vec![committed]),
        Err(StoreError::Integrity("admission session closed"))
    ));
}
