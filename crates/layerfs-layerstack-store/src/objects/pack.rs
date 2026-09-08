//! Explicit legacy/native wire grammars. Byte work only: no database or publication.
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

/// The old wrappers remain version-1-only so a legacy caller cannot reinterpret
/// native kind tags. Point readers opt into this explicit dispatch.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Version {
    Legacy,
    Native,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Header {
    pub version: Version,
    pub group_count: usize,
}

pub(super) fn versioned_header(bytes: &[u8; 16], blob_length: usize) -> Result<Header> {
    match u32_at(bytes, 8)? {
        1 => Ok(Header {
            version: Version::Legacy,
            group_count: header(bytes, blob_length)?,
        }),
        2 => {
            let count = u32_at(bytes, 12)?;
            if &bytes[..8] != MAGIC
                || blob_length > PACK_LIMIT
                || !(1..=GROUP_COUNT_LIMIT).contains(&count)
                || blob_length <= 16 + 16 * count
            {
                return Err(invalid());
            }
            Ok(Header {
                version: Version::Native,
                group_count: count,
            })
        }
        _ => Err(invalid()),
    }
}

pub(super) fn versioned_entry(
    bytes: &[u8; 16],
    header: Header,
    blob_length: usize,
) -> Result<GroupEntry> {
    let parsed = entry(bytes, header.group_count, blob_length)?;
    if header.version == Version::Native
        && (parsed.oversized || parsed.codec != Codec::Raw || blob_length > PACK_LIMIT)
    {
        return Err(invalid());
    }
    Ok(parsed)
}

pub(super) const NATIVE_RAW_LIMIT: usize = 32_768;
pub(super) const NATIVE_FRAME_LIMIT: usize = 33_024;
pub(super) const NATIVE_ENCODE_WORKSPACE: usize = 1_048_576;
pub(super) const NATIVE_DECODE_WORKSPACE: usize = 262_144;

#[derive(Debug)]
pub(super) enum NativeRecord<'a> {
    Full {
        raw_length: usize,
        frame: &'a [u8],
    },
    Prefix {
        raw_length: usize,
        base: ObjectId,
        frame: &'a [u8],
    },
}

pub(super) fn native_record(bytes: &[u8]) -> Result<NativeRecord<'_>> {
    let raw_length = u32_at(bytes, 1)?;
    if raw_length > NATIVE_RAW_LIMIT {
        return Err(invalid());
    }
    let (base, start) = match bytes.first() {
        Some(0) => (None, 5),
        Some(1) => (
            Some(ObjectId::from_bytes(bytes.get(5..37).ok_or_else(invalid)?)?),
            37,
        ),
        _ => return Err(invalid()),
    };
    let frame = bytes.get(start..).ok_or_else(invalid)?;
    if frame.is_empty() || frame.len() > NATIVE_FRAME_LIMIT {
        return Err(invalid());
    }
    Ok(match base {
        None => NativeRecord::Full { raw_length, frame },
        Some(base) => NativeRecord::Prefix {
            raw_length,
            base,
            frame,
        },
    })
}

/// Validate the COMPLETE end directory but return only one group-relative range.
/// The caller reads count/directory and selected bytes; neighboring bodies are
/// intentionally not authenticated by a point read.
pub(super) fn native_record_range(
    count: usize,
    ends: &[u8],
    group_length: usize,
    ordinal: usize,
) -> Result<Range<usize>> {
    if !(1..=RECORD_COUNT_LIMIT).contains(&count)
        || ordinal >= count
        || group_length > GROUP_LIMIT
        || ends.len() != 4 * count
    {
        return Err(invalid());
    }
    let area_start = 4 + ends.len();
    let area_length = group_length.checked_sub(area_start).ok_or_else(invalid)?;
    let mut previous = 0;
    let mut selected = 0..0;
    for index in 0..count {
        let end = u32_at(ends, 4 * index)?;
        if end <= previous || end > area_length {
            return Err(invalid());
        }
        if index == ordinal {
            selected = area_start + previous..area_start + end;
        }
        previous = end;
    }
    if previous != area_length {
        return Err(invalid());
    }
    Ok(selected)
}

pub(super) fn native_encode_record(
    raw_length: usize,
    base: Option<ObjectId>,
    frame: &[u8],
) -> Result<Vec<u8>> {
    if raw_length > NATIVE_RAW_LIMIT || frame.is_empty() || frame.len() > NATIVE_FRAME_LIMIT {
        return Err(invalid());
    }
    let mut bytes = Vec::with_capacity((if base.is_some() { 37 } else { 5 }) + frame.len());
    bytes.push(u8::from(base.is_some()));
    put_u32(&mut bytes, raw_length)?;
    if let Some(base) = base {
        bytes.extend_from_slice(base.as_bytes());
    }
    bytes.extend_from_slice(frame);
    Ok(bytes)
}

pub(super) fn native_group(records: &[&[u8]]) -> Result<EncodedGroup> {
    if records.is_empty() || records.len() > RECORD_COUNT_LIMIT {
        return Err(invalid());
    }
    let mut length = 4 + 4 * records.len();
    for record in records {
        native_record(record)?;
        length = length.checked_add(record.len()).ok_or_else(invalid)?;
    }
    if length > GROUP_LIMIT {
        return Err(invalid());
    }
    let mut bytes = Vec::with_capacity(length);
    put_u32(&mut bytes, records.len())?;
    let mut end = 0;
    for record in records {
        end += record.len();
        put_u32(&mut bytes, end)?;
    }
    for record in records {
        bytes.extend_from_slice(record);
    }
    Ok(EncodedGroup {
        bytes,
        decoded_length: length,
        records: records.len(),
        codec: Codec::Raw,
    })
}

