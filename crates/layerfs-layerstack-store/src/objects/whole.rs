//! Product content compaction wire format. Canonical identities stay unchanged;
//! whole-file owners are authenticated physical dependencies of native slices.
use super::{
    pack,
    read::{HintReadBudget, Location},
    CanonicalObject,
};
use crate::{schema::StoreDb, PhysicalStorageReceipt, Result, StoreError};
use layerfs_content::{
    file::{content, extent_codec},
    ObjectId,
};
use std::{collections::BTreeMap, ops::Range};

pub(super) const MAGIC: &[u8; 8] = b"LFCNT1\0\0";
pub(super) const VERSION: u32 = 107;
pub(super) const PACK_LIMIT: usize = 4 * 1024 * 1024;
pub(super) const EDGES: usize = 50;
pub(super) const CLOSURE: usize = 64 * 1024 * 1024;

fn invalid() -> StoreError {
    StoreError::Integrity("whole-file physical record")
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Role {
    Small,
    Whole,
    Native,
}
impl Role {
    pub(super) fn canonical(self, raw: &[u8]) -> Result<Vec<u8>> {
        Ok(match self {
            Self::Small => content::encode_small(raw)?,
            Self::Whole => content::encode_whole(raw)?,
            Self::Native => extent_codec::encode_chunk_object(raw)?,
        })
    }
    pub(super) fn raw<'a>(self, canonical: &'a [u8]) -> Result<&'a [u8]> {
        match self {
            Self::Small => content::small_bytes(canonical)?.ok_or_else(invalid),
            Self::Whole => content::whole_bytes(canonical)?.ok_or_else(invalid),
            Self::Native => Ok(extent_codec::decode_chunk_payload(
                layerfs_content::decode_bytes_object(canonical)?,
            )?),
        }
    }
}

pub(super) struct Record<'a> {
    pub role: Role,
    pub raw_length: usize,
    pub base: Option<ObjectId>,
    pub slice: Option<Range<usize>>,
    pub frame: &'a [u8],
}

pub(super) fn record(bytes: &[u8], canonical_length: usize) -> Result<Record<'_>> {
    let kind = *bytes.first().ok_or_else(invalid)?;
    let role = match kind {
        0 | 1 => Role::Small,
        2 | 3 => Role::Whole,
        4 | 5 => Role::Native,
        _ => return Err(invalid()),
    };
    let raw_length = canonical_length
        .checked_sub(if role == Role::Small { 23 } else { 21 })
        .ok_or_else(invalid)?;
    if !match role {
        Role::Small => (1..content::SMALL_LIMIT).contains(&raw_length),
        Role::Whole => (content::SMALL_LIMIT..=content::WHOLE_LIMIT).contains(&raw_length),
        Role::Native => (1..=pack::NATIVE_RAW_LIMIT).contains(&raw_length),
    } {
        return Err(invalid());
    }
    let base = if matches!(kind, 1 | 3 | 4) {
        Some(ObjectId::from_bytes(bytes.get(1..33).ok_or_else(invalid)?)?)
    } else {
        None
    };
    if kind == 4 {
        if bytes.len() != 41 {
            return Err(invalid());
        }
        let offset = u32::from_le_bytes(bytes[33..37].try_into().unwrap()) as usize;
        let length = u32::from_le_bytes(bytes[37..41].try_into().unwrap()) as usize;
        let end = offset.checked_add(length).ok_or_else(invalid)?;
        if length != raw_length || end > content::WHOLE_LIMIT {
            return Err(invalid());
        }
        return Ok(Record {
            role,
            raw_length,
            base,
            slice: Some(offset..end),
            frame: &[],
        });
    }
    let frame = bytes
        .get(if base.is_some() { 33 } else { 1 }..)
        .ok_or_else(invalid)?;
    let limit = if role == Role::Whole {
        content::WHOLE_LIMIT + 1024
    } else {
        super::delta::FRAME_LIMIT
    };
    if frame.is_empty() || frame.len() > limit {
        return Err(invalid());
    }
    Ok(Record {
        role,
        raw_length,
        base,
        slice: None,
        frame,
    })
}

pub(super) fn encode(
    role: Role,
    raw_length: usize,
    base: Option<ObjectId>,
    frame: &[u8],
) -> Result<Vec<u8>> {
    let kind = match (role, base.is_some()) {
        (Role::Small, false) => 0,
        (Role::Small, true) => 1,
        (Role::Whole, false) => 2,
        (Role::Whole, true) => 3,
        (Role::Native, false) => 5,
        _ => return Err(invalid()),
    };
    let mut bytes = Vec::with_capacity(1 + if base.is_some() { 32 } else { 0 } + frame.len());
    bytes.push(kind);
    if let Some(base) = base {
        bytes.extend_from_slice(base.as_bytes());
    }
    bytes.extend_from_slice(frame);
    record(
        &bytes,
        raw_length + if role == Role::Small { 23 } else { 21 },
    )?;
    Ok(bytes)
}

