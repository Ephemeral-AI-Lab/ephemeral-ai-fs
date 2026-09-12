use super::*;
use layerfs_content::tree::inode::{InodeKind, InodeRecordV1};
use rusqlite::limits::Limit;

struct Fixture {
    db: StoreDb,
    folder: std::path::PathBuf,
}

impl Fixture {
    fn new() -> Self {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let folder = std::env::temp_dir().join(format!(
            "metadata-fingerprint-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        ));
        std::fs::create_dir(&folder).unwrap();
        Self {
            db: StoreDb::create(folder.join("store.sqlite")).unwrap(),
            folder,
        }
    }

    // Actual schema10 authenticated groups isolate the index contract. Complete
    // inode/dependency admission and publication are covered by admission tests.
    fn append(&self, values: &[[u8; 73]], copies: usize) -> u64 {
        let canonical = values
            .iter()
            .map(value_canonical)
            .collect::<Result<Vec<_>>>()
            .unwrap();
        let slices = canonical.iter().map(Vec::as_slice).collect::<Vec<_>>();
        let (group, mixed) =
            pack::encode_group(&slices, &vec![None; values.len()], &mut Default::default())
                .unwrap();
        assert!(!mixed);
        let body = pack::decode_group(
            pack::GroupEntry {
                range: 0..group.bytes.len(),
                decoded_length: group.decoded_length,
                codec: group.codec,
                oversized: false,
            },
            group.bytes.clone(),
        )
        .unwrap();
        let digest = ObjectId::for_bytes(&body);
        let mut bytes = pack::assemble(&[group]).unwrap();
        bytes[8..12].copy_from_slice(&6u32.to_le_bytes());
        let first = self.db.next_metadata_ordinal().unwrap();
        let mut connection = self.db.reader().unwrap();
        let transaction = connection.transaction().unwrap();
        for number in 0..copies {
            let ordinal = first + (number * values.len()) as u64;
            transaction
                .execute(
                    "INSERT INTO object_packs(pack_id,data) VALUES (?1,?2)",
                    rusqlite::params![ordinal as i64, &bytes],
                )
                .unwrap();
            transaction.execute(
                "INSERT INTO metadata_value_groups(first_ordinal,count,pack_id,group_number,digest) VALUES (?1,?2,?1,0,?3)",
                rusqlite::params![ordinal as i64,values.len() as i64,digest.as_bytes().as_slice()],
            ).unwrap();
        }
        transaction.commit().unwrap();
        first
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.folder);
    }
}

fn value(number: u64) -> [u8; 73] {
    compact::encode_inode_value(InodeRecordV1 {
        kind: InodeKind::RegularFile,
        namespace_ref_count: number + 1,
        content_root: ObjectId::for_bytes(b"index test content"),
        metadata_root: ObjectId::for_bytes(b"index test metadata"),
    })
}

#[test]
fn metadata_fingerprint_collisions_compare_all_bytes_and_choose_first_duplicate() {
    let f = Fixture::new();
    let a = value(1);
    let b = value(2);
    f.append(&[a, b, a], 1);
    let mut index = ValueIndex::new().unwrap();
    index.sync(&f.db).unwrap();
    // Force a false-positive filter hit without changing canonical group bytes.
    index
        .connection
        .execute(
            "UPDATE values_by_fingerprint SET fingerprint=?1 WHERE ordinal=2",
            [fingerprint(&a)],
        )
        .unwrap();
    let before = f.db.physical_storage_receipt();
    let found = index.find_batch(&f.db, &[a, a]).unwrap();
    assert_eq!(found, BTreeMap::from([(a, 1)]));
    let receipt = f.db.physical_storage_receipt().since(before);
    assert_eq!(receipt.metadata_pool_group_fetches, 1);
    let absent = value(999);
    index
        .connection
        .execute(
            "UPDATE values_by_fingerprint SET fingerprint=?1",
            [fingerprint(&absent)],
        )
        .unwrap();
    assert!(index.find_batch(&f.db, &[absent]).unwrap().is_empty());
}

#[test]
fn metadata_fingerprint_global_query_dedup_and_low_parameter_limit_bound_visits() {
    let f = Fixture::new();
    let values = (0..330).map(value).collect::<Vec<_>>();
    f.append(&values[..165], 1);
    f.append(&values[165..], 1);
    let mut index = ValueIndex::new().unwrap();
    index.sync(&f.db).unwrap();
    index
        .connection
        .set_limit(Limit::SQLITE_LIMIT_VARIABLE_NUMBER, 2)
        .unwrap();
    let queries = values
        .iter()
        .copied()
        .chain(values.iter().rev().copied())
        .chain(std::iter::repeat_n(values[0], 1025))
        .collect::<Vec<_>>();
    let before = f.db.physical_storage_receipt();
    let found = index.find_batch(&f.db, &queries).unwrap();
    assert_eq!(found.len(), values.len());
    for (number, value) in values.iter().enumerate() {
        assert_eq!(found[value], number as u32 + 1);
    }
    let receipt = f.db.physical_storage_receipt().since(before);
    assert_eq!(receipt.metadata_pool_group_fetches, 2);
    assert!(index.find_batch(&f.db, &[]).unwrap().is_empty());
}

#[test]
fn metadata_fingerprint_real_duplicate_heavy_eviction_keeps_bounded_exact_window() {
    let f = Fixture::new();
    let old = value(1);
    let repeated = value(2);
    f.append(&[old], 1);
    f.append(&vec![repeated; 165], 795);
    let mut index = ValueIndex::new().unwrap();
    index.sync(&f.db).unwrap();
    assert_eq!(index.entries, 165);
    let rows: i64 = index
        .connection
        .query_row("SELECT COUNT(*) FROM values_by_fingerprint", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(
        rows, 165,
        "equal fingerprints retain every ordinal in the current window"
    );
    let pages: i64 = index
        .connection
        .pragma_query_value(None, "page_count", |r| r.get(0))
        .unwrap();
    let page_size: i64 = index
        .connection
        .pragma_query_value(None, "page_size", |r| r.get(0))
        .unwrap();
    assert_eq!(page_size, 4096);
    assert!(pages * page_size <= 32 * 1024 * 1024);
    let before = f.db.physical_storage_receipt();
    assert_eq!(
        index.find_batch(&f.db, &[old, repeated]).unwrap(),
        BTreeMap::from([(repeated, 131012)])
    );
    let receipt = f.db.physical_storage_receipt().since(before);
    assert_eq!(receipt.metadata_pool_group_fetches, 1);
    index.sync(&f.db).unwrap();
}

#[test]
fn metadata_fingerprint_matches_never_bypass_corrupt_or_missing_groups() {
    for missing in [false, true] {
        let f = Fixture::new();
        let a = value(1);
        f.append(&[a, value(2)], 1);
        let mut index = ValueIndex::new().unwrap();
        index.sync(&f.db).unwrap();
        if missing {
            f.db.reader()
                .unwrap()
                .execute(
                    "DELETE FROM metadata_value_groups WHERE first_ordinal=1",
                    [],
                )
                .unwrap();
        } else {
            f.db.reader()
                .unwrap()
                .execute(
                    "UPDATE metadata_value_groups SET digest=zeroblob(32) WHERE first_ordinal=1",
                    [],
                )
                .unwrap();
        }
        assert!(
            index.find_batch(&f.db, &[a]).is_err(),
            "filter hit still requires current group authentication"
        );
    }
}

fn fingerprint_rows(index: &ValueIndex) -> Vec<(i64, u32)> {
    index
        .connection
        .prepare("SELECT fingerprint,ordinal FROM values_by_fingerprint ORDER BY ordinal")
        .unwrap()
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap()
}

#[test]
#[ignore = "explicit production metadata-retention history proof"]
fn metadata_fingerprint_reopen_tail_matches_original_replay_and_bounds_payload_work() {
    let mut f = Fixture::new();
    let values = (0..165).map(value).collect::<Vec<_>>();
    let mut groups = 0;
    for scale in [1, 2, 4] {
        // Unequal whole groups put eviction at boundaries which an ordinal
        // subtraction cannot recover. Grow the actual catalogue past 1x/2x/4x
        // retention, with at least one full reset each time.
        f.append(&[value(999)], 1);
        groups += 1;
        f.append(&values, if scale == 4 { 1590 } else { 795 });
        groups += if scale == 4 { 1590 } else { 795 };
        f.append(&values[..17], 3);
        groups += 3;
        f.append(&values[..1], 2);
        groups += 2;
        let path = f.db.path().to_owned();
        let replacement =
            StoreDb::create(f.folder.join(format!("replacement-{scale}.sqlite"))).unwrap();
        drop(std::mem::replace(&mut f.db, replacement));
        f.db = StoreDb::connect(path).unwrap();
        let end = f.db.next_metadata_ordinal().unwrap();
        assert!(end > scale * INDEX_VALUES as u64);

        let mut original = ValueIndex::new().unwrap();
        let before = f.db.physical_storage_receipt();
        // The unmodified replay loop, starting at ordinal 1, is the comparator.
        original.sync_to(&f.db, end).unwrap();
        let full = f.db.physical_storage_receipt().since(before);
        assert_eq!(full.metadata_pool_group_fetches, groups);

        let mut recovered = ValueIndex::new().unwrap();
        let before = f.db.physical_storage_receipt();
        recovered.sync(&f.db).unwrap();
        let tail = f.db.physical_storage_receipt().since(before);
        assert_eq!(fingerprint_rows(&recovered), fingerprint_rows(&original));
        assert_eq!(recovered.entries, original.entries);
        assert_eq!(recovered.next, original.next);
        assert!(recovered.entries <= INDEX_VALUES);
        let retained_groups: u32 =
            f.db.reader()
                .unwrap()
                .query_row(
                    "SELECT COUNT(*) FROM metadata_value_groups WHERE first_ordinal>=?1",
                    [(end - recovered.entries as u64) as i64],
                    |row| row.get(0),
                )
                .unwrap();
        assert_eq!(tail.metadata_pool_group_fetches, u64::from(retained_groups));
        assert!(tail.metadata_pool_group_fetches < full.metadata_pool_group_fetches);
        assert!(tail.metadata_pool_decoded_bytes <= INDEX_VALUES as u64 * 128);
        let queries = values
            .iter()
            .copied()
            .chain([value(999), value(9000), values[0]])
            .collect::<Vec<_>>();
        assert_eq!(
            recovered.find_batch(&f.db, &queries).unwrap(),
            original.find_batch(&f.db, &queries).unwrap(),
        );
        println!(
            "tail scale={scale} values={} full_groups={} tail_groups={} retained={} decoded={}",
            end - 1,
            full.metadata_pool_group_fetches,
            tail.metadata_pool_group_fetches,
            recovered.entries,
            tail.metadata_pool_decoded_bytes
        );

        let before = f.db.physical_storage_receipt();
        recovered.sync(&f.db).unwrap();
        assert_eq!(
            f.db.physical_storage_receipt()
                .since(before)
                .metadata_pool_group_fetches,
            0
        );
        f.append(&[value(9001)], 1);
        groups += 1;
        let before = f.db.physical_storage_receipt();
        recovered.sync(&f.db).unwrap();
        assert_eq!(
            f.db.physical_storage_receipt()
                .since(before)
                .metadata_pool_group_fetches,
            1
        );
        original
            .sync_to(&f.db, f.db.next_metadata_ordinal().unwrap())
            .unwrap();
        assert_eq!(fingerprint_rows(&recovered), fingerprint_rows(&original));
    }
}

#[test]
#[ignore = "explicit production metadata-retention corruption proof"]
fn metadata_fingerprint_reopen_tail_preserves_catalogue_checks_and_retained_authentication() {
    let f = Fixture::new();
    let values = (0..165).map(value).collect::<Vec<_>>();
    f.append(&values, 795);
    let end = f.db.next_metadata_ordinal().unwrap();
    let first = ValueIndex::retained_start(&f.db, end).unwrap();
    assert!(first > 1);
    for sql in [
        "DELETE FROM metadata_value_groups WHERE first_ordinal=1",
        "UPDATE metadata_value_groups SET count=count-1 WHERE first_ordinal=1",
        "PRAGMA ignore_check_constraints=ON; UPDATE metadata_value_groups SET count=166 WHERE first_ordinal=1; PRAGMA ignore_check_constraints=OFF",
    ] {
        f.db.reader().unwrap().execute_batch("BEGIN").unwrap();
        f.db.reader().unwrap().execute_batch(sql).unwrap();
        assert!(ValueIndex::new().unwrap().sync(&f.db).is_err(), "{sql}");
        f.db.reader().unwrap().execute_batch("ROLLBACK").unwrap();
    }
    f.db.reader().unwrap().execute_batch("BEGIN").unwrap();
    f.db.reader()
        .unwrap()
        .execute(
            "UPDATE metadata_value_groups SET digest=zeroblob(32) WHERE first_ordinal=?1",
            [first as i64],
        )
        .unwrap();
    assert!(ValueIndex::new().unwrap().sync(&f.db).is_err());
    f.db.reader().unwrap().execute_batch("ROLLBACK").unwrap();
    let mut index = ValueIndex::new().unwrap();
    index.sync(&f.db).unwrap();
    assert_eq!(
        index.find_batch(&f.db, &[values[0]]).unwrap()[&values[0]],
        first as u32
    );

    // An evicted group's body is not consumed by recovery. The explicit full
    // catalogue validator still rejects it, as does any later read of that group.
    f.db.reader().unwrap().execute_batch("BEGIN").unwrap();
    f.db.reader()
        .unwrap()
        .execute(
            "UPDATE metadata_value_groups SET digest=zeroblob(32) WHERE first_ordinal=1",
            [],
        )
        .unwrap();
    ValueIndex::new().unwrap().sync(&f.db).unwrap();
    assert!(f.db.validate_metadata_groups().is_err());
    assert!(f
        .db
        .read_metadata_values(f.db.metadata_group(1).unwrap())
        .is_err());
    f.db.reader().unwrap().execute_batch("ROLLBACK").unwrap();
    f.db.validate_metadata_groups().unwrap();
}
