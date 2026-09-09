//! Session-local, content-keyed hints. Only selected SmallContent winners enter this cache.
use layerfs_content::ObjectId;

pub(super) const INDEX_BYTES: usize = 128 * 1024;
const SLOTS: usize = 1024;
const REFERENCES: usize = 8192;
const NO_ENTRY: u16 = u16::MAX;
const WINDOW: usize = 16;
const EMPTY: u64 = u64::MAX;

#[derive(Clone, Copy)]
struct Entry {
    id: ObjectId,
    signature: [u64; 8],
}

pub(crate) struct Candidates {
    slots: Box<[Option<Entry>]>,
    references: Box<[u16]>,
    next: usize,
}

const _: () = {
    assert!(SLOTS.is_power_of_two());
    assert!(REFERENCES.is_power_of_two() && SLOTS < NO_ENTRY as usize);
    assert!(SLOTS * std::mem::size_of::<Option<Entry>>()
        + REFERENCES * std::mem::size_of::<u16>()
        + std::mem::size_of::<Option<std::sync::Mutex<Candidates>>>() <= INDEX_BYTES);
};

fn mix(mut value: u64) -> u64 {
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

/// Eight smallest distinct mixed hashes, computed without input-sized allocation.
/// MAX denotes an unused slot; that one possible hash is deliberately omitted.
pub(super) fn signature(raw: &[u8]) -> [u64; 8] {
    let mut result = [EMPTY; 8];
    if raw.len() < WINDOW {
        return result;
    }
    let high = 257u64.wrapping_pow((WINDOW - 1) as u32);
    let mut rolling = raw[..WINDOW]
        .iter()
        .fold(0u64, |hash, byte| hash.wrapping_mul(257).wrapping_add(u64::from(*byte)));
    for start in 0..=raw.len() - WINDOW {
        if start != 0 {
            rolling = rolling
                .wrapping_sub(u64::from(raw[start - 1]).wrapping_mul(high))
                .wrapping_mul(257)
                .wrapping_add(u64::from(raw[start + WINDOW - 1]));
        }
        let hash = mix(rolling);
        if hash < result[7] && !result.contains(&hash) {
            let index = result.partition_point(|value| *value < hash);
            result.copy_within(index..7, index + 1);
            result[index] = hash;
        }
    }
    result
}

impl Candidates {
    pub(super) fn new() -> Self {
        // Boxed slices retain exactly the fixed slot count, with no spare capacity.
        Self {
            slots: vec![None; SLOTS].into_boxed_slice(),
            references: vec![NO_ENTRY; REFERENCES].into_boxed_slice(),
            next: 0,
        }
    }

    pub(super) fn insert(&mut self, id: ObjectId, signature: [u64; 8]) {
        if signature[0] == EMPTY { return; }
        let slot = self.next;
        if let Some(old) = self.slots[slot] {
            for hash in old.signature.iter().copied().filter(|hash| *hash != EMPTY) {
                let reference = &mut self.references[hash as usize & (REFERENCES - 1)];
                if *reference == slot as u16 { *reference = NO_ENTRY; }
            }
        }
        self.slots[slot] = Some(Entry { id, signature });
        for hash in signature.iter().copied().filter(|hash| *hash != EMPTY) {
            self.references[hash as usize & (REFERENCES - 1)] = slot as u16;
        }
        self.next = (slot + 1) & (SLOTS - 1);
    }

    pub(super) fn find(&self, target_id: ObjectId, signature: &[u64; 8]) -> Option<ObjectId> {
        let mut best: Option<(usize, ObjectId)> = None;
        for hash in signature.iter().copied().filter(|hash| *hash != EMPTY) {
            let reference = self.references[hash as usize & (REFERENCES - 1)];
            if reference == NO_ENTRY { continue; }
            let Some(entry) = self.slots[reference as usize] else { continue; };
            if entry.id == target_id || !entry.signature.contains(&hash) { continue; }
            let overlap = signature.iter()
                .filter(|hash| **hash != EMPTY && entry.signature.contains(hash))
                .count();
            if overlap >= 2 && best.is_none_or(|(count, id)| overlap > count || (overlap == count && entry.id < id)) {
                best = Some((overlap, entry.id));
            }
        }
        best.map(|(_, id)| id)
    }
}