pub(super) fn assemble_native(groups: &[EncodedGroup]) -> Result<Vec<u8>> {
    if groups
        .iter()
        .any(|group| group.codec != Codec::Raw || group.decoded_length > GROUP_LIMIT)
    {
        return Err(invalid());
    }
    // Existing assembly owns all pack-size/count/contiguous-directory checks.
    let mut bytes = assemble(groups)?;
    if bytes.len() > PACK_LIMIT {
        return Err(invalid());
    }
    bytes[8..12].copy_from_slice(&2u32.to_le_bytes());
    Ok(bytes)
}

/// One bounded encoder scratch allocation, owned by one admission preparation.
/// The cached context points only into owned scratch; borrowed inputs reset on every call.
pub(super) struct NativeEncoder {
    memory: Vec<u64>,
    context: Option<zstandard::NativeContext>,
}

impl NativeEncoder {
    pub(super) fn new() -> Result<Self> {
        Ok(Self {
            memory: zstandard::native_workspace()?,
            context: None,
        })
    }

    pub(super) fn compress(&mut self, raw: &[u8], prefix: Option<&[u8]>) -> Result<Vec<u8>> {
        zstandard::native_compress_in(&mut self.memory, &mut self.context, raw, prefix)
    }
}

#[cfg(test)]
pub(super) fn native_compress(raw: &[u8], prefix: Option<&[u8]>) -> Result<Vec<u8>> {
    NativeEncoder::new()?.compress(raw, prefix)
}

