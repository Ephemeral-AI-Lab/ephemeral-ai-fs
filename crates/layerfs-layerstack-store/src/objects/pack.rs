//! Wire version 1. Byte work only: no database, permits, or publication.
use crate::{Result, StoreError};
use layerfs_content::ObjectId;
use std::ops::Range;

pub(super) const GROUP_LIMIT: usize = 65_536;
pub(super) const PACK_LIMIT: usize = 256 * 1024;
pub(super) const GROUP_COUNT_LIMIT: usize = 256;
pub(super) const RECORD_COUNT_LIMIT: usize = 8191;
pub(super) const CANONICAL_LIMIT: usize = 16 * 1024 * 1024;
const MAGIC: &[u8; 8] = b"LFPACK\0\0";

fn invalid() -> StoreError {
    StoreError::Integrity("packed object framing")
}

fn u32_at(bytes: &[u8], offset: usize) -> Result<usize> {
    let end = offset.checked_add(4).ok_or_else(invalid)?;
    let value = bytes.get(offset..end).ok_or_else(invalid)?;
    usize::try_from(u32::from_le_bytes(value.try_into().map_err(|_| invalid())?))
        .map_err(|_| invalid())
}

fn put_u32(bytes: &mut Vec<u8>, value: usize) -> Result<()> {
    bytes.extend_from_slice(&u32::try_from(value).map_err(|_| invalid())?.to_le_bytes());
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Codec {
    Raw,
    Zstandard,
}

#[derive(Clone, Debug)]
pub(super) struct GroupEntry {
    pub range: Range<usize>,
    pub decoded_length: usize,
    pub codec: Codec,
    pub oversized: bool,
}

/// Validate length before a caller allocates even the directory or encoded group.
pub(super) fn header(bytes: &[u8; 16], blob_length: usize) -> Result<usize> {
    if blob_length > CANONICAL_LIMIT + 41 || &bytes[..8] != MAGIC || bytes[8..12] != [1, 0, 0, 0] {
        return Err(invalid());
    }
    let count = u32_at(bytes, 12)?;
    if !(1..=GROUP_COUNT_LIMIT).contains(&count) || blob_length <= 16 + 16 * count {
        return Err(invalid());
    }
    Ok(count)
}

/// Point extraction validates only the selected entry. Whole-pack admission uses
/// construction facts; it does not replay this parser over freshly built bytes.
pub(super) fn entry(
    bytes: &[u8; 16],
    group_count: usize,
    blob_length: usize,
) -> Result<GroupEntry> {
    if !(1..=GROUP_COUNT_LIMIT).contains(&group_count)
        || blob_length > CANONICAL_LIMIT + 41
        || bytes[13..] != [0, 0, 0]
    {
        return Err(invalid());
    }
    let start = u32_at(bytes, 0)?;
    let encoded = u32_at(bytes, 4)?;
    let decoded = u32_at(bytes, 8)?;
    let end = start.checked_add(encoded).ok_or_else(invalid)?;
    if encoded == 0 || decoded == 0 || start < 16 + 16 * group_count || end > blob_length {
        return Err(invalid());
    }
    let codec = match bytes[12] {
        0 if encoded == decoded => Codec::Raw,
        1 if encoded <= decoded && decoded <= GROUP_LIMIT => Codec::Zstandard,
        _ => return Err(invalid()),
    };
    let oversized = decoded > GROUP_LIMIT || blob_length > PACK_LIMIT;
    if oversized
        && (group_count != 1
            || codec != Codec::Raw
            || start != 32
            || end != blob_length
            || decoded > CANONICAL_LIMIT + 9)
    {
        return Err(invalid());
    }
    Ok(GroupEntry {
        range: start..end,
        decoded_length: decoded,
        codec,
        oversized,
    })
}

pub(super) enum Record<'a> {
    Full(&'a [u8]),
    Delta {
        base: ObjectId,
        output_length: usize,
        instructions: &'a [u8],
        count: usize,
    },
}

pub(super) fn record(bytes: &[u8]) -> Result<Record<'_>> {
    match bytes.first() {
        Some(0) if bytes.len() > 1 && bytes.len() <= CANONICAL_LIMIT + 1 => {
            Ok(Record::Full(&bytes[1..]))
        }
        Some(1) if bytes.len() >= 41 && bytes.len() <= GROUP_LIMIT => {
            let base = ObjectId::from_bytes(&bytes[1..33])?;
            let output_length = u32_at(bytes, 33)?;
            let count = u32_at(bytes, 37)?;
            if !(1..=GROUP_LIMIT).contains(&output_length)
                || !(1..=RECORD_COUNT_LIMIT).contains(&count)
            {
                return Err(invalid());
            }
            let instructions = &bytes[41..];
            // Validate complete consumption and output bounds before retaining a
            // program or acquiring its base. Base ranges are checked on application.
            walk_instructions(instructions, count, output_length, |_| Ok(()))?;
            Ok(Record::Delta {
                base,
                output_length,
                instructions,
                count,
            })
        }
        _ => Err(invalid()),
    }
}

