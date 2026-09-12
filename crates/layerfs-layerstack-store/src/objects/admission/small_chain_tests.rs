use super::*;

fn small(raw: &[u8], prior: Option<ObjectId>) -> AuthenticatedCanonicalObject {
    let mut object = AuthenticatedCanonicalObject::new(
        layerfs_content::file::content::encode_small(raw).unwrap(),
        None,
    )
    .unwrap();
    object.1.prior_ids[0] = prior;
    object
}

#[test]
fn small_chain_depth_closure_integrity_and_exact_reuse() {
    for (length, last_delta) in [(8192, 8), (96 * 1024, 4)] {
        let f = Fixture::new();
        let mut raw = random().repeat(3);
        raw.truncate(length);
        let first = small(&raw, None);
        let mut previous = first.id;
        let mut history = vec![first.clone()];
        f.publish(f.prepare(vec![first]));
        for step in 1..=last_delta + 1 {
            raw[step * 701] ^= step as u8;
            let target = small(&raw, Some(previous));
            let prepared = f.prepare(vec![target.clone()]);
            assert_eq!(prepared.objects[0].delta, step <= last_delta);
            f.publish(prepared);
            assert_eq!(f.db.read_object_row(target.id).unwrap(), target.bytes);
            previous = target.id;
            history.push(target);
        }
        for target in &history {
            let before = f.db.physical_storage_receipt();
            f.publish(f.prepare(vec![target.clone()]));
            assert_eq!(
                f.db.physical_storage_receipt()
                    .since(before)
                    .diag_selected_pack_count,
                0
            );
        }
        // A kind-1 record must still reject a DELTA base, even in schema 9.
        let target = &history[2];
        let location = f.db.object_locations(&[target.id]).unwrap()[&target.id];
        let original: Vec<u8> =
            f.db.reader()
                .unwrap()
                .query_row(
                    "SELECT data FROM object_packs WHERE pack_id=?1",
                    [location.pack],
                    |r| r.get(0),
                )
                .unwrap();
        let mut corrupt = original.clone();
        let directory = 16 + location.group * 16;
        let start =
            u32::from_le_bytes(corrupt[directory..directory + 4].try_into().unwrap()) as usize;
        assert_eq!(corrupt[start], 2);
        corrupt[start] = 1;
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
            &mut ObjectInsertMetrics::default(),
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
        // Missing intermediate dependency cannot return plausible unchecked bytes.
        f.db.reader()
            .unwrap()
            .execute(
                "DELETE FROM objects WHERE object_id=?1",
                [history[1].id.as_bytes().as_slice()],
            )
            .unwrap();
        assert!(f.db.read_object_row(target.id).is_err());
    }
}

#[test]
fn small_chain_schema8_nonpromoting_and_explicit_upgrade() {
    let mut f = Fixture::new();
    let old = f.folder.join("schema8.sqlite");
    let connection = rusqlite::Connection::open(&old).unwrap();
    connection
        .execute_batch(crate::statements::schema::V8)
        .unwrap();
    drop(connection);
    f.db = StoreDb::connect(&old).unwrap();
    assert!(f.db.small_content_format());
    assert!(!f.db.small_chain_format());
    let mut raw = random();
    let first = small(&raw, None);
    f.publish(f.prepare(vec![first.clone()]));
    raw[10] ^= 1;
    let second = small(&raw, Some(first.id));
    f.publish(f.prepare(vec![second.clone()]));
    raw[11] ^= 1;
    let third = small(&raw, Some(second.id));
    f.publish(f.prepare(vec![third.clone()]));
    assert_eq!(f.db.small_physical_base(third.id).unwrap(), Some(first.id));
    assert_eq!(
        f.db.reader()
            .unwrap()
            .pragma_query_value(None, "user_version", |r| r.get::<_, i64>(0))
            .unwrap(),
        8
    );
    let holder = StoreDb::create(f.folder.join("holder.sqlite")).unwrap();
    drop(std::mem::replace(&mut f.db, holder));
    crate::schema::upgrade_format(&old).unwrap();
    f.db = StoreDb::connect(&old).unwrap();
    assert!(f.db.small_chain_format());
    for object in [first, second, third] {
        assert_eq!(f.db.read_object_row(object.id).unwrap(), object.bytes);
    }
}

#[test]
fn compact_framing_multiple_groups_and_authenticated_locator_failures() {
    let f = Fixture::new();
    let objects: Vec<_> = [1, 8192, 131071]
        .into_iter()
        .map(|length| {
            small(
                &random()
                    .into_iter()
                    .cycle()
                    .take(length)
                    .collect::<Vec<_>>(),
                None,
            )
        })
        .collect();
    let prepared = f.prepare(objects.clone());
    assert_eq!(prepared.packs.len(), 1);
    assert_eq!(
        &prepared.packs[0].prepared_bytes()[8..16],
        &[4, 0, 0, 0, 3, 0, 0, 0]
    );
    f.publish(prepared);
    for object in &objects {
        assert_eq!(f.db.read_object_row(object.id).unwrap(), object.bytes);
    }
    f.db.validate_small_packs().unwrap();
    let target = &objects[2];
    let location = f.db.object_locations(&[target.id]).unwrap()[&target.id];
    assert_eq!(location.group, 2);
    let original: Vec<u8> =
        f.db.reader()
            .unwrap()
            .query_row(
                "SELECT data FROM object_packs WHERE pack_id=?1",
                [location.pack],
                |r| r.get(0),
            )
            .unwrap();
    // The indexed length is an input to reconstruction, never an authentication oracle.
    f.db.reader()
        .unwrap()
        .execute(
            "UPDATE objects SET canonical_length=canonical_length-1 WHERE object_id=?1",
            [target.id.as_bytes().as_slice()],
        )
        .unwrap();
    assert!(f.db.read_object_row(target.id).is_err());
    f.db.reader()
        .unwrap()
        .execute(
            "UPDATE objects SET canonical_length=canonical_length+1 WHERE object_id=?1",
            [target.id.as_bytes().as_slice()],
        )
        .unwrap();
    // Corrupt the directory origin, the selected group start, or its frame.
    // Moving an unrelated group boundary does not invalidate this point read.
    for offset in [16, 24, original.len() - 1] {
        let mut corrupt = original.clone();
        corrupt[offset] ^= 1;
        f.db.reader()
            .unwrap()
            .execute(
                "UPDATE object_packs SET data=?1 WHERE pack_id=?2",
                rusqlite::params![corrupt, location.pack],
            )
            .unwrap();
        assert!(f.db.read_object_row(target.id).is_err());
    }
    f.db.reader()
        .unwrap()
        .execute(
            "UPDATE object_packs SET data=?1 WHERE pack_id=?2",
            rusqlite::params![original, location.pack],
        )
        .unwrap();
    assert_eq!(f.db.read_object_row(target.id).unwrap(), target.bytes);
}
