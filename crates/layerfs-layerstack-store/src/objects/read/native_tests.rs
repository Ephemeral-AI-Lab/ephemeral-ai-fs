use super::*;

struct Fixture {
    db: StoreDb,
    folder: std::path::PathBuf,
}
impl Fixture {
    fn new() -> Self {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let folder = loop {
            let sequence = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let folder = std::env::temp_dir().join(format!(
                "layerfs-native-reader-{}-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos(),
                sequence,
            ));
            match std::fs::create_dir(&folder) {
                Ok(()) => break folder,
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("create exclusive native fixture: {error}"),
            }
        };
        let db = StoreDb::create(folder.join("store.sqlite")).unwrap();
        Self { db, folder }
    }
    fn insert(&self, pack_id: i64, bytes: &[u8], rows: &[(ObjectId, usize, usize)]) {
        let connection = self.db.reader().unwrap();
        connection
            .execute(
                "INSERT INTO object_packs(pack_id,data) VALUES (?,?)",
                rusqlite::params![pack_id, bytes],
            )
            .unwrap();
        for (id, length, record) in rows {
            connection.execute("INSERT INTO objects(object_id,canonical_length,pack_id,group_number,record_number) VALUES (?,?,?,0,?)", rusqlite::params![id.as_bytes().as_slice(), *length as i64, pack_id, *record as i64]).unwrap();
        }
    }
    fn read(&self, id: ObjectId) -> Result<Vec<u8>> {
        let mut locations = self
            .db
            .object_locations(&[id])?
            .into_iter()
            .collect::<Vec<_>>();
        let mut result = None;
        self.db.visit_locations(&mut locations, |value| {
            result = Some(value.bytes);
            Ok(())
        })?;
        result.ok_or(StoreError::Integrity("test missing object"))
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.folder);
    }
}
fn canonical(raw: &[u8]) -> (ObjectId, Vec<u8>) {
    let bytes = layerfs_content::file::extent_codec::encode_chunk_object(raw).unwrap();
    (ObjectId::for_bytes(&bytes), bytes)
}
fn record(raw: &[u8], prefix: Option<(ObjectId, &[u8])>) -> Vec<u8> {
    let frame = pack::native_compress(raw, prefix.map(|(_, raw)| raw)).unwrap();
    pack::native_encode_record(raw.len(), prefix.map(|(id, _)| id), &frame).unwrap()
}
fn native_pack(records: &[&[u8]]) -> Vec<u8> {
    pack::assemble_native(&[pack::native_group(records).unwrap()]).unwrap()
}

#[test]
fn native_reader_point_scope_and_exact_ranges() {
    let f = Fixture::new();
    let values = [b"first".as_slice(), b"middle", b"last"];
    let records = values.map(|raw| record(raw, None));
    let ids = values.map(canonical);
    let packed = native_pack(&records.iter().map(Vec::as_slice).collect::<Vec<_>>());
    f.insert(
        1,
        &packed,
        &ids.iter()
            .enumerate()
            .map(|(i, (id, bytes))| (*id, bytes.len(), i))
            .collect::<Vec<_>>(),
    );
    for index in 0..3 {
        let before = f.db.physical_storage_receipt();
        assert_eq!(f.read(ids[index].0).unwrap(), ids[index].1);
        let work = f.db.physical_storage_receipt().since(before);
        assert_eq!(work.blob_ranges, 5);
        assert_eq!(
            work.native_request_bytes,
            (32 + 4 + 12 + records[index].len()) as u64
        );
        assert_eq!(
            work.native_parser_bytes,
            (4 + 12 + records[index].len()) as u64
        );
        assert_eq!(work.encoded_read_bytes, work.native_parser_bytes);
        assert_eq!(work.decoded_read_bytes, 0);
        assert_eq!(work.native_depth_0, 1);
    }
    // Point scope validates all ends but deliberately not unrelated record bodies.
    let mut bad_neighbor = packed.clone();
    bad_neighbor[32 + 4 + 12] = 255;
    f.db.reader()
        .unwrap()
        .execute(
            "UPDATE object_packs SET data=? WHERE pack_id=1",
            [&bad_neighbor],
        )
        .unwrap();
    assert_eq!(f.read(ids[1].0).unwrap(), ids[1].1);
    assert!(f.read(ids[0].0).is_err());
    bad_neighbor[36..40].copy_from_slice(&0u32.to_le_bytes());
    f.db.reader()
        .unwrap()
        .execute(
            "UPDATE object_packs SET data=? WHERE pack_id=1",
            [&bad_neighbor],
        )
        .unwrap();
    assert!(f.read(ids[1].0).is_err());
}

