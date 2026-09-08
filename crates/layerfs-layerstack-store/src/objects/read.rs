//! Connection-only extraction followed by bounded target and FULL-base waves.
use super::{pack, CanonicalObject, OBJECT_PAGE_COUNT};
use crate::schema::StoreDb;
use crate::{Result, StoreError};
use layerfs_content::ObjectId;
use rusqlite::{limits::Limit, params_from_iter, OptionalExtension};
use std::collections::BTreeMap;

// Leave the rest of the 2-MiB physical scratch reservation for group backing,
// parsed records, associations, one reconstructed object, and the codec context.
pub(super) const VALIDATION_RESERVE: usize = 1024 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Location {
    pub canonical_length: usize,
    pub pack: i64,
    pub group: usize,
    pub record: usize,
}

pub(super) fn validation_reserve(length: usize) -> usize {
    // Include request/locator/slot/map ownership, not just instruction bytes.
    // The other 1 MiB of physical scratch covers one encoded/decoded group,
    // a <=256-KiB decoder context, base/target buffers, and bounded wave overhead.
    512 + if length > pack::GROUP_LIMIT {
        0
    } else {
        (41 + 9 * length).min(pack::GROUP_LIMIT)
    }
}

impl StoreDb {
    /// This is the membership probe. Callers retain the returned locations for
    /// decoding, rather than issuing a second membership query during validation.
    pub(super) fn object_locations(
        &self,
        ids: &[ObjectId],
    ) -> Result<BTreeMap<ObjectId, Location>> {
        let mut found = BTreeMap::new();
        if ids.is_empty() {
            return Ok(found);
        }
        let connection = self.reader()?;
        if let [id] = ids {
            let row = connection
                .prepare_cached(crate::statements::objects::GET)?
                .query_row([id.as_bytes().as_slice()], |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, i64>(2)?,
                        row.get::<_, i64>(3)?,
                    ))
                })
                .optional()?;
            if let Some((length, pack, group, record)) = row {
                found.insert(*id, location(length, pack, group, record)?);
            }
            return Ok(found);
        }
        let count = usize::try_from(connection.limit(Limit::SQLITE_LIMIT_VARIABLE_NUMBER)?)
            .map_err(|_| StoreError::Integrity("SQLite parameter limit"))?
            .min(OBJECT_PAGE_COUNT);
        if count == 0 {
            return Err(StoreError::Integrity("SQLite parameter limit"));
        }
        for page in ids.chunks(count) {
            let sql = format!(
                "SELECT object_id,canonical_length,pack_id,group_number,record_number \
                 FROM objects WHERE object_id IN ({})",
                vec!["?"; page.len()].join(",")
            );
            let mut statement = connection.prepare_cached(&sql)?;
            for row in statement.query_map(
                params_from_iter(page.iter().map(|id| id.as_bytes().as_slice())),
                |row| {
                    Ok((
                        row.get::<_, Vec<u8>>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, i64>(2)?,
                        row.get::<_, i64>(3)?,
                        row.get::<_, i64>(4)?,
                    ))
                },
            )? {
                let (id, length, pack, group, record) = row?;
                found.insert(
                    ObjectId::from_bytes(&id)?,
                    location(length, pack, group, record)?,
                );
            }
        }
        Ok(found)
    }

    /// The guard and Blob never leave this extraction boundary.
    fn extract_group(&self, pack_id: i64, group: usize) -> Result<(pack::GroupEntry, Vec<u8>)> {
        let connection = self.reader()?;
        let blob = connection.blob_open("main", "object_packs", "data", pack_id, true)?;
        let length = blob.len();
        let mut header = [0; 16];
        blob.read_at_exact(&mut header, 0)?;
        let count = pack::header(&header, length)?;
        if group >= count {
            return Err(StoreError::Integrity("object group locator"));
        }
        let mut directory = [0; 16];
        blob.read_at_exact(&mut directory, 16 + 16 * group)?;
        let entry = pack::entry(&directory, count, length)?;
        if entry.oversized {
            return Err(StoreError::Integrity(
                "oversized object requires bounded extraction",
            ));
        }
        let mut encoded = vec![0; entry.range.len()];
        blob.read_at_exact(&mut encoded, entry.range.start)?;
        Ok((entry, encoded))
    }

    fn singleton_range(&self, location: Location) -> Result<std::ops::Range<usize>> {
        let connection = self.reader()?;
        let blob = connection.blob_open("main", "object_packs", "data", location.pack, true)?;
        let length = blob.len();
        let mut header = [0; 16];
        blob.read_at_exact(&mut header, 0)?;
        let count = pack::header(&header, length)?;
        if count != 1 || location.group != 0 || location.record != 0 {
            return Err(StoreError::Integrity("oversized object locator"));
        }
        let mut directory = [0; 16];
        blob.read_at_exact(&mut directory, 16)?;
        let entry = pack::entry(&directory, count, length)?;
        if !entry.oversized
            || entry.codec != pack::Codec::Raw
            || location.canonical_length <= pack::GROUP_LIMIT - 9
            || length != location.canonical_length + 41
        {
            return Err(StoreError::Integrity("oversized object framing"));
        }
        let mut framing = [0; 9];
        blob.read_at_exact(&mut framing, 32)?;
        if framing[..4] != 1u32.to_le_bytes()
            || framing[4..8] != ((location.canonical_length + 1) as u32).to_le_bytes()
            || framing[8] != 0
        {
            return Err(StoreError::Integrity("oversized object record"));
        }
        Ok(41..length)
    }

    fn singleton(&self, id: ObjectId, location: Location) -> Result<Vec<u8>> {
        let range = self.singleton_range(location)?;
        // This allocation is the returned canonical object, not extraction scratch.
        let mut output = vec![0; range.len()];
        for (index, part) in output.chunks_mut(pack::GROUP_LIMIT).enumerate() {
            let connection = self.reader()?;
            let blob = connection.blob_open("main", "object_packs", "data", location.pack, true)?;
            blob.read_at_exact(part, range.start + index * pack::GROUP_LIMIT)?;
        }
        authenticate(id, &output, location.canonical_length)?;
        Ok(output)
    }

    pub(super) fn compare_singleton(
        &self,
        id: ObjectId,
        location: Location,
        canonical: &[u8],
    ) -> Result<()> {
        let range = self.singleton_range(location)?;
        if range.len() != canonical.len() {
            return Err(StoreError::Integrity("object length collision"));
        }
        // Reuse the canonical reader's domain-separated hash. Each Read releases
        // SQLite before comparison and before the hasher receives the bytes.
        let reader = SingletonComparison {
            db: self,
            pack: location.pack,
            canonical,
            cursor: 0,
        };
        if ObjectId::from_reader(reader)? != id {
            return Err(StoreError::Integrity("object identity"));
        }
        super::note_read_batch_hash();
        Ok(())
    }

    /// Plan slots once, then drain forward. Repeated groups across these bounded
    /// drains count again; results returned by the caller have separate ownership.
    pub(super) fn visit_locations(
        &self,
        locations: &[(ObjectId, Location)],
        mut emit: impl FnMut(CanonicalObject) -> Result<()>,
    ) -> Result<()> {
        let mut start = 0;
        while start < locations.len() {
            let mut end = start;
            let mut reserve = 0;
            while end < locations.len() && end - start < OBJECT_PAGE_COUNT {
                let next = validation_reserve(locations[end].1.canonical_length);
                if reserve + next > VALIDATION_RESERVE {
                    break;
                }
                reserve += next;
                end += 1;
            }
            if end == start {
                return Err(StoreError::Integrity("packed read scratch limit"));
            }
            self.visit_wave(&locations[start..end], &mut emit)?;
            start = end;
        }
        Ok(())
    }

    fn visit_wave(
        &self,
        locations: &[(ObjectId, Location)],
        emit: &mut impl FnMut(CanonicalObject) -> Result<()>,
    ) -> Result<()> {
        let groups = group_locations(locations);
        let mut pending = BTreeMap::<ObjectId, Vec<PendingDelta>>::new();
        for ((pack_id, group), targets) in groups {
            // Lengths 65528..65536 may be stored DELTA. The selected header,
            // not the caller's proposed representation, decides the route.
            if targets
                .iter()
                .any(|(_, location)| location.canonical_length > pack::GROUP_LIMIT - 9)
            {
                if let [(id, location)] = targets.as_slice() {
                    if self.is_singleton(*location)? {
                        emit(CanonicalObject {
                            id: *id,
                            bytes: self.singleton(*id, *location)?,
                        })?;
                        continue;
                    }
                }
            }
            let (entry, encoded) = self.extract_group(pack_id, group)?;
            let decoded = decode(entry, encoded)?;
            let mut requested = record_slots(targets)?;
            pack::visit_records(&decoded, false, |index, record| {
                let Some((id, location)) = requested.remove(&index) else {
                    return Ok(());
                };
                match record {
                    pack::Record::Full(bytes) => {
                        authenticate(id, bytes, location.canonical_length)?;
                        emit(CanonicalObject {
                            id,
                            bytes: bytes.to_vec(),
                        })?;
                    }
                    pack::Record::Delta {
                        base,
                        output_length,
                        instructions,
                        count,
                    } => {
                        if base == id || output_length != location.canonical_length {
                            return Err(StoreError::Integrity("delta target"));
                        }
                        pending.entry(base).or_default().push(PendingDelta {
                            id,
                            output_length,
                            count,
                            instructions: instructions.to_vec(),
                        });
                    }
                }
                Ok(())
            })?;
            if !requested.is_empty() {
                return Err(StoreError::Integrity("object record locator"));
            }
        }
        let ids = pending.keys().copied().collect::<Vec<_>>();
        let bases = self.object_locations(&ids)?;
        if bases.len() != ids.len() {
            return Err(StoreError::Integrity("delta base missing"));
        }
        let bases = bases.into_iter().collect::<Vec<_>>();
        for ((pack_id, group), targets) in group_locations(&bases) {
            if targets
                .iter()
                .any(|(_, location)| location.canonical_length > pack::GROUP_LIMIT)
            {
                return Err(StoreError::Integrity("delta base length"));
            }
            if let [(id, location)] = targets.as_slice() {
                if location.canonical_length > pack::GROUP_LIMIT - 9
                    && self.is_singleton(*location)?
                {
                    let bytes = self.singleton(*id, *location)?;
                    finish_deltas(
                        pending
                            .remove(id)
                            .ok_or(StoreError::Integrity("delta base association"))?,
                        &bytes,
                        emit,
                    )?;
                    continue;
                }
            }
            let (entry, encoded) = self.extract_group(pack_id, group)?;
            let decoded = decode(entry, encoded)?;
            let mut requested = record_slots(targets)?;
            pack::visit_records(&decoded, false, |index, record| {
                let Some((id, location)) = requested.remove(&index) else {
                    return Ok(());
                };
                let pack::Record::Full(bytes) = record else {
                    return Err(StoreError::Integrity("delta base is not FULL"));
                };
                authenticate(id, bytes, location.canonical_length)?;
                finish_deltas(
                    pending
                        .remove(&id)
                        .ok_or(StoreError::Integrity("delta base association"))?,
                    bytes,
                    emit,
                )
            })?;
            if !requested.is_empty() {
                return Err(StoreError::Integrity("base record locator"));
            }
        }
        if !pending.is_empty() {
            return Err(StoreError::Integrity("delta base coverage"));
        }
        Ok(())
    }

    fn is_singleton(&self, location: Location) -> Result<bool> {
        let connection = self.reader()?;
        let blob = connection.blob_open("main", "object_packs", "data", location.pack, true)?;
        let mut header = [0; 16];
        blob.read_at_exact(&mut header, 0)?;
        let count = pack::header(&header, blob.len())?;
        if location.group >= count {
            return Err(StoreError::Integrity("object group locator"));
        }
        let mut directory = [0; 16];
        blob.read_at_exact(&mut directory, 16 + 16 * location.group)?;
        Ok(pack::entry(&directory, count, blob.len())?.oversized)
    }
}

