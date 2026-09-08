//! Connection-only extraction followed by bounded target and FULL-base waves.
use super::{pack, CanonicalObject, OBJECT_PAGE_COUNT};
use crate::schema::StoreDb;
use crate::{PhysicalStorageReceipt, Result, StoreError};
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

/// Optional predecessor discovery has independent per-target and batch bounds.
#[derive(Default)]
pub(super) struct HintReadBudget {
    target_fetches: usize,
    target_encoded: usize,
    target_decoded: usize,
    batch_encoded: usize,
    batch_decoded: usize,
    pub exhausted: bool,
}

impl HintReadBudget {
    pub fn begin_target(&mut self) {
        self.target_fetches = 0;
        self.target_encoded = 0;
        self.target_decoded = 0;
        self.exhausted = false;
    }

    fn charge(&mut self, encoded: usize, decoded: usize) -> bool {
        if self.target_encoded + encoded > 512 * 1024
            || self.target_decoded + decoded > 512 * 1024
            || self.batch_encoded + encoded > 8 * 1024 * 1024
            || self.batch_decoded + decoded > 8 * 1024 * 1024
        {
            self.exhausted = true;
            return false;
        }
        self.target_encoded += encoded;
        self.target_decoded += decoded;
        self.batch_encoded += encoded;
        self.batch_decoded += decoded;
        true
    }
}