#[test]
fn native_reader_depth_budget_and_legacy_hint_separation() {
    let f = Fixture::new();
    let mut previous: Option<(ObjectId, Vec<u8>)> = None;
    let mut last = None;
    for index in 0..6 {
        let mut raw = vec![b'x'; pack::NATIVE_RAW_LIMIT];
        raw[index] = index as u8;
        let (id, bytes) = canonical(&raw);
        let encoded = record(
            &raw,
            previous.as_ref().map(|(id, raw)| (*id, raw.as_slice())),
        );
        f.insert(
            index as i64 + 1,
            &native_pack(&[&encoded]),
            &[(id, bytes.len(), 0)],
        );
        if index <= 4 {
            assert_eq!(f.read(id).unwrap(), bytes);
            let mut budget = HintReadBudget::default();
            budget.begin_target();
            match f.db.read_native_prior(id, &mut budget).unwrap() {
                NativePriorOutcome::Available {
                    depth,
                    raw_closure,
                    canonical,
                    location,
                } => {
                    assert_eq!(depth, index as u8);
                    assert_eq!(raw_closure, (index + 1) * pack::NATIVE_RAW_LIMIT);
                    assert_eq!(canonical.bytes, bytes);
                    assert_eq!(location.pack, index as i64 + 1);
                    assert_eq!(budget.target_fetches, index + 1);
                }
                _ => panic!("native prior must be available"),
            }
            budget.begin_target();
            assert!(f.db.read_hint(id, false, &mut budget).unwrap().is_none());
        } else {
            assert!(f.read(id).is_err());
        }
        last = Some(id);
        previous = Some((id, raw));
    }
    let mut budget = HintReadBudget {
        batch_encoded: 8 * 1024 * 1024,
        ..Default::default()
    };
    budget.begin_target();
    assert!(matches!(
        f.db.read_native_prior(last.unwrap(), &mut budget).unwrap(),
        NativePriorOutcome::Budget
    ));
    budget.begin_target();
    let before = f.db.physical_storage_receipt();
    assert!(matches!(
        f.db.read_native_prior(last.unwrap(), &mut budget).unwrap(),
        NativePriorOutcome::Budget
    ));
    assert_eq!(
        f.db.physical_storage_receipt().since(before).base_fetches,
        0
    );
}

#[test]
fn native_reader_missing_same_pack_and_wrong_identity_reject() {
    for same_pack in [false, true] {
        let f = Fixture::new();
        let raw = b"target";
        let (id, bytes) = canonical(raw);
        let base = if same_pack {
            id
        } else {
            ObjectId::for_bytes(b"absent")
        };
        let encoded = record(raw, Some((base, b"prior")));
        f.insert(1, &native_pack(&[&encoded]), &[(id, bytes.len(), 0)]);
        assert!(f.read(id).is_err());
        let mut budget = HintReadBudget::default();
        budget.begin_target();
        assert!(f.db.read_native_prior(id, &mut budget).is_err());
    }
    let f = Fixture::new();
    let (_, bytes) = canonical(b"actual");
    let wrong = canonical(b"wrong!").0;
    f.insert(
        1,
        &native_pack(&[&record(b"actual", None)]),
        &[(wrong, bytes.len(), 0)],
    );
    assert!(f.read(wrong).is_err());
}

fn legacy_pack(record: &[u8]) -> Vec<u8> {
    let mut bytes = 1u32.to_le_bytes().to_vec();
    bytes.extend_from_slice(&(record.len() as u32).to_le_bytes());
    bytes.extend_from_slice(record);
    let length = bytes.len();
    pack::assemble(&[pack::EncodedGroup {
        bytes,
        decoded_length: length,
        records: 1,
        codec: pack::Codec::Raw,
    }])
    .unwrap()
}