pub(super) fn slice(owner: ObjectId, offset: usize, length: usize) -> Result<Vec<u8>> {
    let mut bytes = Vec::with_capacity(41);
    bytes.push(4);
    bytes.extend_from_slice(owner.as_bytes());
    bytes.extend_from_slice(&u32::try_from(offset).map_err(|_| invalid())?.to_le_bytes());
    bytes.extend_from_slice(&u32::try_from(length).map_err(|_| invalid())?.to_le_bytes());
    record(&bytes, length + 21)?;
    Ok(bytes)
}

pub(super) fn header(bytes: &[u8; 16], length: usize) -> Result<usize> {
    let count = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
    if &bytes[..8] != MAGIC
        || bytes[8..12] != VERSION.to_le_bytes()
        || !(1..=256).contains(&count)
        || length > PACK_LIMIT
        || length <= 16 + 4 * count
    {
        return Err(invalid());
    }
    Ok(count)
}

pub(super) fn range(starts: &[u8], length: usize, number: usize) -> Result<Range<usize>> {
    let count = starts.len() / 4;
    if starts.len() % 4 != 0
        || !(1..=256).contains(&count)
        || number >= count
        || length > PACK_LIMIT
    {
        return Err(invalid());
    }
    let mut offset = 16 + starts.len();
    let mut selected = 0..0;
    for index in 0..count {
        let start =
            u32::from_le_bytes(starts[4 * index..4 * index + 4].try_into().unwrap()) as usize;
        let end = if index + 1 == count {
            length
        } else {
            u32::from_le_bytes(starts[4 * index + 4..4 * index + 8].try_into().unwrap()) as usize
        };
        if start != offset
            || end <= start
            || end > length
            || end - start > content::WHOLE_LIMIT + 1057
        {
            return Err(invalid());
        }
        if index == number {
            selected = start..end;
        }
        offset = end;
    }
    Ok(selected)
}

pub(super) fn assemble(records: &[Vec<u8>]) -> Result<Vec<u8>> {
    let length = 16 + 4 * records.len() + records.iter().map(Vec::len).sum::<usize>();
    if records.is_empty() || records.len() > 256 || length > PACK_LIMIT {
        return Err(invalid());
    }
    let mut bytes = Vec::with_capacity(length);
    bytes.extend_from_slice(MAGIC);
    bytes.extend_from_slice(&VERSION.to_le_bytes());
    bytes.extend_from_slice(&(records.len() as u32).to_le_bytes());
    let mut offset = 16 + 4 * records.len();
    for record in records {
        bytes.extend_from_slice(&(offset as u32).to_le_bytes());
        offset += record.len();
    }
    for record in records {
        bytes.extend_from_slice(record);
    }
    range(&bytes[16..16 + 4 * records.len()], bytes.len(), 0)?;
    Ok(bytes)
}

#[derive(Default)]
pub(super) struct OwnerCache {
    values: BTreeMap<ObjectId, Vec<u8>>,
    bytes: usize,
}
impl OwnerCache {
    fn insert(&mut self, id: ObjectId, bytes: Vec<u8>) {
        let size = bytes.capacity() + 256;
        if self.bytes + size > 4 * 1024 * 1024 {
            self.values.clear();
            self.bytes = 0;
        }
        self.bytes += size;
        self.values.insert(id, bytes);
    }
}

impl StoreDb {
    pub(super) fn whole_record(
        &self,
        location: Location,
        mut budget: Option<&mut HintReadBudget>,
    ) -> Result<Option<Vec<u8>>> {
        if !self.compact_namespace() || location.record != 0 {
            return Err(invalid());
        }
        let connection = self.reader()?;
        let blob = connection.blob_open("main", "object_packs", "data", location.pack, true)?;
        let mut prefix = [0; 16];
        blob.read_at_exact(&mut prefix, 0)?;
        let count = header(&prefix, blob.len())?;
        let mut starts = [0; 1024];
        let starts = &mut starts[..4 * count];
        blob.read_at_exact(starts, 16)?;
        let range = range(starts, blob.len(), location.group)?;
        if budget.as_deref_mut().is_some_and(|budget| {
            !budget.charge(16 + starts.len() + range.len(), location.canonical_length)
        }) {
            return Ok(None);
        }
        let mut bytes = vec![0; range.len()];
        blob.read_at_exact(&mut bytes, range.start)?;
        record(&bytes, location.canonical_length)?;
        self.note_physical(PhysicalStorageReceipt {
            group_fetches: 1,
            blob_ranges: 3,
            encoded_read_bytes: (16 + starts.len() + bytes.len()) as u64,
            ..Default::default()
        });
        Ok(Some(bytes))
    }

