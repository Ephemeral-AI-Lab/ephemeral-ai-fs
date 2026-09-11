use super::*;
use layerfs_content::tree::{
    compact::{self, InodeNode, InodeSerial},
    inode::{InodeKind, InodeRecordV1},
};

fn dependency(value: u64) -> AuthenticatedCanonicalObject {
    AuthenticatedCanonicalObject::new(
        layerfs_content::encode_bytes_object(&value.to_be_bytes()).unwrap(),
        None,
    )
    .unwrap()
}
fn dependencies(f: &Fixture) {
    f.publish(f.prepare((0..=100).map(dependency).collect()));
}

fn leaf(count: usize, step: u64, prior: Option<ObjectId>) -> AuthenticatedCanonicalObject {
    let rows = (1..=count)
        .map(|index| {
            (
                InodeSerial::new(
                    ((index as u64) << 48)
                        | (u64::from_le_bytes(
                            dependency(index as u64).id.as_bytes()[..8]
                                .try_into()
                                .unwrap(),
                        ) & ((1 << 48) - 1)),
                )
                .unwrap(),
                InodeRecordV1 {
                    kind: InodeKind::RegularFile,
                    namespace_ref_count: 1,
                    content_root: dependency(index as u64).id,
                    metadata_root: dependency(if index == 1 { step } else { index as u64 }).id,
                },
            )
        })
        .collect();
    let mut object = AuthenticatedCanonicalObject::new(
        compact::encode_inode(&InodeNode::Leaf(rows)).unwrap(),
        None,
    )
    .unwrap();
    object.1.prior_ids[0] = prior;
    object
}

#[test]
fn metadata_chain_limits_reopen_and_exact_cas() {
    // The short leaf hits depth first; the full leaf hits canonical work first.
    for (count, last_delta) in [(50, 16), (100, 15)] {
        let mut f = Fixture::new();
        dependencies(&f);
        let mut previous = None;
        let mut history = Vec::new();
        for step in 0..=last_delta + 1 {
            let object = leaf(count, step as u64, previous);
            let prepared = f.prepare(vec![object.clone()]);
            assert_eq!(&prepared.packs[0][8..12], &6u32.to_le_bytes());
            assert_eq!(
                prepared.objects[0].delta,
                step != 0 && step <= last_delta,
                "count={count} step={step}"
            );
            f.publish(prepared);
            assert_eq!(f.db.read_object_row(object.id).unwrap(), object.bytes);
            previous = Some(object.id);
            history.push(object);
        }
        let path = f.db.path().to_owned();
        let replacement = StoreDb::create(f.folder.join("replacement.sqlite")).unwrap();
        drop(std::mem::replace(&mut f.db, replacement));
        f.db = StoreDb::connect(path).unwrap();
        for object in history {
            assert_eq!(f.db.read_object_row(object.id).unwrap(), object.bytes);
            let before = f.db.physical_storage_receipt();
            f.publish(f.prepare(vec![object]));
            assert_eq!(
                f.db.physical_storage_receipt()
                    .since(before)
                    .diag_selected_pack_count,
                0
            );
        }
    }
}

#[test]
fn metadata_authenticates_unused_base_bytes_and_requires_dependencies() {
    let f = Fixture::new();
    dependencies(&f);
    let base = leaf(100, 0, None);
    f.publish(f.prepare(vec![base.clone()]));
    let target = leaf(100, 1, Some(base.id));
    f.publish(f.prepare(vec![target.clone()]));
    assert_eq!(f.db.small_physical_base(target.id).unwrap(), Some(base.id));
    let location = f.db.object_locations(&[base.id]).unwrap()[&base.id];
    let original: Vec<u8> =
        f.db.reader()
            .unwrap()
            .query_row(
                "SELECT data FROM object_packs WHERE pack_id=?1",
                [location.pack],
                |row| row.get(0),
            )
            .unwrap();
    let wrong = leaf(100, 999, None);
    let (group, _) = pack::encode_group(&[&wrong.bytes], &[None], &mut Default::default()).unwrap();
    let mut corrupt = pack::assemble(&[group]).unwrap();
    corrupt[8..12].copy_from_slice(&5u32.to_le_bytes());
    f.db.reader()
        .unwrap()
        .execute(
            "UPDATE object_packs SET data=?1 WHERE pack_id=?2",
            rusqlite::params![corrupt, location.pack],
        )
        .unwrap();
    assert!(f.db.read_object_row(target.id).is_err());
    let known = f.db.object_locations(&[target.id]).unwrap();
    assert!(compare(
        &f.db,
        &known,
        &mut vec![(target.id, &target.bytes)],
        &mut Default::default(),
        0
    )
    .is_err());
    f.db.reader()
        .unwrap()
        .execute(
            "UPDATE object_packs SET data=?1 WHERE pack_id=?2",
            rusqlite::params![original, location.pack],
        )
        .unwrap();
    assert_eq!(f.db.read_object_row(target.id).unwrap(), target.bytes);
    f.db.reader()
        .unwrap()
        .execute(
            "DELETE FROM objects WHERE object_id=?1",
            [base.id.as_bytes().as_slice()],
        )
        .unwrap();
    assert!(f.db.read_object_row(target.id).is_err());
}