fn location(length: i64, pack: i64, group: i64, record: i64) -> Result<Location> {
    if !(1..=pack::CANONICAL_LIMIT as i64).contains(&length)
        || pack <= 0
        || !(0..pack::GROUP_COUNT_LIMIT as i64).contains(&group)
        || !(0..pack::RECORD_COUNT_LIMIT as i64).contains(&record)
    {
        return Err(StoreError::Integrity("object locator"));
    }
    Ok(Location {
        canonical_length: length as usize,
        pack,
        group: group as usize,
        record: record as usize,
    })
}

struct SingletonComparison<'a> {
    db: &'a StoreDb,
    pack: i64,
    canonical: &'a [u8],
    cursor: usize,
}

impl std::io::Read for SingletonComparison<'_> {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        let length = buffer
            .len()
            .min(self.canonical.len() - self.cursor)
            .min(pack::GROUP_LIMIT);
        if length == 0 {
            return Ok(0);
        }
        let result = (|| -> Result<()> {
            let connection = self.db.reader()?;
            let blob = connection.blob_open("main", "object_packs", "data", self.pack, true)?;
            blob.read_at_exact(&mut buffer[..length], 41 + self.cursor)?;
            Ok(())
        })();
        result.map_err(std::io::Error::other)?;
        if buffer[..length] != self.canonical[self.cursor..self.cursor + length] {
            return Err(std::io::Error::other(StoreError::Integrity(
                "object collision",
            )));
        }
        self.cursor += length;
        Ok(length)
    }
}

