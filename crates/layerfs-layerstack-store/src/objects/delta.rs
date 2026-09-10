//! Pack-v3 whole-file record framing. The shared pinned codec owns static scratch.
use super::pack;
use crate::{Result, StoreError};
use layerfs_content::{file::content, ObjectId};

pub(super) const FRAME_LIMIT: usize = 135168;
pub(super) const CHAIN_EDGES: usize = 8;
pub(super) const CHAIN_CANONICAL_LIMIT: usize = 512 * 1024;
pub(super) const CHAIN_ENCODED_LIMIT: usize = 256 * 1024;
pub(super) struct Record<'a> {
    pub kind: u8,
    pub raw_length: usize,
    pub base: Option<ObjectId>,
    pub frame: &'a [u8],
}
fn invalid() -> StoreError {
    StoreError::Integrity("SmallContent physical record")
}

pub(super) fn compact_record_parts(bytes: &[u8]) -> Result<(u8, &[u8])> {
    let kind = *bytes.first().ok_or_else(invalid)?;
    let start = match kind { 0 => 1, 1 | 2 => 33, _ => return Err(invalid()) };
    let frame = bytes.get(start..).ok_or_else(invalid)?;
    if !(1..=FRAME_LIMIT).contains(&frame.len()) { return Err(invalid()); }
    Ok((kind, frame))
}

/// Restore the shared decoder's record in place. The indexed canonical length
/// is checked again by the frame decoder and full canonical authentication.
pub(super) fn expand_compact(bytes: &mut Vec<u8>, canonical_length: usize) -> Result<()> {
    let raw = canonical_length.checked_sub(23).ok_or_else(invalid)?;
    if !(1..content::SMALL_LIMIT).contains(&raw) { return Err(invalid()); }
    let (_, frame) = compact_record_parts(bytes)?;
    let frame_length = frame.len() as u32;
    let old_length = bytes.len();
    bytes.resize(old_length + 8, 0);
    bytes.copy_within(1..old_length, 9);
    bytes[1..5].copy_from_slice(&(raw as u32).to_le_bytes());
    bytes[5..9].copy_from_slice(&frame_length.to_le_bytes());
    record(bytes)?;
    Ok(())
}
pub(super) fn record(bytes: &[u8]) -> Result<Record<'_>> {
    if bytes.len() < 10 || bytes.len() > 192 * 1024 {
        return Err(invalid());
    }
    let raw_length = u32::from_le_bytes(bytes[1..5].try_into().unwrap()) as usize;
    let frame_length = u32::from_le_bytes(bytes[5..9].try_into().unwrap()) as usize;
    if !(1..content::SMALL_LIMIT).contains(&raw_length)
        || !(1..=FRAME_LIMIT).contains(&frame_length)
    {
        return Err(invalid());
    }
    let (base, start) = match bytes[0] {
        0 => (None, 9),
        1 | 2 => (
            Some(ObjectId::from_bytes(bytes.get(9..41).ok_or_else(invalid)?)?),
            41,
        ),
        _ => return Err(invalid()),
    };
    let frame = bytes.get(start..).ok_or_else(invalid)?;
    if frame.len() != frame_length {
        return Err(invalid());
    }
    Ok(Record {
        kind: bytes[0],
        raw_length,
        base,
        frame,
    })
}
pub(super) fn encode(
    kind: u8,
    raw_length: usize,
    base: Option<ObjectId>,
    frame: Vec<u8>,
) -> Result<pack::EncodedGroup> {
    if kind > 2 || (kind == 0) != base.is_none()
        || !(1..content::SMALL_LIMIT).contains(&raw_length) || !(1..=FRAME_LIMIT).contains(&frame.len())
    {
        return Err(invalid());
    }
    let mut bytes = Vec::with_capacity(9 + usize::from(base.is_some()) * 32 + frame.len());
    bytes.push(kind);
    bytes.extend_from_slice(&(raw_length as u32).to_le_bytes());
    bytes.extend_from_slice(&(frame.len() as u32).to_le_bytes());
    if let Some(base) = base {
        bytes.extend_from_slice(base.as_bytes());
    }
    bytes.extend_from_slice(&frame);
    Ok(pack::EncodedGroup {
        decoded_length: bytes.len(),
        bytes,
        records: 1,
        codec: pack::Codec::Raw,
    })
}
pub(super) fn decode(bytes: &[u8], base: Option<&[u8]>) -> Result<Vec<u8>> {
    let record = record(bytes)?;
    if record.base.is_some() != base.is_some() {
        return Err(invalid());
    }
    let raw = pack::small_decompress(record.frame, record.raw_length, base)?;
    Ok(content::encode_small(&raw)?)
}