#[test]
fn metadata_values_share_across_prepared_packs_and_reopen() {
    let mut f = Fixture::new();
    dependencies(&f);
    let history = (0..40)
        .map(|step| leaf(100, step, None))
        .collect::<Vec<_>>();
    let prepared = f.prepare(history.clone());
    assert!(prepared.packs.len() > 1);
    f.publish(prepared);
    // 99 unchanged values plus 40 distinct values for inode one. Pending values
    // are shared even when one prepared batch is split into multiple packs.
    assert_eq!(f.db.next_metadata_ordinal().unwrap(), 140);
    f.db.validate_metadata_groups().unwrap();
    for object in &history {
        assert_eq!(f.db.read_object_row(object.id).unwrap(), object.bytes);
    }
    let path = f.db.path().to_owned();
    let replacement = StoreDb::create(f.folder.join("replacement.sqlite")).unwrap();
    drop(std::mem::replace(&mut f.db, replacement));
    f.db = StoreDb::connect(path).unwrap();
    let next = leaf(100, 40, Some(history[39].id));
    f.publish(f.prepare(vec![next.clone()]));
    assert_eq!(f.db.next_metadata_ordinal().unwrap(), 141);
    assert_eq!(f.db.read_object_row(next.id).unwrap(), next.bytes);
    // Pool canonical values have no per-value CAS rows or permanent value index.
    let count: i64 =
        f.db.reader()
            .unwrap()
            .query_row("SELECT count(*) FROM objects", [], |row| row.get(0))
            .unwrap();
    assert_eq!(count, 101 + 41);
}

#[test]
fn metadata_batched_value_lookup_pages_and_sharing() {
    fn unique_rows(tag: u64, count: usize, key_offset: u64) -> Vec<(InodeSerial, InodeRecordV1)> {
        let dep = |value: u64| dependency(tag * 1_000 + value);
        (1..=count)
            .map(|index| {
                (
                    InodeSerial::new(
                        (((index as u64) + key_offset) << 48)
                            | (u64::from_le_bytes(
                                dep(index as u64).id.as_bytes()[..8].try_into().unwrap(),
                            ) & ((1 << 48) - 1)),
                    )
                    .unwrap(),
                    InodeRecordV1 {
                        kind: InodeKind::RegularFile,
                        namespace_ref_count: 1,
                        content_root: dep(index as u64).id,
                        metadata_root: dep(index as u64).id,
                    },
                )
            })
            .collect()
    }
    fn leaf_of(rows: Vec<(InodeSerial, InodeRecordV1)>) -> AuthenticatedCanonicalObject {
        AuthenticatedCanonicalObject::new(
            compact::encode_inode(&InodeNode::Leaf(rows)).unwrap(),
            None,
        )
        .unwrap()
    }
    let mut f = Fixture::new();
    let deps = (0..15_u64)
        .flat_map(|tag| (1..=100_u64).map(move |index| dependency(tag * 1_000 + index)))
        .collect::<Vec<_>>();
    f.publish(f.prepare(deps));
    let history = (0..15_u64)
        .map(|tag| leaf_of(unique_rows(tag, 100, 0)))
        .collect::<Vec<_>>();
    f.publish(f.prepare(history.clone()));
    // 1500 distinct absent values crossed the batched-lookup page boundary.
    assert_eq!(f.db.next_metadata_ordinal().unwrap(), 1501);
    f.db.validate_metadata_groups().unwrap();
    for object in &history {
        assert_eq!(f.db.read_object_row(object.id).unwrap(), object.bytes);
    }
    // A different leaf reusing every indexed value resolves all ordinals
    // through one batched lookup and assigns no new ordinals.
    let reuse = leaf_of(unique_rows(0, 100, 200));
    f.publish(f.prepare(vec![reuse.clone()]));
    assert_eq!(f.db.next_metadata_ordinal().unwrap(), 1501);
    assert_eq!(f.db.read_object_row(reuse.id).unwrap(), reuse.bytes);
}

