//! Session-local, content-keyed hints. Only selected SmallContent winners enter this cache.
use layerfs_content::ObjectId;

pub(super) const INDEX_BYTES: usize = 128 * 1024;
const SLOTS: usize = 1024;
const WINDOW: usize = 16;
const EMPTY: u64 = u64::MAX;

#[derive(Clone, Copy)]
struct Entry {
    id: ObjectId,
    signature: [u64; 8],
}

pub(super) struct Candidates {
    slots: Box<[Option<Entry>]>,
}

const _: () = {
    assert!(SLOTS.is_power_of_two());
    assert!(SLOTS * std::mem::size_of::<Option<Entry>>()
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
        Self { slots: vec![None; SLOTS].into_boxed_slice() }
    }

    pub(super) fn insert(&mut self, id: ObjectId, signature: [u64; 8]) {
        for hash in signature.iter().copied().filter(|hash| *hash != EMPTY) {
            self.slots[hash as usize & (SLOTS - 1)] = Some(Entry { id, signature });
        }
    }

    pub(super) fn find(&self, target_id: ObjectId, signature: &[u64; 8]) -> Option<ObjectId> {
        let mut best: Option<(usize, ObjectId)> = None;
        for hash in signature.iter().copied().filter(|hash| *hash != EMPTY) {
            let Some(entry) = self.slots[hash as usize & (SLOTS - 1)] else { continue; };
            if entry.id == target_id { continue; }
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
