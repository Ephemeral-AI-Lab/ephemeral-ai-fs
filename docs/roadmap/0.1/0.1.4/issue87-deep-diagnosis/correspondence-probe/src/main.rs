//! Synthetic algorithm proof. No Store, benchmark, encoding trial, or replay.
use layerfs_content::file::rope::{FileStateRoot, ObjectRead, PredecessorCursor};
use layerfs_content::file::{extent::*, extent_codec::*};
use layerfs_content::{CoreError, CoreResult, ObjectId};
use std::{
    cell::Cell,
    collections::BTreeMap,
    sync::atomic::{AtomicU64, Ordering},
};

#[derive(Default)]
struct Memory {
    objects: BTreeMap<ObjectId, Vec<u8>>,
    read_bytes: Cell<u64>,
}
impl Memory {
    fn insert(&mut self, bytes: Vec<u8>) -> ObjectId {
        let id = ObjectId::for_bytes(&bytes);
        self.objects.insert(id, bytes);
        id
    }
    fn file(&mut self, leaf_count: usize, per_leaf: usize) -> FileStateRoot {
        let mut children = Vec::new();
        for leaf in 0..leaf_count {
            let extents = (0..per_leaf)
                .map(|i| {
                    let bytes = ((leaf * per_leaf + i) as u32).to_be_bytes();
                    let id = self.insert(encode_chunk_object(&bytes).unwrap());
                    ExtentSliceV3::new(id, 0, 4).unwrap()
                })
                .collect();
            let id = self.insert(
                encode_node(&ExtentNodeV3::Leaf {
                    subtree_logical_bytes: (per_leaf * 4) as u64,
                    extents,
                })
                .unwrap(),
            );
            children.push(ChildDescriptorV3 {
                cumulative_logical_end: ((leaf + 1) * per_leaf * 4) as u64,
                cumulative_extent_end: ((leaf + 1) * per_leaf) as u64,
                child_object_id: id,
            });
        }
        let logical_len = (leaf_count * per_leaf * 4) as u64;
        let extent_count = (leaf_count * per_leaf) as u64;
        let mapping_root = if leaf_count == 1 {
            children[0].child_object_id
        } else {
            self.insert(
                encode_node(&ExtentNodeV3::Branch {
                    level: 1,
                    subtree_logical_bytes: logical_len,
                    subtree_extent_count: extent_count,
                    children,
                })
                .unwrap(),
            )
        };
        FileStateRoot(
            self.insert(
                encode_file_state(FileStateV3 {
                    logical_len,
                    extent_count,
                    tree_level: u8::from(leaf_count != 1),
                    profile_id: profile_id(),
                    mapping_root,
                })
                .unwrap(),
            ),
        )
    }
}
impl ObjectRead for Memory {
    fn get(&self, id: ObjectId) -> CoreResult<Vec<u8>> {
        let bytes = self.objects.get(&id).ok_or(CoreError::IdentityMismatch)?;
        self.read_bytes
            .set(self.read_bytes.get() + bytes.len() as u64);
        Ok(bytes.clone())
    }
}

// Exact reservation policy from objects.rs, guarded against source drift by the runner.
fn reserve(file_reserved: &mut u64, operation_reserved: &AtomicU64) -> bool {
    const FETCH_RESERVATION: u64 = 131136;
    if *file_reserved + FETCH_RESERVATION > 1024 * 1024 {
        return false;
    }
    if operation_reserved
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |used| {
            used.checked_add(FETCH_RESERVATION)
                .filter(|sum| *sum <= 16 * 1024 * 1024)
        })
        .is_err()
    {
        return false;
    }
    *file_reserved += FETCH_RESERVATION;
    true
}

fn main() {
    let mut memory = Memory::default();
    let root = memory.file(1, 1);
    let operation = AtomicU64::new(0);
    let mut hinted = 0;
    let mut exhausted = 0;
    for _ in 0..128 {
        let mut cursor = PredecessorCursor::new(root);
        let mut file_reserved = 0;
        let hints = cursor
            .hints(&memory, 0, 4, || reserve(&mut file_reserved, &operation))
            .unwrap();
        hinted += u64::from(hints[0].is_some());
        exhausted += cursor.counters().1;
    }
    assert_eq!(
        (hinted, exhausted, operation.load(Ordering::Relaxed)),
        (63, 65, 16_654_272)
    );
    println!(
        "shared_operation: hinted_files={hinted}, exhausted_cursors={exhausted}, reserved_bytes={}, canonical_metadata_bytes_read={}",
        operation.load(Ordering::Relaxed),
        memory.read_bytes.get()
    );

    // Cursor spans are monotonic; no-overlap is not exhaustion.
    let mut cursor = PredecessorCursor::new(root);
    assert_eq!(cursor.hints(&memory, 4, 4, || true).unwrap(), [None; 4]);
    assert_eq!(cursor.counters().1, 0);
    assert!(cursor.hints(&memory, 0, 4, || true).is_err());

    // Production per-file reservation binds before the 4096-descriptor ceiling.
    let big = memory.file(33, 128);
    let operation = AtomicU64::new(0);
    let mut file_reserved = 0;
    let mut cursor = PredecessorCursor::new(big);
    let mut hinted = 0;
    for i in 0..4224 {
        hinted += u64::from(
            cursor
                .hints(&memory, i * 4, 4, || {
                    reserve(&mut file_reserved, &operation)
                })
                .unwrap()[0]
                .is_some(),
        );
    }
    assert_eq!(
        (hinted, file_reserved, cursor.counters()),
        (640, 917_952, (640, 1))
    );
    println!(
        "large_file: hinted_extents={hinted}, reserved_bytes={file_reserved}, descriptors={}, exhausted={}",
        cursor.counters().0,
        cursor.counters().1
    );

    let mut cursor = PredecessorCursor::new(big);
    let mut hinted = 0;
    for i in 0..4224 {
        hinted += u64::from(cursor.hints(&memory, i * 4, 4, || true).unwrap()[0].is_some());
    }
    assert_eq!((hinted, cursor.counters()), (4096, (4096, 1)));
    println!(
        "unlimited_reservation: hinted_extents={hinted}, descriptors={}, exhausted={}",
        cursor.counters().0,
        cursor.counters().1
    );

    // First four unique overlapping IDs are selected, not a similarity ranking.
    let short = memory.file(1, 8);
    let mut cursor = PredecessorCursor::new(short);
    let hints = cursor.hints(&memory, 0, 32, || true).unwrap();
    for (i, hint) in hints.into_iter().enumerate() {
        assert_eq!(
            hint,
            Some(ObjectId::for_bytes(
                &encode_chunk_object(&(i as u32).to_be_bytes()).unwrap()
            ))
        );
    }
    assert_eq!(cursor.counters(), (8, 0));
    println!("all_checks_passed=true");
}