#[test]
fn native_reader_legacy_full_root_and_delta_rejection() {
    let f = Fixture::new();
    let raw = vec![b'a'; 4096];
    let (base, base_canonical) = canonical(&raw);
    let mut full = vec![0];
    full.extend_from_slice(&base_canonical);
    f.insert(1, &legacy_pack(&full), &[(base, base_canonical.len(), 0)]);
    let mut next = raw.clone();
    next[4] = b'b';
    let (target, target_canonical) = canonical(&next);
    f.insert(
        2,
        &native_pack(&[&record(&next, Some((base, &raw)))]),
        &[(target, target_canonical.len(), 0)],
    );
    assert_eq!(f.read(target).unwrap(), target_canonical);
    let mut budget = HintReadBudget::default();
    budget.begin_target();
    assert!(matches!(
        f.db.read_native_prior(base, &mut budget).unwrap(),
        NativePriorOutcome::Available { depth: 0, .. }
    ));
    // A legacy custom DELTA remains unsupported for optional native roots.
    let mut delta_target_raw = raw.clone();
    delta_target_raw[5] = b'c';
    let (delta_id, delta_canonical) = canonical(&delta_target_raw);
    let delta = pack::delta_record(
        base,
        &base_canonical,
        &delta_canonical,
        &mut (16 * 1024 * 1024),
        &mut PhysicalStorageReceipt::default(),
    )
    .unwrap()
    .unwrap();
    f.insert(
        3,
        &legacy_pack(&delta),
        &[(delta_id, delta_canonical.len(), 0)],
    );
    assert_eq!(f.read(delta_id).unwrap(), delta_canonical);
    budget.begin_target();
    assert!(matches!(
        f.db.read_native_prior(delta_id, &mut budget).unwrap(),
        NativePriorOutcome::UnsupportedLegacyDelta
    ));
    let mut child_raw = raw.clone();
    child_raw[6] = b'd';
    let (child, child_canonical) = canonical(&child_raw);
    f.insert(
        4,
        &native_pack(&[&record(&child_raw, Some((delta_id, &delta_target_raw)))]),
        &[(child, child_canonical.len(), 0)],
    );
    assert!(f.read(child).is_err());
    // Nor can old DELTA grammar use a native FULL/prefix as its legacy FULL anchor.
    let mut other_raw = raw.clone();
    other_raw[7] = b'e';
    let (other, other_canonical) = canonical(&other_raw);
    let invalid_delta = pack::delta_record(
        target,
        &target_canonical,
        &other_canonical,
        &mut (16 * 1024 * 1024),
        &mut PhysicalStorageReceipt::default(),
    )
    .unwrap()
    .unwrap();
    f.insert(
        5,
        &legacy_pack(&invalid_delta),
        &[(other, other_canonical.len(), 0)],
    );
    assert!(f.read(other).is_err());
    let path = f.db.path().to_owned();
    let folder = f.folder.clone();
    // Keep the fixture directory while releasing the exclusive Store owner.
    let temporary = StoreDb::create(folder.join("owner-swap.sqlite")).unwrap();
    let mut f = f;
    drop(std::mem::replace(&mut f.db, temporary));
    let db = StoreDb::connect(&path).unwrap();
    assert_eq!(db.read_object_row(target).unwrap(), target_canonical);
    assert_eq!(db.read_object_row(base).unwrap(), base_canonical);
    assert_eq!(db.read_object_row(delta_id).unwrap(), delta_canonical);
    assert!(db.read_object_row(child).is_err());
    assert!(db.read_object_row(other).is_err());
    drop(db);
    drop(f);
}

#[test]
fn native_reader_maximum_owned_capacity_bound() {
    let records = (0..5)
        .map(|_| vec![0u8; pack::NATIVE_FRAME_LIMIT + 37])
        .collect::<Vec<_>>();
    let nodes = Vec::<NativeNode>::with_capacity(5);
    let maximum = records.iter().map(Vec::capacity).sum::<usize>()
        + nodes.capacity() * std::mem::size_of::<NativeNode>()
        + 2 * pack::GROUP_LIMIT
        + 3 * (pack::NATIVE_RAW_LIMIT + 21)
        + 4 * pack::RECORD_COUNT_LIMIT
        + pack::NATIVE_DECODE_WORKSPACE;
    assert!(maximum < VALIDATION_RESERVE, "actual capacities={maximum}");
    assert_eq!(records.len(), 5);
}