enum Instruction<'a> {
    Copy { offset: usize, length: usize },
    Insert(&'a [u8]),
}

fn walk_instructions(
    bytes: &[u8],
    count: usize,
    output_length: usize,
    mut visit: impl FnMut(Instruction<'_>) -> Result<()>,
) -> Result<()> {
    if !(1..=RECORD_COUNT_LIMIT).contains(&count) || !(1..=GROUP_LIMIT).contains(&output_length) {
        return Err(invalid());
    }
    let mut cursor = 0usize;
    let mut output = 0usize;
    for _ in 0..count {
        let opcode = *bytes.get(cursor).ok_or_else(invalid)?;
        cursor = cursor.checked_add(1).ok_or_else(invalid)?;
        let (offset, length) = match opcode {
            0 => {
                let offset = u32_at(bytes, cursor)?;
                cursor = cursor.checked_add(4).ok_or_else(invalid)?;
                (Some(offset), u32_at(bytes, cursor)?)
            }
            1 => (None, u32_at(bytes, cursor)?),
            _ => return Err(invalid()),
        };
        cursor = cursor.checked_add(4).ok_or_else(invalid)?;
        output = output.checked_add(length).ok_or_else(invalid)?;
        if length == 0 || output > output_length {
            return Err(invalid());
        }
        if let Some(offset) = offset {
            visit(Instruction::Copy { offset, length })?;
        } else {
            let end = cursor.checked_add(length).ok_or_else(invalid)?;
            visit(Instruction::Insert(
                bytes.get(cursor..end).ok_or_else(invalid)?,
            ))?;
            cursor = end;
        }
    }
    if cursor != bytes.len() || output != output_length {
        return Err(invalid());
    }
    Ok(())
}

pub(super) fn apply_delta(
    instructions: &[u8],
    count: usize,
    output_length: usize,
    base: &[u8],
) -> Result<Vec<u8>> {
    if base.is_empty() || base.len() > GROUP_LIMIT || !(1..=GROUP_LIMIT).contains(&output_length) {
        return Err(invalid());
    }
    let mut output = Vec::with_capacity(output_length);
    walk_instructions(instructions, count, output_length, |instruction| {
        match instruction {
            Instruction::Copy { offset, length } => {
                let end = offset.checked_add(length).ok_or_else(invalid)?;
                output.extend_from_slice(base.get(offset..end).ok_or_else(invalid)?);
            }
            Instruction::Insert(bytes) => output.extend_from_slice(bytes),
        }
        Ok(())
    })?;
    Ok(output)
}

/// One forward parse per decoded group; callers select requested records from
/// requested records without retaining an entry-sized allocation for unrequested
/// records. A visitor result remains provisional until this complete parse succeeds.
pub(super) fn visit_records<'a>(
    bytes: &'a [u8],
    oversized: bool,
    mut visit: impl FnMut(usize, Record<'a>) -> Result<()>,
) -> Result<()> {
    if bytes.len()
        > if oversized {
            CANONICAL_LIMIT + 9
        } else {
            GROUP_LIMIT
        }
    {
        return Err(invalid());
    }
    let count = u32_at(bytes, 0)?;
    if !(1..=RECORD_COUNT_LIMIT).contains(&count) || (oversized && count != 1) {
        return Err(invalid());
    }
    let start = 4 + 4 * count;
    let area = bytes.get(start..).ok_or_else(invalid)?;
    let mut previous = 0;
    for index in 0..count {
        let end = u32_at(bytes, 4 + index * 4)?;
        if end <= previous || end > area.len() {
            return Err(invalid());
        }
        let parsed = record(&area[previous..end])?;
        if oversized && !matches!(parsed, Record::Full(value) if value.len() > GROUP_LIMIT - 9) {
            return Err(invalid());
        }
        visit(index, parsed)?;
        previous = end;
    }
    if previous != area.len() {
        return Err(invalid());
    }
    Ok(())
}

pub(super) struct EncodedGroup {
    pub bytes: Vec<u8>,
    pub decoded_length: usize,
    pub records: usize,
    pub codec: Codec,
}

pub(super) fn full_group(canonical: &[&[u8]]) -> Result<EncodedGroup> {
    if canonical.is_empty() || canonical.len() > RECORD_COUNT_LIMIT {
        return Err(invalid());
    }
    let framing = 4 + 5 * canonical.len();
    let length = canonical.iter().try_fold(framing, |sum, bytes| {
        if bytes.is_empty() || bytes.len() > CANONICAL_LIMIT {
            return Err(invalid());
        }
        sum.checked_add(bytes.len()).ok_or_else(invalid)
    })?;
    if length > GROUP_LIMIT && canonical.len() != 1 {
        return Err(invalid());
    }
    let mut bytes = Vec::with_capacity(length);
    put_u32(&mut bytes, canonical.len())?;
    let mut end = 0;
    for value in canonical {
        end += 1 + value.len(); // bounded by the checked total above
        put_u32(&mut bytes, end)?;
    }
    for value in canonical {
        bytes.push(0);
        bytes.extend_from_slice(value);
    }
    Ok(EncodedGroup {
        bytes,
        decoded_length: length,
        records: canonical.len(),
        codec: Codec::Raw,
    })
}

pub(super) fn assemble(groups: &[EncodedGroup]) -> Result<Vec<u8>> {
    if groups.is_empty() || groups.len() > GROUP_COUNT_LIMIT {
        return Err(invalid());
    }
    let directory_end = 16 + 16 * groups.len();
    let mut decoded = directory_end;
    let mut encoded = directory_end;
    let mut records = 0usize;
    for group in groups {
        decoded = decoded
            .checked_add(group.decoded_length)
            .ok_or_else(invalid)?;
        encoded = encoded.checked_add(group.bytes.len()).ok_or_else(invalid)?;
        records = records.checked_add(group.records).ok_or_else(invalid)?;
        if group.records == 0
            || group.bytes.is_empty()
            || (group.codec == Codec::Raw && group.bytes.len() != group.decoded_length)
            || (group.codec == Codec::Zstandard
                && (group.bytes.len() > group.decoded_length || group.decoded_length > GROUP_LIMIT))
        {
            return Err(invalid());
        }
    }
    let oversized = groups
        .iter()
        .any(|group| group.decoded_length > GROUP_LIMIT)
        || decoded > PACK_LIMIT;
    if records > RECORD_COUNT_LIMIT
        || (oversized
            && (groups.len() != 1
                || records != 1
                || groups[0].codec != Codec::Raw
                || decoded > CANONICAL_LIMIT + 41
                || groups[0].decoded_length <= GROUP_LIMIT))
    {
        return Err(invalid());
    }
    let mut bytes = Vec::with_capacity(encoded);
    bytes.extend_from_slice(MAGIC);
    bytes.extend_from_slice(&[1, 0, 0, 0]);
    put_u32(&mut bytes, groups.len())?;
    let mut offset = directory_end;
    for group in groups {
        put_u32(&mut bytes, offset)?;
        put_u32(&mut bytes, group.bytes.len())?;
        put_u32(&mut bytes, group.decoded_length)?;
        bytes.extend_from_slice(&[
            match group.codec {
                Codec::Raw => 0,
                Codec::Zstandard => 1,
            },
            0,
            0,
            0,
        ]);
        offset += group.bytes.len();
    }
    for group in groups {
        bytes.extend_from_slice(&group.bytes);
    }
    Ok(bytes)
}