pub(super) fn native_decompress(
    frame: &[u8],
    raw_length: usize,
    prefix: Option<&[u8]>,
) -> Result<Vec<u8>> {
    zstandard::native_decompress(frame, raw_length, prefix)
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

/// Fixed seed table and one bounded program. A budget-exhausted trial is never
/// returned; previously completed candidates remain owned by the caller.
pub(super) fn delta_record(
    base_id: ObjectId,
    base: &[u8],
    target: &[u8],
    remaining: &mut usize,
    stats: &mut crate::telemetry::PhysicalStorageReceipt,
) -> Result<Option<Vec<u8>>> {
    if base.is_empty()
        || base.len() > GROUP_LIMIT
        || target.is_empty()
        || target.len() > GROUP_LIMIT
    {
        return Err(invalid());
    }
    fn charge(remaining: &mut usize, bytes: usize) -> bool {
        if bytes > *remaining {
            *remaining = 0;
            false
        } else {
            *remaining -= bytes;
            true
        }
    }
    fn bucket(seed: &[u8]) -> usize {
        let hash = blake3::hash(seed);
        usize::from(u16::from_le_bytes([hash.as_bytes()[0], hash.as_bytes()[1]])) & 4095
    }
    fn instruction(
        out: &mut Vec<u8>,
        count: &mut usize,
        offset: Option<usize>,
        bytes: &[u8],
        length: usize,
        limit: usize,
    ) -> Result<bool> {
        let size = if offset.is_some() { 9 } else { 5 + length };
        if *count == RECORD_COUNT_LIMIT || out.len() + size > limit {
            return Ok(false);
        }
        out.push(u8::from(offset.is_none()));
        if let Some(offset) = offset {
            put_u32(out, offset)?;
        }
        put_u32(out, length)?;
        if offset.is_none() {
            out.extend_from_slice(bytes);
        }
        *count += 1;
        Ok(true)
    }
    // ponytail: four seed positions per bucket can miss better matches; expand
    // only under a separately approved search/memory policy.
    let mut table = vec![[u32::MAX; 4]; 4096];
    for offset in (0..base.len().saturating_sub(15)).step_by(16) {
        if !charge(remaining, 16) {
            stats.match_budget_skips += 1;
            return Ok(None);
        }
        stats.seed_hash_bytes += 16;
        let slots = &mut table[bucket(&base[offset..offset + 16])];
        if let Some(slot) = slots.iter_mut().find(|slot| **slot == u32::MAX) {
            *slot = offset as u32;
        }
    }
    let limit = (target.len() + 1).min(GROUP_LIMIT);
    if limit < 41 {
        return Ok(None);
    }
    let mut output = Vec::with_capacity(limit);
    output.push(1);
    output.extend_from_slice(base_id.as_bytes());
    put_u32(&mut output, target.len())?;
    put_u32(&mut output, 0)?;
    let mut count = 0;
    let mut cursor = 0;
    let mut literal = 0;
    while cursor + 16 <= target.len() {
        if !charge(remaining, 16) {
            stats.match_budget_skips += 1;
            return Ok(None);
        }
        stats.seed_hash_bytes += 16;
        let mut best = (0usize, 0usize);
        for offset in table[bucket(&target[cursor..cursor + 16])] {
            if offset == u32::MAX {
                break;
            }
            let offset = offset as usize;
            let mut length = 0;
            while offset + length < base.len() && cursor + length < target.len() {
                if !charge(remaining, 1) {
                    stats.match_budget_skips += 1;
                    return Ok(None);
                }
                stats.match_comparisons += 1;
                if base[offset + length] != target[cursor + length] {
                    break;
                }
                length += 1;
            }
            if length >= 16 && (length > best.1 || (length == best.1 && offset < best.0)) {
                best = (offset, length);
            }
        }
        if best.1 == 0 {
            cursor += 1;
            continue;
        }
        if cursor > literal
            && !instruction(
                &mut output,
                &mut count,
                None,
                &target[literal..cursor],
                cursor - literal,
                limit,
            )?
        {
            stats.instruction_budget_skips += u64::from(count == RECORD_COUNT_LIMIT);
            return Ok(None);
        }
        if !instruction(&mut output, &mut count, Some(best.0), &[], best.1, limit)? {
            stats.instruction_budget_skips += u64::from(count == RECORD_COUNT_LIMIT);
            return Ok(None);
        }
        cursor += best.1;
        literal = cursor;
    }
    if literal < target.len()
        && !instruction(
            &mut output,
            &mut count,
            None,
            &target[literal..],
            target.len() - literal,
            limit,
        )?
    {
        stats.instruction_budget_skips += u64::from(count == RECORD_COUNT_LIMIT);
        return Ok(None);
    }
    if count == 0 {
        return Ok(None);
    }
    output[37..41].copy_from_slice(&(count as u32).to_le_bytes());
    Ok(Some(output))
}

/// Membership is chosen by FULL sizes by the admission owner. Candidate records
/// include kind/header and are bounded by the corresponding FULL record size.
pub(super) fn encode_group(
    canonical: &[&[u8]],
    deltas: &[Option<Vec<u8>>],
    stats: &mut crate::telemetry::PhysicalStorageReceipt,
) -> Result<(EncodedGroup, bool)> {
    let full = group_alternative(canonical, &[], stats)?;
    stats.full_alternative_bytes += full.bytes.len() as u64;
    if deltas.iter().any(Option::is_some) && full.decoded_length <= GROUP_LIMIT {
        let mixed = group_alternative(canonical, deltas, stats)?;
        stats.mixed_alternative_bytes += mixed.bytes.len() as u64;
        if full.bytes.len().saturating_sub(mixed.bytes.len())
            >= 64.max(full.bytes.len().div_ceil(8))
        {
            stats.selected_encoded_bytes += mixed.bytes.len() as u64;
            return Ok((mixed, true));
        }
        stats.rejected_mixed_groups += 1;
    }
    stats.selected_encoded_bytes += full.bytes.len() as u64;
    Ok((full, false))
}

fn group_alternative(
    canonical: &[&[u8]],
    deltas: &[Option<Vec<u8>>],
    stats: &mut crate::telemetry::PhysicalStorageReceipt,
) -> Result<EncodedGroup> {
    if canonical.is_empty()
        || canonical.len() > RECORD_COUNT_LIMIT
        || (!deltas.is_empty() && deltas.len() != canonical.len())
    {
        return Err(invalid());
    }
    let framing = 4 + 4 * canonical.len();
    let mut length = framing;
    for (index, value) in canonical.iter().enumerate() {
        if value.is_empty() || value.len() > CANONICAL_LIMIT {
            return Err(invalid());
        }
        let size = deltas
            .get(index)
            .and_then(Option::as_ref)
            .map_or(value.len() + 1, Vec::len);
        if size > value.len() + 1 {
            return Err(invalid());
        }
        length = length.checked_add(size).ok_or_else(invalid)?;
    }
    if length > GROUP_LIMIT && canonical.len() != 1 {
        return Err(invalid());
    }
    let mut bytes = Vec::with_capacity(length);
    put_u32(&mut bytes, canonical.len())?;
    let mut end = 0;
    for (index, value) in canonical.iter().enumerate() {
        end += deltas
            .get(index)
            .and_then(Option::as_ref)
            .map_or(value.len() + 1, Vec::len);
        put_u32(&mut bytes, end)?;
    }
    for (index, value) in canonical.iter().enumerate() {
        if let Some(delta) = deltas.get(index).and_then(Option::as_ref) {
            bytes.extend_from_slice(delta);
        } else {
            bytes.push(0);
            bytes.extend_from_slice(value);
        }
    }
    let codec = if length <= GROUP_LIMIT {
        let started = std::time::Instant::now();
        let result = zstandard::compress(&bytes);
        stats.encoding_calls += 1;
        stats.encoding_ns += super::elapsed_ns(started);
        let compressed = result?;
        if compressed.len() + 16 <= length {
            bytes = compressed;
            Codec::Zstandard
        } else {
            Codec::Raw
        }
    } else {
        Codec::Raw
    };
    Ok(EncodedGroup {
        bytes,
        decoded_length: length,
        records: canonical.len(),
        codec,
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

/// Consumes the selected BLOB range; RAW transfers ownership without a copy.
/// Framing and declared sizes are checked before any decoder/output allocation.
pub(super) fn decode_group(entry: GroupEntry, encoded: Vec<u8>) -> Result<Vec<u8>> {
    if entry.range.end.checked_sub(entry.range.start) != Some(encoded.len())
        || encoded.is_empty()
        || entry.decoded_length == 0
    {
        return Err(invalid());
    }
    match entry.codec {
        Codec::Raw if encoded.len() == entry.decoded_length => Ok(encoded),
        Codec::Zstandard
            if !entry.oversized
                && entry.decoded_length <= GROUP_LIMIT
                && encoded.len() <= entry.decoded_length =>
        {
            zstandard::decompress(&encoded, entry.decoded_length)
        }
        _ => Err(invalid()),
    }
}

// zstd-safe has no static-workspace API. Keep the small FFI boundary here:
// static contexts cannot malloc/realloc, including when an estimate is too small.
// The pinned library's one-shot API needs no separate streaming window buffer.
#[allow(unsafe_code)]
mod zstandard {
    use super::{invalid, Result, StoreError, GROUP_LIMIT};
    use zstd_sys::*;

    const ENCODE_CONTEXT_LIMIT: usize = 1024 * 1024;
    const DECODE_CONTEXT_LIMIT: usize = 256 * 1024;

    fn resource() -> StoreError {
        StoreError::Io(std::io::Error::other(
            "bounded Zstandard workspace unavailable",
        ))
    }

    fn checked(code: usize) -> Result<usize> {
        // SAFETY: ZSTD_isError only interprets the numeric return code.
        if unsafe { ZSTD_isError(code) } != 0 {
            return Err(StoreError::Integrity("Zstandard codec failure"));
        }
        Ok(code)
    }

    // Only optional native encoding may fall back on a workspace shortage.
    // Other codec failures remain integrity failures; legacy/decode callers keep
    // using checked() and retain their existing error policy.
    fn native_encode_checked(code: usize) -> Result<usize> {
        // SAFETY: This API only interprets the numeric codec return value.
        if unsafe { ZSTD_getErrorCode(code) } == ZSTD_ErrorCode::ZSTD_error_memory_allocation {
            return Err(resource());
        }
        checked(code)
    }

    fn workspace(size: usize, limit: usize) -> Result<Vec<u64>> {
        let size = checked(size)?;
        if size == 0 || size > limit {
            return Err(resource());
        }
        let mut words = Vec::new();
        words
            .try_reserve_exact(size.div_ceil(8))
            .map_err(|_| resource())?;
        words.resize(size.div_ceil(8), 0u64);
        // u64 alignment is eight on supported targets; enforce the C contract
        // even on a target whose Rust u64 alignment is smaller.
        if words.capacity() > limit / 8 || words.as_ptr().align_offset(8) != 0 {
            return Err(resource());
        }
        Ok(words)
    }

    fn output(size: usize) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();
        bytes.try_reserve_exact(size).map_err(|_| resource())?;
        bytes.resize(size, 0);
        Ok(bytes)
    }

    use super::{
        NATIVE_DECODE_WORKSPACE, NATIVE_ENCODE_WORKSPACE, NATIVE_FRAME_LIMIT, NATIVE_RAW_LIMIT,
    };

    /// Exact S2 requested setter sequence, shared with the dynamic-equivalence
    /// test. No CParams substitution or parameter adjustment to fit workspace.
    unsafe fn native_parameters(context: *mut ZSTD_CCtx) -> Result<()> {
        // SAFETY: Caller supplies a live initialized context, exclusively owned.
        unsafe {
            checked(ZSTD_CCtx_reset(
                context,
                ZSTD_ResetDirective::ZSTD_reset_session_and_parameters,
            ))?;
            for (parameter, value) in [
                (ZSTD_cParameter::ZSTD_c_compressionLevel, 3),
                (ZSTD_cParameter::ZSTD_c_windowLog, 20),
                (ZSTD_cParameter::ZSTD_c_contentSizeFlag, 1),
                (ZSTD_cParameter::ZSTD_c_checksumFlag, 1),
                (ZSTD_cParameter::ZSTD_c_dictIDFlag, 0),
                (ZSTD_cParameter::ZSTD_c_nbWorkers, 0),
            ] {
                checked(ZSTD_CCtx_setParameter(context, parameter, value))?;
            }
        }
        Ok(())
    }

    pub(super) fn native_workspace() -> Result<Vec<u64>> {
        workspace(NATIVE_ENCODE_WORKSPACE, NATIVE_ENCODE_WORKSPACE)
    }

    #[cfg(test)]
    fn native_compress(raw: &[u8], prefix: Option<&[u8]>) -> Result<Vec<u8>> {
        super::NativeEncoder::new()?.compress(raw, prefix)
    }

    pub(super) struct NativeContext {
        context: *mut ZSTD_CCtx,
        workspace: *mut u64,
        words: usize,
    }

    #[cfg(test)]
    thread_local! { static NATIVE_CONTEXT_INITIALIZATIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) }; }

    pub(super) fn native_compress_in(
        memory: &mut [u64],
        cached: &mut Option<NativeContext>,
        raw: &[u8],
        prefix: Option<&[u8]>,
    ) -> Result<Vec<u8>> {
        let prefix = prefix.unwrap_or(&[]);
        if raw.len() > NATIVE_RAW_LIMIT || prefix.len() > NATIVE_RAW_LIMIT {
            return Err(invalid());
        }
        // SAFETY: All workspaces are live, aligned and disjoint. Prefix is a
        // borrowed raw prefix (refPrefix's default), never a dictionary parser.
        // The static context has no C free and cannot grow beyond this region.
        unsafe {
            let workspace = memory.as_mut_ptr();
            let words = memory.len();
            if cached
                .as_ref()
                .is_none_or(|state| state.workspace != workspace || state.words != words)
            {
                *cached = None;
                #[cfg(test)]
                NATIVE_CONTEXT_INITIALIZATIONS.with(|count| count.set(count.get() + 1));
                let context = ZSTD_initStaticCCtx(workspace.cast(), words * 8);
                if context.is_null() {
                    return Err(resource());
                }
                *cached = Some(NativeContext {
                    context,
                    workspace,
                    words,
                });
            }
            let context = cached.as_ref().unwrap().context;
            // Reset settings and borrowed operands, while preserving the bounded
            // context/workspace allocations and the exact frozen frame parameters.
            let result = (|| {
                native_parameters(context)?;
                checked(ZSTD_CCtx_refPrefix(
                    context,
                    if prefix.is_empty() {
                        std::ptr::null()
                    } else {
                        prefix.as_ptr().cast()
                    },
                    prefix.len(),
                ))?;
                let bound = checked(ZSTD_compressBound(raw.len()))?;
                if bound > NATIVE_FRAME_LIMIT {
                    return Err(resource());
                }
                let mut encoded = output(bound)?;
                if encoded.capacity() > NATIVE_FRAME_LIMIT {
                    return Err(resource());
                }
                let length = native_encode_checked(ZSTD_compress2(
                    context,
                    encoded.as_mut_ptr().cast(),
                    encoded.len(),
                    raw.as_ptr().cast(),
                    raw.len(),
                ))?;
                if length == 0 || length > encoded.len() {
                    return Err(invalid());
                }
                encoded.truncate(length);
                Ok(encoded)
            })();
            // Clear borrowed prefix/input references on success AND failure,
            // before the caller can release their buffers. Reset does not read
            // their contents and the backing remains exclusively owned here.
            let reset = checked(ZSTD_CCtx_reset(
                context,
                ZSTD_ResetDirective::ZSTD_reset_session_and_parameters,
            ));
            if result.is_err() || reset.is_err() {
                *cached = None;
            }
            match result {
                Err(error) => Err(error),
                Ok(encoded) => {
                    reset?;
                    Ok(encoded)
                }
            }
        }
    }

    pub(super) fn native_decompress(
        encoded: &[u8],
        length: usize,
        prefix: Option<&[u8]>,
    ) -> Result<Vec<u8>> {
        let prefix = prefix.unwrap_or(&[]);
        if length > NATIVE_RAW_LIMIT
            || prefix.len() > NATIVE_RAW_LIMIT
            || encoded.is_empty()
            || encoded.len() > NATIVE_FRAME_LIMIT
            || encoded.get(..4) != Some(&[0x28, 0xb5, 0x2f, 0xfd])
            || encoded
                .get(4)
                .is_none_or(|descriptor| descriptor & 0x1b != 0)
        {
            return Err(invalid());
        }
        // SAFETY: Header parsing only borrows encoded. DCtx and DDict have
        // separate caller-owned aligned regions; DDict borrows raw prefix until
        // one-shot decoding completes. Neither static object uses a C free.
        // DCtx_refPrefix is deliberately avoided: it creates a heap DDict.
        unsafe {
            let mut header = std::mem::MaybeUninit::<ZSTD_FrameHeader>::uninit();
            if checked(ZSTD_getFrameHeader(
                header.as_mut_ptr(),
                encoded.as_ptr().cast(),
                encoded.len(),
            ))? != 0
            {
                return Err(invalid());
            }
            let header = header.assume_init();
            if header.frameType != ZSTD_FrameType_e::ZSTD_frame
                || header.frameContentSize != length as u64
                || header.windowSize > 1_048_576
                || header.dictID != 0
                || header.checksumFlag != 1
                || checked(ZSTD_findFrameCompressedSize(
                    encoded.as_ptr().cast(),
                    encoded.len(),
                ))? != encoded.len()
            {
                return Err(invalid());
            }
            let mut memory = workspace(ZSTD_estimateDCtxSize(), NATIVE_DECODE_WORKSPACE)?;
            let context = ZSTD_initStaticDCtx(memory.as_mut_ptr().cast(), memory.len() * 8);
            if context.is_null() {
                return Err(resource());
            }
            checked(ZSTD_DCtx_reset(
                context,
                ZSTD_ResetDirective::ZSTD_reset_session_and_parameters,
            ))?;
            checked(ZSTD_DCtx_setParameter(
                context,
                ZSTD_dParameter::ZSTD_d_windowLogMax,
                20,
            ))?;
            let remaining = NATIVE_DECODE_WORKSPACE
                .checked_sub(memory.capacity() * 8)
                .ok_or_else(resource)?;
            let mut dictionary = if prefix.is_empty() {
                Vec::new()
            } else {
                workspace(
                    ZSTD_estimateDDictSize(prefix.len(), ZSTD_dictLoadMethod_e::ZSTD_dlm_byRef),
                    remaining,
                )?
            };
            let ddict = if prefix.is_empty() {
                std::ptr::null()
            } else {
                let ptr = ZSTD_initStaticDDict(
                    dictionary.as_mut_ptr().cast(),
                    dictionary.len() * 8,
                    prefix.as_ptr().cast(),
                    prefix.len(),
                    ZSTD_dictLoadMethod_e::ZSTD_dlm_byRef,
                    ZSTD_dictContentType_e::ZSTD_dct_rawContent,
                );
                if ptr.is_null() {
                    return Err(resource());
                }
                ptr
            };
            let mut decoded = output(length)?;
            if decoded.capacity() > NATIVE_RAW_LIMIT {
                return Err(resource());
            }
            if checked(ZSTD_decompress_usingDDict(
                context,
                decoded.as_mut_ptr().cast(),
                decoded.len(),
                encoded.as_ptr().cast(),
                encoded.len(),
                ddict,
            ))? != length
            {
                return Err(invalid());
            }
            Ok(decoded)
        }
    }

    pub(super) fn compress(raw: &[u8]) -> Result<Vec<u8>> {
        if raw.is_empty() || raw.len() > GROUP_LIMIT {
            return Err(invalid());
        }
        // SAFETY: All C pointers borrow live, non-overlapping Rust allocations.
        // The aligned workspace outlives its context; static contexts must not
        // be freed with ZSTD_freeCCtx. No pointer escapes this one-shot call.
        unsafe {
            let mut parameters = ZSTD_getCParams(1, raw.len() as u64, 0);
            parameters.windowLog = parameters.windowLog.min(16);
            let mut memory = workspace(
                ZSTD_estimateCCtxSize_usingCParams(parameters),
                ENCODE_CONTEXT_LIMIT,
            )?;
            #[cfg(test)]
            NATIVE_CONTEXT_INITIALIZATIONS.with(|count| count.set(count.get() + 1));
            let context = ZSTD_initStaticCCtx(memory.as_mut_ptr().cast(), memory.len() * 8);
            if context.is_null() {
                return Err(resource());
            }
            checked(ZSTD_CCtx_setCParams(context, parameters))?;
            checked(ZSTD_CCtx_setFParams(
                context,
                ZSTD_frameParameters {
                    contentSizeFlag: 1,
                    checksumFlag: 1,
                    noDictIDFlag: 1,
                },
            ))?;
            let bound = checked(ZSTD_compressBound(raw.len()))?;
            if bound > GROUP_LIMIT + 1024 {
                return Err(resource());
            }
            let mut encoded = output(bound)?;
            let length = checked(ZSTD_compress2(
                context,
                encoded.as_mut_ptr().cast(),
                encoded.len(),
                raw.as_ptr().cast(),
                raw.len(),
            ))?;
            encoded.truncate(length);
            Ok(encoded)
        }
    }

    pub(super) fn decompress(encoded: &[u8], length: usize) -> Result<Vec<u8>> {
        // Reject dictionary fields (including explicit ID zero), reserved bits,
        // and nonordinary frame magic before even parsing the frame header.
        if encoded.get(..4) != Some(&[0x28, 0xb5, 0x2f, 0xfd])
            || encoded
                .get(4)
                .is_none_or(|descriptor| descriptor & 0x1b != 0)
        {
            return Err(invalid());
        }
        // SAFETY: Header parsing and frame-size scanning only read encoded.
        // getFrameHeader returning zero initializes every field used below.
        // The checked fixed workspace and output remain live until decoding ends;
        // static contexts neither allocate nor require a C free operation.
        unsafe {
            let mut header = std::mem::MaybeUninit::<ZSTD_FrameHeader>::uninit();
            if checked(ZSTD_getFrameHeader(
                header.as_mut_ptr(),
                encoded.as_ptr().cast(),
                encoded.len(),
            ))? != 0
            {
                return Err(invalid());
            }
            let header = header.assume_init();
            if header.frameType != ZSTD_FrameType_e::ZSTD_frame
                || header.frameContentSize != length as u64
                || header.windowSize > GROUP_LIMIT as u64
                || header.dictID != 0
                || header.checksumFlag != 1
                || checked(ZSTD_findFrameCompressedSize(
                    encoded.as_ptr().cast(),
                    encoded.len(),
                ))? != encoded.len()
            {
                return Err(invalid());
            }
            let mut memory = workspace(ZSTD_estimateDCtxSize(), DECODE_CONTEXT_LIMIT)?;
            let context = ZSTD_initStaticDCtx(memory.as_mut_ptr().cast(), memory.len() * 8);
            if context.is_null() {
                return Err(resource());
            }
            let mut decoded = output(length)?;
            // The default decoder verifies the required checksum. Destination
            // capacity is the validated declared size, never a frame-derived grow.
            if checked(ZSTD_decompressDCtx(
                context,
                decoded.as_mut_ptr().cast(),
                decoded.len(),
                encoded.as_ptr().cast(),
                encoded.len(),
            ))? != length
            {
                return Err(invalid());
            }
            Ok(decoded)
        }
    }
    #[cfg(test)]
    mod native_tests {
        use super::*;

        // A dynamic control is test-only and uses the exact frozen setter
        // sequence. Product execution never retries static failures dynamically.
        fn dynamic_frame(raw: &[u8], prefix: &[u8]) -> Vec<u8> {
            struct Context(*mut ZSTD_CCtx);
            impl Drop for Context {
                fn drop(&mut self) {
                    // SAFETY: This test owns a dynamic context, unlike product.
                    unsafe {
                        ZSTD_freeCCtx(self.0);
                    }
                }
            }
            // SAFETY: Owned context plus disjoint live borrowed slices/output.
            unsafe {
                let context = Context(ZSTD_createCCtx());
                assert!(!context.0.is_null());
                native_parameters(context.0).unwrap();
                checked(ZSTD_CCtx_refPrefix(
                    context.0,
                    prefix.as_ptr().cast(),
                    prefix.len(),
                ))
                .unwrap();
                let mut frame = vec![0; ZSTD_compressBound(raw.len())];
                let n = checked(ZSTD_compress2(
                    context.0,
                    frame.as_mut_ptr().cast(),
                    frame.len(),
                    raw.as_ptr().cast(),
                    raw.len(),
                ))
                .unwrap();
                frame.truncate(n);
                frame
            }
        }

        #[test]
        fn native_static_workspace_exhaustion_is_resource_only() {
            // Pinned compress.c returns memory_allocation when a static CCtx
            // cannot resize. This smaller workspace is TEST-ONLY: production
            // continues to use the frozen 1-MiB region and exact parameters.
            // SAFETY: All input/output/workspaces are disjoint live allocations;
            // the static context is never passed to a C free function.
            unsafe {
                let mut memory = workspace(128 * 1024, 128 * 1024).unwrap();
                #[cfg(test)]
                NATIVE_CONTEXT_INITIALIZATIONS.with(|count| count.set(count.get() + 1));
                let context = ZSTD_initStaticCCtx(memory.as_mut_ptr().cast(), memory.len() * 8);
                assert!(
                    !context.is_null(),
                    "fixture must initialize before encoding exhausts workspace"
                );
                native_parameters(context).unwrap();
                checked(ZSTD_CCtx_refPrefix(context, std::ptr::null(), 0)).unwrap();
                let raw = vec![b'x'; NATIVE_RAW_LIMIT];
                let mut frame = vec![0; ZSTD_compressBound(raw.len())];
                let code = ZSTD_compress2(
                    context,
                    frame.as_mut_ptr().cast(),
                    frame.len(),
                    raw.as_ptr().cast(),
                    raw.len(),
                );
                assert_eq!(
                    ZSTD_getErrorCode(code),
                    ZSTD_ErrorCode::ZSTD_error_memory_allocation
                );
                assert!(matches!(
                    native_encode_checked(code),
                    Err(StoreError::Io(_))
                ));
                assert!(matches!(checked(code), Err(StoreError::Integrity(_))));
                let other =
                    0usize.wrapping_sub(ZSTD_ErrorCode::ZSTD_error_dstSize_tooSmall as usize);
                assert!(matches!(
                    native_encode_checked(other),
                    Err(StoreError::Integrity(_))
                ));
                assert_eq!(native_encode_checked(42).unwrap(), 42);
            }
        }

        #[test]
        fn native_encoder_reuses_bounded_scratch_without_retaining_prefixes() {
            let mut encoder = super::super::NativeEncoder::new().unwrap();
            let address = encoder.memory.as_ptr();
            let capacity = encoder.memory.capacity();
            assert_eq!(capacity * 8, NATIVE_ENCODE_WORKSPACE);
            let mut state = 0x174ab28du32;
            let raw: Vec<u8> = (0..32768)
                .map(|_| {
                    state ^= state << 13;
                    state ^= state >> 17;
                    state ^= state << 5;
                    state as u8
                })
                .collect();
            for (index, variant) in [0, 1, 2, 1, 0, 2].into_iter().enumerate() {
                // Each borrowed prefix is destroyed before the next call.
                let prefix = match variant {
                    0 => Some(raw.clone()),
                    1 => None,
                    _ => Some(vec![b'x'; 16384]),
                };
                let before = NATIVE_CONTEXT_INITIALIZATIONS.with(|count| count.get());
                let frame = encoder.compress(&raw, prefix.as_deref()).unwrap();
                let after = NATIVE_CONTEXT_INITIALIZATIONS.with(|count| count.get());
                assert_eq!(
                    after - before,
                    usize::from(index == 0),
                    "stable native workspace must retain its initialized context"
                );
                assert_eq!(frame, native_compress(&raw, prefix.as_deref()).unwrap());
                assert_eq!(frame, dynamic_frame(&raw, prefix.as_deref().unwrap_or(&[])));
                assert_eq!(
                    native_decompress(&frame, raw.len(), prefix.as_deref()).unwrap(),
                    raw
                );
                drop(prefix);
                assert_eq!(encoder.memory.as_ptr(), address);
                assert_eq!(encoder.memory.capacity(), capacity);
            }
            let oversized = vec![0; NATIVE_RAW_LIMIT + 1];
            assert!(encoder.compress(&oversized, None).is_err());
            assert!(encoder.compress(&raw, Some(&oversized)).is_err());
            // Exercise an error AFTER binding a borrowed prefix. Only this
            // test supplies a smaller slice; production always owns 1 MiB.
            {
                let prefix = vec![b'x'; NATIVE_RAW_LIMIT];
                let large = vec![b'x'; NATIVE_RAW_LIMIT];
                assert!(matches!(
                    native_compress_in(
                        &mut encoder.memory[..128 * 1024 / 8],
                        &mut encoder.context,
                        &large,
                        Some(&prefix)
                    ),
                    Err(StoreError::Io(_))
                ));
            }
            let frame = encoder.compress(&raw, None).unwrap();
            assert_eq!(frame, dynamic_frame(&raw, &[]));
            assert_eq!(native_decompress(&frame, raw.len(), None).unwrap(), raw);
            assert_eq!(encoder.memory.as_ptr(), address);
            assert_eq!(encoder.memory.capacity(), capacity);
        }

        #[test]
        fn native_static_fit_equivalence_and_strict_frames() {
            // Source-size boundaries cover tiny inputs and the pinned level-3
            // size-table regimes; exercise empty, tiny and maximum prefixes.
            // SAFETY: This API returns the linked library's immutable version.
            assert_eq!(unsafe { ZSTD_versionNumber() }, 10507);
            let sizes = [
                0, 1, 7, 8, 15, 16, 31, 32, 63, 64, 127, 128, 255, 256, 511, 512, 1023, 1024, 2047,
                2048, 4095, 4096, 8191, 8192, 16383, 16384, 32767, 32768,
            ];
            let mut random = vec![0; NATIVE_RAW_LIMIT];
            let mut state = 0x174ab28du32;
            for byte in &mut random {
                state ^= state << 13;
                state ^= state >> 17;
                state ^= state << 5;
                *byte = state as u8;
            }
            let repeated = vec![b'a'; NATIVE_RAW_LIMIT];
            let mut dictionary_magic = random.clone();
            dictionary_magic[..4].copy_from_slice(&[0x37, 0xa4, 0x30, 0xec]);
            let mut reused = super::super::NativeEncoder::new().unwrap();
            for source in [&random, &repeated, &dictionary_magic] {
                // ZSTD_window_update reduces an overlapping dictionary range
                // (compress_internal.h). Use disjoint operands as production does.
                let prefix_source = source.to_vec();
                for size in sizes {
                    for prefix_size in [0, 1, 7, 8, 63, 64, 1024, 32768] {
                        let raw = &source[..size];
                        let prefix = &prefix_source[..prefix_size];
                        let encoded = native_compress(raw, Some(prefix)).unwrap();
                        assert_eq!(reused.compress(raw, Some(prefix)).unwrap(), encoded);
                        assert!(encoded.capacity() <= NATIVE_FRAME_LIMIT);
                        assert_eq!(encoded, dynamic_frame(raw, prefix));
                        assert_eq!(
                            native_decompress(&encoded, size, Some(prefix)).unwrap(),
                            raw
                        );
                    }
                }
            }
            let valid_base = random.clone();
            let frame = native_compress(&random, Some(&valid_base)).unwrap();
            let full = native_compress(&random, None).unwrap();
            assert!(
                37 + frame.len() < 5 + full.len(),
                "fixture must select and use PREFIX"
            );
            assert_eq!(
                native_decompress(&frame, random.len(), Some(&valid_base)).unwrap(),
                random
            );
            let wrong = vec![0u8; NATIVE_RAW_LIMIT];
            assert!(native_decompress(&frame, random.len(), Some(&wrong)).is_err());
            assert!(native_decompress(&frame, random.len() - 1, Some(&valid_base)).is_err());
            assert!(
                native_decompress(&frame[..frame.len() - 1], random.len(), Some(&valid_base))
                    .is_err()
            );
            let mut extra = frame.clone();
            extra.push(0);
            assert!(native_decompress(&extra, random.len(), Some(&valid_base)).is_err());
            extra = frame.clone();
            extra.extend_from_slice(&frame);
            assert!(native_decompress(&extra, random.len(), Some(&valid_base)).is_err());
            for bad in [0x01, 0x02, 0x03, 0x08, 0x10] {
                let mut broken = frame.clone();
                broken[4] |= bad;
                assert!(native_decompress(&broken, random.len(), Some(&valid_base)).is_err());
            }
            let mut no_checksum = frame.clone();
            no_checksum[4] &= !4;
            assert!(native_decompress(&no_checksum, random.len(), Some(&valid_base)).is_err());
            let mut checksum = frame.clone();
            *checksum.last_mut().unwrap() ^= 1;
            assert!(native_decompress(&checksum, random.len(), Some(&valid_base)).is_err());
            let mut skippable = frame.clone();
            skippable[..4].copy_from_slice(&[0x50, 0x2a, 0x4d, 0x18]);
            assert!(native_decompress(&skippable, random.len(), Some(&valid_base)).is_err());
            // Explicit non-single-segment header declares windowLog21. The
            // oversized window is rejected before block/checksum decoding.
            let excessive_window = [
                0x28, 0xb5, 0x2f, 0xfd, 0x84, 0x58, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0,
            ];
            assert!(native_decompress(&excessive_window, 0, None).is_err());
            assert!(native_compress(&vec![0; NATIVE_RAW_LIMIT + 1], None).is_err());
            assert!(native_compress(&[], Some(&vec![0; NATIVE_RAW_LIMIT + 1])).is_err());
        }
    }
}

#[cfg(test)]
mod native_framing_tests {
    use super::*;

    #[test]
    fn native_grammar_ranges_and_legacy_dispatch() {
        // Fixed-header parsing does not claim frame authentication. This literal
        // nonempty frame lets framing checks run without invoking any codec.
        let full = native_encode_record(0, None, &[1]).unwrap();
        assert_eq!(full, [0, 0, 0, 0, 0, 1]);
        let base = ObjectId::for_bytes(b"base");
        let prefix = native_encode_record(NATIVE_RAW_LIMIT, Some(base), &[2]).unwrap();
        assert_eq!(&prefix[..5], &[1, 0, 128, 0, 0]);
        assert_eq!(&prefix[5..37], base.as_bytes());
        assert!(
            matches!(native_record(&prefix).unwrap(), NativeRecord::Prefix { raw_length: NATIVE_RAW_LIMIT, base: id, frame: [2] } if id == base)
        );
        let group = native_group(&[&full, &prefix]).unwrap();
        assert_eq!(
            native_record_range(2, &group.bytes[4..12], group.bytes.len(), 0).unwrap(),
            12..18
        );
        assert_eq!(
            native_record_range(2, &group.bytes[4..12], group.bytes.len(), 1).unwrap(),
            18..56
        );
        let packed = assemble_native(&[group]).unwrap();
        let multiple = assemble_native(&[
            native_group(&[&full]).unwrap(),
            native_group(&[&prefix]).unwrap(),
        ])
        .unwrap();
        let mh = versioned_header(multiple[..16].try_into().unwrap(), multiple.len()).unwrap();
        assert_eq!(mh.group_count, 2);
        let first =
            versioned_entry(multiple[16..32].try_into().unwrap(), mh, multiple.len()).unwrap();
        let second =
            versioned_entry(multiple[32..48].try_into().unwrap(), mh, multiple.len()).unwrap();
        assert_eq!(first.range.start, 48);
        assert_eq!(first.range.end, second.range.start);
        assert_eq!(second.range.end, multiple.len());
        let h: &[u8; 16] = packed[..16].try_into().unwrap();
        assert!(header(h, packed.len()).is_err());
        let parsed = versioned_header(h, packed.len()).unwrap();
        assert_eq!(parsed.version, Version::Native);
        let e: &[u8; 16] = packed[16..32].try_into().unwrap();
        assert_eq!(
            versioned_entry(e, parsed, packed.len()).unwrap().range,
            32..packed.len()
        );
        for offset in [12, 13, 14, 15] {
            let mut corrupt = *e;
            corrupt[offset] = 1;
            assert!(versioned_entry(&corrupt, parsed, packed.len()).is_err());
        }
        assert!(versioned_header(h, PACK_LIMIT + 1).is_err());
        assert!(native_record_range(0, &[], 4, 0).is_err());
        assert!(native_record_range(RECORD_COUNT_LIMIT + 1, &[], 4, 0).is_err());
        assert!(native_record_range(2, &[1, 0, 0, 0, 1, 0, 0, 0], 14, 0).is_err());
        assert!(native_record_range(2, &[1, 0, 0, 0, 3, 0, 0, 0], 14, 0).is_err());
        assert!(native_record_range(2, &[1, 0, 0, 0, 2, 0, 0, 0], 15, 0).is_err());
        assert!(native_record_range(2, &[1, 0, 0, 0, 2, 0, 0, 0], 14, 2).is_err());
        assert!(native_record(&[0, 0, 0, 0, 0]).is_err());
        assert!(native_record(&[1, 0, 0, 0, 0, 1]).is_err());
        assert!(native_record(&[2, 0, 0, 0, 0, 1]).is_err());
        assert!(native_encode_record(NATIVE_RAW_LIMIT + 1, None, &[1]).is_err());
        let maximal =
            native_encode_record(NATIVE_RAW_LIMIT, None, &vec![1; NATIVE_FRAME_LIMIT]).unwrap();
        assert!(native_group(&[&maximal]).is_ok());
        assert!(native_group(&[&maximal, &maximal]).is_err());
        let legacy = assemble(&[EncodedGroup {
            bytes: vec![1, 0, 0, 0, 2, 0, 0, 0, 0, 1],
            decoded_length: 10,
            records: 1,
            codec: Codec::Raw,
        }])
        .unwrap();
        assert_eq!(
            header(legacy[..16].try_into().unwrap(), legacy.len()).unwrap(),
            1
        );
        assert_eq!(
            versioned_header(legacy[..16].try_into().unwrap(), legacy.len())
                .unwrap()
                .version,
            Version::Legacy
        );
    }
}