#[test]
fn metadata_pool_catalogue_corruption_and_publication_rollback() {
    let f = Fixture::new();
    dependencies(&f);
    let base = leaf(100, 0, None);
    f.publish(f.prepare(vec![base.clone()]));
    let baseline = f.db.next_metadata_ordinal().unwrap();
    let target = leaf(100, 1, Some(base.id));
    let mut prepared = f.prepare(vec![target.clone()]);
    let session = prepared.session.clone();
    prepared.final_batch = false;
    f.publish(prepared);
    assert_eq!(f.db.next_metadata_ordinal().unwrap(), baseline + 1);
    // Warm the derived index with the new visible group, then roll back both its
    // canonical locator and physical pool. Reused ordinals cannot hit stale cache.
    f.db.metadata_index()
        .unwrap()
        .as_mut()
        .unwrap()
        .sync(&f.db)
        .unwrap();
    session.rollback().unwrap();
    assert!(f.db.metadata_index().unwrap().is_none());
    assert_eq!(f.db.next_metadata_ordinal().unwrap(), baseline);
    assert_eq!(f.db.read_object_row(base.id).unwrap(), base.bytes);
    assert!(f.db.read_object_row(target.id).is_err());
    drop(session);
    let prepared = f.prepare(vec![target.clone()]);
    let failure = prepared.publish(&f.db, &mut 0, |_, _, _| -> Result<()> {
        Err(StoreError::Integrity("injected pool publication failure"))
    });
    assert!(failure.is_err());
    assert_eq!(f.db.next_metadata_ordinal().unwrap(), baseline);
    f.publish(f.prepare(vec![target.clone()]));
    assert_eq!(f.db.read_object_row(target.id).unwrap(), target.bytes);

    let group = f.db.metadata_group(1).unwrap();
    f.db.reader()
        .unwrap()
        .execute(
            "UPDATE metadata_value_groups SET digest=zeroblob(32) WHERE first_ordinal=1",
            [],
        )
        .unwrap();
    assert!(f.db.read_object_row(target.id).is_err());
    assert!(f.db.validate_metadata_groups().is_err());
    f.db.reader()
        .unwrap()
        .execute(
            "UPDATE metadata_value_groups SET digest=?1 WHERE first_ordinal=1",
            [group.digest.as_bytes().as_slice()],
        )
        .unwrap();
    assert_eq!(f.db.read_object_row(target.id).unwrap(), target.bytes);
    // Endpoint acceleration relies on schema count bounds; full authentication
    // and catalogue validation must still reject corruption bypassing CHECKs.
    {
        let connection = f.db.reader().unwrap();
        connection.execute_batch(
            "PRAGMA ignore_check_constraints=ON;
             UPDATE metadata_value_groups SET count=166 WHERE first_ordinal=1;
             PRAGMA ignore_check_constraints=OFF;",
        ).unwrap();
    }
    assert!(f.db.validate_metadata_groups().is_err());
    assert!(f.db.read_object_row(target.id).is_err());
    f.db.reader().unwrap().execute(
        "UPDATE metadata_value_groups SET count=?1 WHERE first_ordinal=1",
        [group.count as i64],
    ).unwrap();
    f.db.reader()
        .unwrap()
        .execute(
            "UPDATE metadata_value_groups SET count=count+1 WHERE first_ordinal=1",
            [],
        )
        .unwrap();
    assert!(f.db.read_object_row(target.id).is_err());
    f.db.reader()
        .unwrap()
        .execute(
            "UPDATE metadata_value_groups SET count=count-1 WHERE first_ordinal=1",
            [],
        )
        .unwrap();
    f.db.reader()
        .unwrap()
        .execute(
            "DELETE FROM metadata_value_groups WHERE first_ordinal=1",
            [],
        )
        .unwrap();
    assert!(f.db.read_object_row(target.id).is_err());
    assert!(f.db.validate_metadata_groups().is_err());
}