    pub(super) fn read_whole(
        &self,
        id: ObjectId,
        location: Location,
        bytes: Vec<u8>,
        cache: &mut OwnerCache,
    ) -> Result<CanonicalObject> {
        let parsed = record(&bytes, location.canonical_length)?;
        let canonical = if let Some(range) = parsed.slice {
            let owner = parsed.base.ok_or_else(invalid)?;
            if owner == id {
                return Err(invalid());
            }
            if !cache.values.contains_key(&owner) {
                self.note_physical(PhysicalStorageReceipt {
                    base_fetches: 1,
                    ..Default::default()
                });
                let owner_location = self
                    .object_locations(&[owner])?
                    .remove(&owner)
                    .ok_or(StoreError::Integrity("whole-file slice owner missing"))?;
                if range.end
                    > owner_location
                        .canonical_length
                        .checked_sub(21)
                        .ok_or_else(invalid)?
                {
                    return Err(invalid());
                }
                let owner_record = self
                    .whole_record(owner_location, None)?
                    .ok_or_else(invalid)?;
                if record(&owner_record, owner_location.canonical_length)?.role != Role::Whole {
                    return Err(invalid());
                }
                cache.insert(
                    owner,
                    self.whole_chain(owner, owner_location, owner_record)?,
                );
            }
            let raw = Role::Whole.raw(&cache.values[&owner])?;
            let canonical = Role::Native.canonical(raw.get(range).ok_or_else(invalid)?)?;
            super::read::authenticate(id, &canonical, location.canonical_length)?;
            canonical
        } else {
            self.whole_chain(id, location, bytes)?
        };
        Ok(CanonicalObject {
            id,
            bytes: canonical,
        })
    }

    // Retain only 51 identifiers/locations, not a 64-MiB frame closure. Replay
    // refetches one frame at a time. Every base is authenticated before its child.
    // Together with the 4-MiB owner cache, the new format's active scratch is
    // bounded by 16 MiB, separately from caller-owned canonical output batches.
    fn whole_chain(
        &self,
        mut id: ObjectId,
        mut location: Location,
        mut bytes: Vec<u8>,
    ) -> Result<Vec<u8>> {
        let role = record(&bytes, location.canonical_length)?.role;
        let mut nodes = Vec::with_capacity(EDGES);
        let mut seen = Vec::with_capacity(EDGES + 1);
        let mut canonical_work = 0;
        let mut encoded_work = 0;
        let mut canonical;
        loop {
            if seen.contains(&id) {
                return Err(StoreError::Integrity("whole-file dependency cycle"));
            }
            seen.push(id);
            canonical_work += location.canonical_length;
            encoded_work += bytes.len();
            if canonical_work > CLOSURE || encoded_work > CLOSURE {
                return Err(StoreError::Integrity("whole-file closure bound"));
            }
            let parsed = record(&bytes, location.canonical_length)?;
            if parsed.role != role || parsed.slice.is_some() {
                return Err(invalid());
            }
            if let Some(base) = parsed.base {
                if nodes.len() == EDGES {
                    return Err(StoreError::Integrity("whole-file depth bound"));
                }
                self.note_physical(PhysicalStorageReceipt {
                    base_fetches: 1,
                    ..Default::default()
                });
                nodes.push((id, location, base));
                id = base;
                location = self
                    .object_locations(&[id])?
                    .remove(&id)
                    .ok_or(StoreError::Integrity("whole-file base missing"))?;
                bytes = self.whole_record(location, None)?.ok_or_else(invalid)?;
            } else {
                canonical = decode(parsed, None)?;
                super::read::authenticate(id, &canonical, location.canonical_length)?;
                self.note_physical(PhysicalStorageReceipt {
                    decompression_calls: 1,
                    decoded_read_bytes: canonical.len() as u64,
                    ..Default::default()
                });
                break;
            }
        }
        while let Some((id, location, base)) = nodes.pop() {
            bytes = self.whole_record(location, None)?.ok_or_else(invalid)?;
            let parsed = record(&bytes, location.canonical_length)?;
            // A concurrent product operation cannot mutate immutable packs. Still
            // reject role/shape changes across passes instead of relying on that.
            if parsed.role != role || parsed.slice.is_some() || parsed.base != Some(base) {
                return Err(invalid());
            }
            canonical = decode(parsed, Some(role.raw(&canonical)?))?;
            super::read::authenticate(id, &canonical, location.canonical_length)?;
            self.note_physical(PhysicalStorageReceipt {
                decompression_calls: 1,
                decoded_read_bytes: canonical.len() as u64,
                ..Default::default()
            });
        }
        Ok(canonical)
    }
}

fn decode(record: Record<'_>, prefix: Option<&[u8]>) -> Result<Vec<u8>> {
    if record.slice.is_some() || record.base.is_some() != prefix.is_some() {
        return Err(invalid());
    }
    let raw = if record.role == Role::Whole {
        pack::whole_decompress(record.frame, record.raw_length, prefix)?
    } else {
        pack::small_decompress(record.frame, record.raw_length, prefix)?
    };
    record.role.canonical(&raw)
}

#[cfg(test)]
#[path = "whole_tests.rs"]
mod tests;
