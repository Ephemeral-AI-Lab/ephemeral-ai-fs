//! Regular-file roots. Metadata ropes remain extent-only.
use super::{extent_codec, rope};
use crate::{CoreError, CoreResult, ObjectId};
use rope::{ObjectRead, ObjectStore, RopeCounters};
use std::io::{Read, Write};
use std::ops::Range;

pub const SMALL_LIMIT: usize = 131072;
pub const MAGIC: &[u8; 8] = b"LFS5SML\0";
pub const WHOLE_MAGIC: &[u8; 8] = b"LFSWFL1\0";
pub const WHOLE_LIMIT: usize = 2 * 1024 * 1024;

/// An authenticated physical owner for native chunk slices, not a logical file root.
pub fn whole_bytes(canonical: &[u8]) -> CoreResult<Option<&[u8]>> {
    let value = crate::decode_bytes_object(canonical)?;
    if !value.starts_with(WHOLE_MAGIC) {
        return Ok(None);
    }
    let raw = &value[8..];
    if !(SMALL_LIMIT..=WHOLE_LIMIT).contains(&raw.len()) {
        return Err(CoreError::InvalidRecord("whole-file owner length"));
    }
    Ok(Some(raw))
}

pub fn encode_whole(bytes: &[u8]) -> CoreResult<Vec<u8>> {
    if !(SMALL_LIMIT..=WHOLE_LIMIT).contains(&bytes.len()) {
        return Err(CoreError::InvalidRecord("whole-file owner length"));
    }
    let mut value = Vec::with_capacity(8 + bytes.len());
    value.extend_from_slice(WHOLE_MAGIC);
    value.extend_from_slice(bytes);
    crate::encode_bytes_object(&value)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileContentRoot(pub ObjectId);

impl From<rope::FileStateRoot> for FileContentRoot {
    fn from(root: rope::FileStateRoot) -> Self {
        Self(root.0)
    }
}

pub fn small_bytes(canonical: &[u8]) -> CoreResult<Option<&[u8]>> {
    small_value(crate::decode_bytes_object(canonical)?)
}

fn small_value(value: &[u8]) -> CoreResult<Option<&[u8]>> {
    if !value.starts_with(MAGIC) {
        return Ok(None);
    }
    if value.get(8..10) != Some(&[0, 1]) || !(11..SMALL_LIMIT + 10).contains(&value.len()) {
        return Err(CoreError::InvalidRecord("SmallContent framing"));
    }
    Ok(Some(&value[10..]))
}

/// Batch readers already own an authenticated outer Bytes value.
pub fn length_from_payload(value: &[u8]) -> CoreResult<u64> {
    match small_value(value)? {
        Some(raw) => Ok(raw.len() as u64),
        None => {
            Ok(extent_codec::decode_file_state(&crate::encode_bytes_object(value)?)?.logical_len)
        }
    }
}

pub fn encode_small(bytes: &[u8]) -> CoreResult<Vec<u8>> {
    if bytes.is_empty() || bytes.len() >= SMALL_LIMIT {
        return Err(CoreError::InvalidRecord("SmallContent length"));
    }
    let mut value = Vec::with_capacity(10 + bytes.len());
    value.extend_from_slice(MAGIC);
    value.extend_from_slice(&1u16.to_be_bytes());
    value.extend_from_slice(bytes);
    crate::encode_bytes_object(&value)
}

#[derive(Clone, Copy, Debug)]
pub enum Content {
    Small { logical_len: u64 },
    Chunked(super::extent::FileStateV3),
}
impl Content {
    pub fn logical_len(&self) -> u64 {
        match self {
            Self::Small { logical_len } => *logical_len,
            Self::Chunked(state) => state.logical_len,
        }
    }
}

pub fn inspect<S: ObjectRead>(store: &S, root: FileContentRoot) -> CoreResult<Content> {
    store.with_authenticated_canonical(root.0, |canonical| {
        Ok(match small_bytes(canonical)? {
            Some(bytes) => Content::Small {
                logical_len: bytes.len() as u64,
            },
            None => Content::Chunked(extent_codec::decode_file_state(canonical)?),
        })
    })
}

pub fn length<S: ObjectRead>(store: &S, root: FileContentRoot) -> CoreResult<u64> {
    Ok(inspect(store, root)?.logical_len())
}

pub fn read_range<S: ObjectRead, W: Write>(
    store: &S,
    root: FileContentRoot,
    range: Range<u64>,
    mut sink: W,
) -> CoreResult<RopeCounters> {
    let chunked = store.with_authenticated_canonical(root.0, |canonical| {
        let Some(bytes) = small_bytes(canonical)? else {
            return Ok(Some(extent_codec::decode_file_state(canonical)?));
        };
        if range.start > range.end || range.end > bytes.len() as u64 {
            return Err(CoreError::InvalidRange {
                start: range.start,
                end: range.end,
                length: bytes.len() as u64,
            });
        }
        sink.write_all(&bytes[range.start as usize..range.end as usize])
            .map_err(|_| CoreError::Io)?;
        Ok(None)
    })?;
    match chunked {
        None => Ok(RopeCounters {
            nodes_read: 1,
            payload_ids_read: 1,
            payload_batches_read: 1,
            max_payload_batch: 1,
            payload_bytes_read: range.end - range.start,
            ..Default::default()
        }),
        Some(state) => {
            let mut counters = RopeCounters {
                nodes_read: 1,
                ..Default::default()
            };
            let plan = rope::read_plan_from_state(store, state, &mut counters)?;
            rope::merge_rope_counters(
                &mut counters,
                rope::read_range_with_plan(store, &plan, range, sink)?,
            )?;
            Ok(counters)
        }
    }
}

pub fn read_all<S: ObjectRead, W: Write>(
    store: &S,
    root: FileContentRoot,
    sink: W,
) -> CoreResult<RopeCounters> {
    read_all_bounded(store, root, u64::MAX, sink)
}

pub fn read_all_bounded<S: ObjectRead, W: Write>(
    store: &S,
    root: FileContentRoot,
    maximum: u64,
    mut sink: W,
) -> CoreResult<RopeCounters> {
    let mut counters = RopeCounters {
        nodes_read: 1,
        ..Default::default()
    };
    let chunked = store.with_authenticated_canonical(root.0, |canonical| {
        let Some(bytes) = small_bytes(canonical)? else {
            return Ok(Some(extent_codec::decode_file_state(canonical)?));
        };
        if bytes.len() as u64 > maximum {
            return Err(CoreError::ObjectLimitExceeded);
        }
        sink.write_all(bytes).map_err(|_| CoreError::Io)?;
        counters.payload_bytes_read = bytes.len() as u64;
        counters.payload_ids_read = 1;
        counters.payload_batches_read = 1;
        counters.max_payload_batch = 1;
        Ok(None)
    })?;
    if let Some(state) = chunked {
        if state.logical_len > maximum {
            return Err(CoreError::ObjectLimitExceeded);
        }
        let plan = rope::read_plan_from_state(store, state, &mut counters)?;
        rope::merge_rope_counters(
            &mut counters,
            rope::read_range_with_plan(store, &plan, 0..state.logical_len, sink)?,
        )?;
    }
    Ok(counters)
}

pub fn validate_file<S: ObjectRead>(store: &S, root: FileContentRoot) -> CoreResult<()> {
    match inspect(store, root)? {
        Content::Small { .. } => Ok(()),
        Content::Chunked(_) => rope::validate_file(store, rope::FileStateRoot(root.0)),
    }
}

pub fn build_bytes<S: ObjectStore>(
    store: &mut S,
    bytes: &[u8],
) -> CoreResult<(FileContentRoot, RopeCounters)> {
    if store.small_content_format() && !bytes.is_empty() && bytes.len() < SMALL_LIMIT {
        let root = store.put_file_payload(encode_small(bytes)?, 0, bytes.len() as u32)?;
        return Ok((
            FileContentRoot(root),
            RopeCounters {
                payload_bytes_written: bytes.len() as u64,
                logical_len_after: Some(bytes.len() as u64),
                ..Default::default()
            },
        ));
    }
    rope::build_bytes(store, bytes).map(|(root, counters)| (FileContentRoot(root.0), counters))
}

pub fn build<S: ObjectStore, R: Read>(
    store: &mut S,
    mut source: R,
) -> CoreResult<(FileContentRoot, RopeCounters)> {
    if !store.small_content_format() {
        return rope::build(store, source)
            .map(|(root, counters)| (FileContentRoot(root.0), counters));
    }
    let mut prefix = Vec::with_capacity(SMALL_LIMIT);
    source
        .by_ref()
        .take(SMALL_LIMIT as u64)
        .read_to_end(&mut prefix)
        .map_err(|_| CoreError::Io)?;
    if prefix.len() < SMALL_LIMIT {
        return build_bytes(store, &prefix);
    }
    rope::build(store, prefix.as_slice().chain(source))
        .map(|(root, counters)| (FileContentRoot(root.0), counters))
}

pub fn replace<S: ObjectStore, R: Read>(
    store: &mut S,
    root: FileContentRoot,
    start: u64,
    delete_len: u64,
    mut replacement: R,
) -> CoreResult<(FileContentRoot, RopeCounters)> {
    let content = inspect(store, root)?;
    let len = content.logical_len();
    let end = start
        .checked_add(delete_len)
        .ok_or(CoreError::LengthOverflow)?;
    if end > len {
        return Err(CoreError::InvalidRange {
            start,
            end,
            length: len,
        });
    }
    let mut bytes = Vec::with_capacity(SMALL_LIMIT);
    replacement
        .by_ref()
        .take(SMALL_LIMIT as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| CoreError::Io)?;
    if delete_len == 0 && bytes.is_empty() {
        return Ok((root, RopeCounters::default()));
    }
    let final_minimum = (len - delete_len)
        .checked_add(bytes.len() as u64)
        .ok_or(CoreError::LengthOverflow)?;
    if matches!(content, Content::Chunked(_))
        && (!store.small_content_format() || final_minimum >= SMALL_LIMIT as u64)
    {
        return rope::replace(
            store,
            rope::FileStateRoot(root.0),
            start,
            delete_len,
            bytes.as_slice().chain(replacement),
        )
        .map(|(root, counters)| (FileContentRoot(root.0), counters));
    }
    // Only the retained bounded base is materialized. A large-to-small edit never reads discarded ranges.
    let mut head = Vec::new();
    let mut tail = Vec::new();
    read_range(store, root, 0..start, &mut head)?;
    read_range(store, root, end..len, &mut tail)?;
    build(
        store,
        head.as_slice()
            .chain(bytes.as_slice())
            .chain(replacement)
            .chain(tail.as_slice()),
    )
}
