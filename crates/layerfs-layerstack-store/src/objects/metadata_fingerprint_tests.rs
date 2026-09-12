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

#[test]
fn pooled_metadata_delta_reconstructs_after_byte_and_group_cache_eviction() {
    use layerfs_content::tree::compact::{InodeNode, InodeSerial};

    fn leaf(ordinals: &[u32], values_per_group: usize) -> (Vec<u8>, Vec<u8>) {
        let rows = ordinals
            .iter()
            .enumerate()
            .map(|(index, ordinal)| {
                let group = (*ordinal as usize - 1) / values_per_group;
                let offset = (*ordinal as usize - 1) % values_per_group;
                (
                    InodeSerial::new(index as u64 + 1).unwrap(),
                    compact::decode_inode_value(&value((group * values_per_group + offset) as u64))
                        .unwrap(),
                )
            })
            .collect();
        let canonical = compact::encode_inode(&InodeNode::Leaf(rows)).unwrap();
        let mut physical = canonical[..44].to_vec();
        for (row, ordinal) in canonical[44..].chunks_exact(81).zip(ordinals) {
            physical.extend_from_slice(&row[..8]);
            physical.extend_from_slice(&ordinal.to_be_bytes());
        }
        (canonical, physical)
    }

    fn publish(
        db: &StoreDb,
        canonical: &[u8],
        physical: &[u8],
        delta: Option<Vec<u8>>,
    ) -> ObjectId {
        let group = if let Some(delta) = delta {
            // A supported one-record RAW DELTA group. This isolates reader
            // reconstruction; it does not claim an encoder selection result.
            let mut body = Vec::new();
            body.extend_from_slice(&1u32.to_le_bytes());
            body.extend_from_slice(&(delta.len() as u32).to_le_bytes());
            body.extend_from_slice(&delta);
            pack::EncodedGroup {
                decoded_length: body.len(),
                bytes: body,
                codec: pack::Codec::Raw,
                records: 1,
            }
        } else {
            pack::encode_group(&[physical], &[None], &mut Default::default())
                .unwrap()
                .0
        };
        let mut bytes = pack::assemble(&[group]).unwrap();
        bytes[8..12].copy_from_slice(&6u32.to_le_bytes());
        let id = ObjectId::for_bytes(canonical);
        let mut connection = db.reader().unwrap();
        let transaction = connection.transaction().unwrap();
        let pack: i64 = transaction
            .query_row(
                "SELECT COALESCE(MAX(pack_id),0)+1 FROM object_packs",
                [],
                |row| row.get(0),
            )
            .unwrap();
        transaction
            .execute(
                "INSERT INTO object_packs(pack_id,data) VALUES(?1,?2)",
                rusqlite::params![pack, bytes],
            )
            .unwrap();
        transaction.execute(
            "INSERT INTO objects(object_id,canonical_length,pack_id,group_number,record_number) VALUES(?1,?2,?3,0,0)",
            rusqlite::params![id.as_bytes().as_slice(), canonical.len() as i64, pack],
        ).unwrap();
        transaction.commit().unwrap();
        id
    }

    for (group_count, values_per_group, row_count) in [(45, 165, 45), (129, 1, 100)] {
        let f = Fixture::new();
        for group in 0..group_count {
            let values = (0..values_per_group)
                .map(|offset| value((group * values_per_group + offset) as u64))
                .collect::<Vec<_>>();
            f.append(&values, 1);
        }
        let original = (0..row_count)
            .map(|row| (row * values_per_group + 1) as u32)
            .collect::<Vec<_>>();
        let mut changed = original.clone();
        if values_per_group == 165 {
            changed[0] += 1;
        } else {
            for (index, ordinal) in changed.iter_mut().take(29).enumerate() {
                *ordinal = 101 + index as u32;
            }
        }
        let (base, base_physical) = leaf(&original, values_per_group);
        let (target, target_physical) = leaf(&changed, values_per_group);

        let mut pool = PoolRead::default();
        pool.begin_chain();
        assert_eq!(
            pool.expand(&f.db, &base_physical, base.len(), None)
                .unwrap()
                .unwrap(),
            base
        );
        assert!(pool.retained <= 512 * 1024 && pool.groups.len() <= 128);
        if values_per_group == 165 {
            assert!(pool.groups.len() < row_count, "actual 512-KiB eviction");
        }
        assert_eq!(
            pool.expand(&f.db, &target_physical, target.len(), None)
                .unwrap()
                .unwrap(),
            target
        );
        assert!(pool.retained <= 512 * 1024 && pool.groups.len() <= 128);
        assert!(pool.groups.len() < row_count, "actual byte/count eviction");
        let owned_values = pool
            .groups
            .values()
            .map(|values| values.capacity() * std::mem::size_of::<[u8; 73]>())
            .sum::<usize>();
        assert_eq!(pool.retained, owned_values + 256 * pool.groups.len());
        println!(
            "pooled retained groups={} actual_value_capacity_bytes={owned_values} charged_bytes={}",
            pool.groups.len(),
            pool.retained
        );
        drop(pool);

        let base_id = publish(&f.db, &base, &base_physical, None);
        let delta = pack::delta_record(
            base_id,
            &base_physical,
            &target_physical,
            &mut (8 * 1024 * 1024),
            &mut Default::default(),
        )
        .unwrap()
        .unwrap();
        let target_id = publish(&f.db, &target, &target_physical, Some(delta));
        let before = f.db.physical_storage_receipt();
        assert_eq!(f.db.read_object_row(target_id).unwrap(), target);
        let counters = f.db.physical_storage_receipt().since(before);
        assert_eq!(counters.base_fetches, 1);
        assert_eq!(counters.metadata_pool_group_fetches, (2 * row_count) as u64);
        assert!(
            counters.metadata_pool_group_fetches > group_count as u64,
            "evicted values were fetched again during the actual DELTA chain"
        );
        println!(
            "pooled eviction groups={group_count} values={} rows={row_count} chain_pool_fetches={}",
            group_count * values_per_group,
            counters.metadata_pool_group_fetches
        );
    }
}