#[test]
fn metadata_pool_authenticates_unused_base_values() {
    use super::super::super::metadata;
    let f = Fixture::new();
    dependencies(&f);
    let base = leaf(100, 0, None);
    f.publish(f.prepare(vec![base.clone()]));
    let target = leaf(100, 1, Some(base.id));
    let prepared = f.prepare(vec![target.clone()]);
    assert!(prepared.objects[0].delta);
    f.publish(prepared);
    let group = f.db.metadata_group(1).unwrap();
    let original: Vec<u8> =
        f.db.reader()
            .unwrap()
            .query_row(
                "SELECT data FROM object_packs WHERE pack_id=?1",
                [group.pack],
                |row| row.get(0),
            )
            .unwrap();
    let header =
        pack::versioned_header(original[..16].try_into().unwrap(), original.len()).unwrap();
    let mut encoded = Vec::new();
    let mut changed_digest = None;
    for number in 0..header.group_count {
        let entry = pack::versioned_entry(
            original[16 + 16 * number..32 + 16 * number]
                .try_into()
                .unwrap(),
            header,
            original.len(),
        )
        .unwrap();
        let raw = original[entry.range.clone()].to_vec();
        if number == group.number {
            let mut values = f.db.read_metadata_values(group).unwrap();
            // Inode one's base value is entirely replaced in the target. Change
            // that value and the group digest together: target-only hashing would
            // still pass, but complete intermediate-base authentication must fail.
            values[0][41..73].copy_from_slice(dependency(999).id.as_bytes());
            let canonical = values
                .iter()
                .map(metadata::value_canonical)
                .collect::<Result<Vec<_>>>()
                .unwrap();
            let (replacement, _) = pack::encode_group(
                &canonical.iter().map(Vec::as_slice).collect::<Vec<_>>(),
                &vec![None; values.len()],
                &mut Default::default(),
            )
            .unwrap();
            let body = pack::decode_group(
                pack::GroupEntry {
                    range: 0..replacement.bytes.len(),
                    decoded_length: replacement.decoded_length,
                    codec: replacement.codec,
                    oversized: false,
                },
                replacement.bytes.clone(),
            )
            .unwrap();
            changed_digest = Some(ObjectId::for_bytes(&body));
            encoded.push(replacement);
        } else {
            let body = pack::decode_group(entry.clone(), raw.clone()).unwrap();
            encoded.push(pack::EncodedGroup {
                bytes: raw,
                decoded_length: entry.decoded_length,
                codec: entry.codec,
                records: u32::from_le_bytes(body[..4].try_into().unwrap()) as usize,
            });
        }
    }
    let mut corrupt = pack::assemble(&encoded).unwrap();
    corrupt[8..12].copy_from_slice(&6u32.to_le_bytes());
    f.db.reader()
        .unwrap()
        .execute(
            "UPDATE object_packs SET data=?1 WHERE pack_id=?2",
            rusqlite::params![corrupt, group.pack],
        )
        .unwrap();
    f.db.reader()
        .unwrap()
        .execute(
            "UPDATE metadata_value_groups SET digest=?1 WHERE first_ordinal=1",
            [changed_digest.unwrap().as_bytes().as_slice()],
        )
        .unwrap();
    assert!(matches!(
        f.db.read_object_row(target.id),
        Err(StoreError::Integrity("object identity"))
    ));
    f.db.reader()
        .unwrap()
        .execute(
            "UPDATE object_packs SET data=?1 WHERE pack_id=?2",
            rusqlite::params![original, group.pack],
        )
        .unwrap();
    f.db.reader()
        .unwrap()
        .execute(
            "UPDATE metadata_value_groups SET digest=?1 WHERE first_ordinal=1",
            [group.digest.as_bytes().as_slice()],
        )
        .unwrap();
    assert_eq!(f.db.read_object_row(target.id).unwrap(), target.bytes);
}

#[test]
fn metadata_retains_unpooled_reads_and_bounds_optional_pool_work() {
    let f = Fixture::new();
    dependencies(&f);
    let base = leaf(100, 0, None);
    let mut prepared = f.prepare(vec![base.clone()]);
    let (group, _) = pack::encode_group(&[&base.bytes], &[None], &mut Default::default()).unwrap();
    let mut old = pack::assemble(&[group]).unwrap();
    old[8..12].copy_from_slice(&5u32.to_le_bytes());
    prepared.packs = vec![old];
    prepared.pool_groups.clear();
    prepared.pending_values.clear();
    f.publish(prepared);
    assert_eq!(f.db.next_metadata_ordinal().unwrap(), 1);
    assert_eq!(f.db.read_object_row(base.id).unwrap(), base.bytes);
    let target = leaf(100, 1, Some(base.id));
    let prepared = f.prepare(vec![target.clone()]);
    assert!(
        !prepared.objects[0].delta,
        "pooled programs require pooled bases"
    );
    f.publish(prepared);
    let mut budget = read::HintReadBudget::default();
    budget.begin_target();
    assert!(budget.charge_metadata_pool(8 * 1024 * 1024));
    assert!(f
        .db
        .metadata_predecessor(target.id, &mut budget)
        .unwrap()
        .is_none());
    assert!(budget.exhausted);
    assert_eq!(f.db.read_object_row(target.id).unwrap(), target.bytes);
    assert_eq!(f.db.read_object_row(base.id).unwrap(), base.bytes);
}
