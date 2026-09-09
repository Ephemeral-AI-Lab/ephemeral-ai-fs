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