#[test]
fn native_reader_empty_chunk_same_pack_and_cross_role() {
    let f = Fixture::new();
    let (empty, empty_canonical) = canonical(b"");
    f.insert(
        1,
        &native_pack(&[&record(b"", None)]),
        &[(empty, empty_canonical.len(), 0)],
    );
    assert_eq!(f.read(empty).unwrap(), empty_canonical);
    let (base, base_canonical) = canonical(b"base");
    let (child, child_canonical) = canonical(b"child");
    let base_record = record(b"base", None);
    let child_record = record(b"child", Some((base, b"base")));
    f.insert(
        2,
        &native_pack(&[&base_record, &child_record]),
        &[
            (base, base_canonical.len(), 0),
            (child, child_canonical.len(), 1),
        ],
    );
    assert!(f.read(child).is_err());
    let structural = layerfs_content::encode_bytes_object(b"this is not a payload chunk").unwrap();
    let structural_id = ObjectId::for_bytes(&structural);
    let mut full = vec![0];
    full.extend_from_slice(&structural);
    f.insert(
        3,
        &legacy_pack(&full),
        &[(structural_id, structural.len(), 0)],
    );
    let mut budget = HintReadBudget::default();
    budget.begin_target();
    assert!(matches!(
        f.db.read_native_prior(structural_id, &mut budget).unwrap(),
        NativePriorOutcome::UnsupportedRole
    ));
    let (other, other_canonical) = canonical(b"other");
    f.insert(
        4,
        &native_pack(&[&record(b"other", Some((structural_id, b"unused")))]),
        &[(other, other_canonical.len(), 0)],
    );
    assert!(f.read(other).is_err());
    budget.begin_target();
    assert!(matches!(
        f.db.read_native_prior(ObjectId::for_bytes(b"missing"), &mut budget)
            .unwrap(),
        NativePriorOutcome::Unavailable
    ));
}

#[test]
fn native_batch_shares_directory_but_only_reads_requested_bodies() {
    let f = Fixture::new();
    let raw = [
        b"unrequested".as_slice(),
        b"requested one",
        b"requested two",
    ];
    let records = raw.map(|bytes| record(bytes, None));
    let ids = raw.map(canonical);
    let mut packed = native_pack(&records.iter().map(Vec::as_slice).collect::<Vec<_>>());
    f.insert(
        1,
        &packed,
        &ids.iter()
            .enumerate()
            .map(|(i, (id, bytes))| (*id, bytes.len(), i))
            .collect::<Vec<_>>(),
    );
    let read = || -> Result<Vec<CanonicalObject>> {
        let mut locations =
            f.db.object_locations(&[ids[1].0, ids[2].0])?
                .into_iter()
                .collect::<Vec<_>>();
        let mut values = Vec::new();
        f.db.visit_locations(&mut locations, |object| {
            values.push(object);
            Ok(())
        })?;
        Ok(values)
    };
    let before = f.db.physical_storage_receipt();
    let values = read().unwrap();
    assert_eq!(
        values
            .iter()
            .map(|v| v.bytes.as_slice())
            .collect::<Vec<_>>(),
        vec![ids[1].1.as_slice(), ids[2].1.as_slice()]
    );
    let work = f.db.physical_storage_receipt().since(before);
    assert_eq!(work.group_fetches, 1);
    assert_eq!(work.native_record_fetches, 2);
    assert_eq!(work.blob_ranges, 6); // header, directory, count, ends, two bodies
    assert_eq!(
        work.native_request_bytes,
        (32 + 4 + 12 + records[1].len() + records[2].len()) as u64
    );
    // Native records start after the 32-byte pack framing and 16-byte group directory.
    packed[48] = 255;
    f.db.reader()
        .unwrap()
        .execute("UPDATE object_packs SET data=? WHERE pack_id=1", [&packed])
        .unwrap();
    assert!(read().is_ok());
    assert!(f.read(ids[0].0).is_err());
    f.db.reader()
        .unwrap()
        .execute(
            "UPDATE objects SET record_number=99 WHERE object_id=?",
            [ids[2].0.as_bytes().as_slice()],
        )
        .unwrap();
    assert!(read().is_err());
    f.db.reader()
        .unwrap()
        .execute(
            "UPDATE objects SET record_number=2 WHERE object_id=?",
            [ids[2].0.as_bytes().as_slice()],
        )
        .unwrap();
    packed[36..40].copy_from_slice(&0u32.to_le_bytes());
    f.db.reader()
        .unwrap()
        .execute("UPDATE object_packs SET data=? WHERE pack_id=1", [&packed])
        .unwrap();
    assert!(read().is_err());
    assert!(f.db.extract_demanded_group(1, 0, &[]).is_err());
    let location = f.db.object_locations(&[ids[1].0]).unwrap()[&ids[1].0];
    assert!(f
        .db
        .extract_demanded_group(1, 0, &vec![(ids[1].0, location); OBJECT_PAGE_COUNT + 1])
        .is_err());
}