pub(super) enum HintRecord {
    Full(CanonicalObject),
    Anchor(ObjectId),
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
    /// Logical closure needs presence, not a second set of allocated locators.
    /// Callers pass distinct IDs; authentication remains on every demanded read.
    pub(super) fn objects_exist(&self, ids: &[ObjectId]) -> Result<bool> {
        if ids.is_empty() {
            return Ok(true);
        }
        let connection = self.reader()?;
        let count = usize::try_from(connection.limit(Limit::SQLITE_LIMIT_VARIABLE_NUMBER)?)
            .map_err(|_| StoreError::Integrity("SQLite parameter limit"))?
            .min(OBJECT_PAGE_COUNT);
        if count == 0 {
            return Err(StoreError::Integrity("SQLite parameter limit"));
        }
        for page in ids.chunks(count) {
            let sql = format!(
                "SELECT count(*) FROM objects WHERE object_id IN ({})",
                vec!["?"; page.len()].join(",")
            );
            let found: i64 = connection.prepare_cached(&sql)?.query_row(
                params_from_iter(page.iter().map(|id| id.as_bytes().as_slice())),
                |row| row.get(0),
            )?;
            if found != page.len() as i64 {
                return Ok(false);
            }
        }
        Ok(true)
    }

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
        self.extract_hint_group(pack_id, group, None)?
            .ok_or(StoreError::Integrity("required group extraction"))
    }

    fn extract_hint_group(
        &self,
        pack_id: i64,
        group: usize,
        budget: Option<&mut HintReadBudget>,
    ) -> Result<Option<(pack::GroupEntry, Vec<u8>)>> {
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
        self.note_physical(PhysicalStorageReceipt {
            group_fetches: 1,
            blob_ranges: 2,
            ..Default::default()
        });
        if let Some(budget) = budget {
            if !budget.charge(entry.range.len(), entry.decoded_length) {
                return Ok(None);
            }
        }
        if entry.oversized {
            // Carry checked directory facts into bounded singleton extraction.
            return Ok(Some((entry, Vec::new())));
        }
        let mut encoded = vec![0; entry.range.len()];
        blob.read_at_exact(&mut encoded, entry.range.start)?;
        self.note_physical(PhysicalStorageReceipt {
            encoded_read_bytes: encoded.len() as u64,
            blob_ranges: 1,
            ..Default::default()
        });
        Ok(Some((entry, encoded)))
    }

    /// Inspect one selected predecessor representation. DELTA hints expose their
    /// FULL anchor without reconstructing an otherwise unused target. The anchor
    /// must be fetched with `require_full` and authenticated before matching.
    pub(super) fn read_hint(
        &self,
        id: ObjectId,
        require_full: bool,
        budget: &mut HintReadBudget,
    ) -> Result<Option<HintRecord>> {
        if budget.exhausted || budget.target_fetches == 8 {
            budget.exhausted = true;
            return Ok(None);
        }
        budget.target_fetches += 1;
        self.note_physical(PhysicalStorageReceipt {
            base_fetches: 1,
            ..Default::default()
        });
        let Some(location) = self.object_locations(&[id])?.remove(&id) else {
            return Ok(None);
        };
        if location.canonical_length > pack::GROUP_LIMIT {
            return Ok(None);
        }
        if !budget.charge(32, 0) {
            return Ok(None);
        }
        let Some((entry, encoded)) =
            self.extract_hint_group(location.pack, location.group, Some(budget))?
        else {
            return Ok(None);
        };
        if entry.oversized {
            return Ok(Some(HintRecord::Full(CanonicalObject {
                id,
                bytes: self.singleton(id, location, &entry)?,
            })));
        }
        self.note_physical(PhysicalStorageReceipt {
            decoded_read_bytes: entry.decoded_length as u64,
            decompression_calls: u64::from(entry.codec == pack::Codec::Zstandard),
            ..Default::default()
        });
        let decoded = pack::decode_group(entry, encoded)?;
        let mut selected = None;
        pack::visit_records(&decoded, false, |index, record| {
            if index != location.record {
                return Ok(());
            }
            selected = Some(match record {
                pack::Record::Full(bytes) => {
                    authenticate(id, bytes, location.canonical_length)?;
                    HintRecord::Full(CanonicalObject {
                        id,
                        bytes: bytes.to_vec(),
                    })
                }
                pack::Record::Delta {
                    base,
                    output_length,
                    ..
                } => {
                    if require_full || base == id || output_length != location.canonical_length {
                        return Err(StoreError::Integrity("delta hint base"));
                    }
                    HintRecord::Anchor(base)
                }
            });
            Ok(())
        })?;
        selected
            .map(Some)
            .ok_or(StoreError::Integrity("hint record locator"))
    }

    fn singleton_range(
        &self,
        location: Location,
        entry: &pack::GroupEntry,
    ) -> Result<std::ops::Range<usize>> {
        let connection = self.reader()?;
        let blob = connection.blob_open("main", "object_packs", "data", location.pack, true)?;
        let length = blob.len();
        if location.group != 0 || location.record != 0 || length != entry.range.end {
            return Err(StoreError::Integrity("oversized object locator"));
        }
        if !entry.oversized
            || entry.codec != pack::Codec::Raw
            || location.canonical_length <= pack::GROUP_LIMIT - 9
            || length != location.canonical_length + 41
        {
            return Err(StoreError::Integrity("oversized object framing"));
        }
        let mut framing = [0; 9];
        blob.read_at_exact(&mut framing, 32)?;
        self.note_physical(PhysicalStorageReceipt {
            encoded_read_bytes: 9,
            decoded_read_bytes: 9,
            blob_ranges: 1,
            ..Default::default()
        });
        if framing[..4] != 1u32.to_le_bytes()
            || framing[4..8] != ((location.canonical_length + 1) as u32).to_le_bytes()
            || framing[8] != 0
        {
            return Err(StoreError::Integrity("oversized object record"));
        }
        Ok(41..length)
    }

    fn singleton(
        &self,
        id: ObjectId,
        location: Location,
        entry: &pack::GroupEntry,
    ) -> Result<Vec<u8>> {
        let range = self.singleton_range(location, entry)?;
        // This allocation is the returned canonical object, not extraction scratch.
        let mut output = vec![0; range.len()];
        for (index, part) in output.chunks_mut(pack::GROUP_LIMIT).enumerate() {
            let connection = self.reader()?;
            let blob = connection.blob_open("main", "object_packs", "data", location.pack, true)?;
            blob.read_at_exact(part, range.start + index * pack::GROUP_LIMIT)?;
            self.note_physical(PhysicalStorageReceipt {
                encoded_read_bytes: part.len() as u64,
                decoded_read_bytes: part.len() as u64,
                blob_ranges: 1,
                ..Default::default()
            });
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
        let (entry, _) = self.extract_group(location.pack, location.group)?;
        let range = self.singleton_range(location, &entry)?;
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

    /// Sort physical locations before draining so a group is not scattered across
    /// internal batches by ObjectId order. Callers restore public slots/duplicates;
    /// sorting in place adds no locator allocation. Repeated groups across bounded
    /// drains still count again, and returned results have separate ownership.
    pub(super) fn visit_locations(
        &self,
        locations: &mut [(ObjectId, Location)],
        mut emit: impl FnMut(CanonicalObject) -> Result<()>,
    ) -> Result<()> {
        locations
            .sort_unstable_by_key(|(_, location)| (location.pack, location.group, location.record));
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
            // The selected entry decides the 65528..65536 RAW/DELTA overlap.
            // Do not fetch its directory a second time just to choose the route.
            let (entry, encoded) = self.extract_group(pack_id, group)?;
            if entry.oversized {
                let [(id, location)] = targets.as_slice() else {
                    return Err(StoreError::Integrity("singleton locator alias"));
                };
                emit(CanonicalObject {
                    id: *id,
                    bytes: self.singleton(*id, *location, &entry)?,
                })?;
                continue;
            }
            self.note_physical(PhysicalStorageReceipt {
                decoded_read_bytes: entry.decoded_length as u64,
                decompression_calls: u64::from(entry.codec == pack::Codec::Zstandard),
                ..Default::default()
            });
            let decoded = pack::decode_group(entry, encoded)?;
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
        self.note_physical(PhysicalStorageReceipt {
            base_fetches: ids.len() as u64,
            ..Default::default()
        });
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
            let (entry, encoded) = self.extract_group(pack_id, group)?;
            if entry.oversized {
                let [(id, location)] = targets.as_slice() else {
                    return Err(StoreError::Integrity("singleton base alias"));
                };
                let bytes = self.singleton(*id, *location, &entry)?;
                finish_deltas(
                    pending
                        .remove(id)
                        .ok_or(StoreError::Integrity("delta base association"))?,
                    &bytes,
                    emit,
                )?;
                continue;
            }
            self.note_physical(PhysicalStorageReceipt {
                decoded_read_bytes: entry.decoded_length as u64,
                decompression_calls: u64::from(entry.codec == pack::Codec::Zstandard),
                ..Default::default()
            });
            let decoded = pack::decode_group(entry, encoded)?;
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
            self.db.note_physical(PhysicalStorageReceipt {
                encoded_read_bytes: length as u64,
                decoded_read_bytes: length as u64,
                blob_ranges: 1,
                ..Default::default()
            });
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