fn record_slots(
    targets: Vec<(ObjectId, Location)>,
) -> Result<BTreeMap<usize, (ObjectId, Location)>> {
    let mut slots = BTreeMap::new();
    for (id, location) in targets {
        if slots.insert(location.record, (id, location)).is_some() {
            return Err(StoreError::Integrity(
                "distinct object IDs alias one record",
            ));
        }
    }
    Ok(slots)
}

fn group_locations(
    locations: &[(ObjectId, Location)],
) -> BTreeMap<(i64, usize), Vec<(ObjectId, Location)>> {
    let mut groups = BTreeMap::<_, Vec<_>>::new();
    for (id, location) in locations {
        groups
            .entry((location.pack, location.group))
            .or_default()
            .push((*id, *location));
    }
    groups
}

fn authenticate(id: ObjectId, bytes: &[u8], length: usize) -> Result<()> {
    if bytes.len() != length {
        return Err(StoreError::Integrity("object canonical length"));
    }
    layerfs_content::authenticate_identity(bytes, id)?;
    super::note_read_batch_hash();
    Ok(())
}

struct PendingDelta {
    id: ObjectId,
    output_length: usize,
    count: usize,
    instructions: Vec<u8>,
}

fn finish_deltas(
    pending: Vec<PendingDelta>,
    base: &[u8],
    emit: &mut impl FnMut(CanonicalObject) -> Result<()>,
) -> Result<()> {
    for target in pending {
        let bytes = pack::apply_delta(
            &target.instructions,
            target.count,
            target.output_length,
            base,
        )?;
        authenticate(target.id, &bytes, target.output_length)?;
        emit(CanonicalObject {
            id: target.id,
            bytes,
        })?;
    }
    Ok(())
}

fn decode(entry: pack::GroupEntry, encoded: Vec<u8>) -> Result<Vec<u8>> {
    match entry.codec {
        pack::Codec::Raw if encoded.len() == entry.decoded_length => Ok(encoded),
        pack::Codec::Zstandard => Err(StoreError::Core(layerfs_content::CoreError::Unsupported)),
        _ => Err(StoreError::Integrity("group decoded length")),
    }
}
