//! Connection-only extraction followed by bounded target and FULL-base waves.
use super::{pack, CanonicalObject, OBJECT_PAGE_COUNT};
use crate::schema::StoreDb;
use crate::{PhysicalStorageReceipt, Result, StoreError};
use layerfs_content::{file::content, ObjectId};
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
    target_pool_decoded: usize,
    batch_pool_decoded: usize,
    target_encoded: usize,
    target_decoded: usize,
    batch_encoded: usize,
    batch_decoded: usize,
    batch_exhausted: bool,
    pub exhausted: bool,
}

impl HintReadBudget {
    pub fn begin_target(&mut self) {
        self.target_fetches = 0;
        self.target_pool_decoded = 0;
        self.target_encoded = 0;
        self.target_decoded = 0;
        // A later target may reset its own allowance, never restart a batch
        // whose optional discovery has already exhausted its work budget.
        self.exhausted = self.batch_exhausted;
    }

    // Metadata groups have a separate physical work budget: their authenticated
    // value expansion is not covered by the canonical chain's 128-KiB limit.
    pub(super) fn charge_metadata_pool(&mut self, decoded: usize) -> bool {
        self.batch_exhausted |= self.batch_pool_decoded + decoded > 64 * 1024 * 1024;
        if self.batch_exhausted || self.target_pool_decoded + decoded > 8 * 1024 * 1024 {
            self.exhausted = true;
            return false;
        }
        self.target_pool_decoded += decoded;
        self.batch_pool_decoded += decoded;
        true
    }

    pub(super) fn charge(&mut self, encoded: usize, decoded: usize) -> bool {
        self.batch_exhausted |= self.batch_encoded + encoded > 8 * 1024 * 1024
            || self.batch_decoded + decoded > 8 * 1024 * 1024;
        if self.batch_exhausted
            || self.target_encoded + encoded > 512 * 1024
            || self.target_decoded + decoded > 512 * 1024
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

/// A missing optional hint differs from corruption in an admitted dependency.
pub(super) enum NativePriorOutcome {
    Available {
        canonical: CanonicalObject,
        location: Location,
        depth: u8,
        raw_closure: usize,
    },
    Unavailable,
    UnsupportedLegacyDelta,
    UnsupportedRole,
    Budget,
}

enum Extraction {
    Whole(Vec<u8>),
    Metadata(bool, pack::GroupEntry, Vec<u8>),
    Small(Vec<u8>),
    Legacy(pack::GroupEntry, Vec<u8>),
    Native {
        record: Vec<u8>,
        requested: usize,
        parsed: usize,
    },
    NativeUnsupported,
}

pub(super) struct SmallPredecessor {
    pub canonical: CanonicalObject,
    pub location: Location,
    pub depth: usize,
    pub canonical_closure: usize,
    pub encoded_closure: usize,
}

struct NativeNode {
    id: ObjectId,
    location: Location,
    record: Vec<u8>,
}

fn chunk_payload(canonical: &[u8]) -> Result<&[u8]> {
    Ok(layerfs_content::file::extent_codec::decode_chunk_payload(
        layerfs_content::decode_bytes_object(canonical)?,
    )?)
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

pub(super) struct MetadataPredecessor {
    pub canonical: CanonicalObject,
    pub physical: Vec<u8>,
    pub pooled: bool,
    pub depth: usize,
    pub canonical_closure: usize,
}

pub(super) const METADATA_EDGES: usize = 16;
pub(super) const METADATA_CLOSURE: usize = 128 * 1024;

pub(super) fn metadata_leaf(bytes: &[u8]) -> bool {
    bytes.get(13..21) == Some(b"LFS6INT\0") && bytes.get(23) == Some(&7)
}

// Keep the selected program only, after validating the complete group directory.
fn metadata_record(decoded: &[u8], ordinal: usize) -> Result<Vec<u8>> {
    let mut selected = None;
    pack::visit_records(decoded, false, |index, record| {
        if index != ordinal {
            return Ok(());
        }
        let mut bytes = Vec::new();
        match record {
            pack::Record::Full(canonical) => {
                bytes.push(0);
                bytes.extend_from_slice(canonical);
            }
            pack::Record::Delta {
                base,
                output_length,
                instructions,
                count,
            } => {
                bytes.push(1);
                bytes.extend_from_slice(base.as_bytes());
                bytes.extend_from_slice(&(output_length as u32).to_le_bytes());
                bytes.extend_from_slice(&(count as u32).to_le_bytes());
                bytes.extend_from_slice(instructions);
            }
        }
        if bytes.len() > 8193 {
            return Err(StoreError::Integrity("metadata record bound"));
        }
        selected = Some(bytes);
        Ok(())
    })?;
    selected.ok_or(StoreError::Integrity("metadata record locator"))
}

impl StoreDb {
    pub(super) fn read_metadata_values(
        &self,
        group: super::metadata::Group,
    ) -> Result<Vec<[u8; 73]>> {
        let Some(Extraction::Metadata(true, entry, encoded)) =
            self.extract_record_group(group.pack, group.number, 0, 0, true, None)?
        else {
            return Err(StoreError::Integrity("metadata value group format"));
        };
        let body = self.decode_metadata_group(entry, encoded)?;
        self.note_physical(PhysicalStorageReceipt {
            metadata_pool_group_fetches: 1,
            metadata_pool_decoded_bytes: body.len() as u64,
            ..Default::default()
        });
        super::metadata::decode_values(&body, group)
    }

    fn decode_metadata_group(&self, entry: pack::GroupEntry, encoded: Vec<u8>) -> Result<Vec<u8>> {
        if !self.compact_namespace() || entry.oversized || entry.decoded_length > 16 * 1024 {
            return Err(StoreError::Integrity("metadata group bound/format"));
        }
        self.note_physical(PhysicalStorageReceipt {
            decoded_read_bytes: entry.decoded_length as u64,
            decompression_calls: u64::from(entry.codec == pack::Codec::Zstandard),
            ..Default::default()
        });
        pack::decode_group(entry, encoded)
    }

    pub(super) fn metadata_predecessor(
        &self,
        id: ObjectId,
        budget: &mut HintReadBudget,
    ) -> Result<Option<MetadataPredecessor>> {
        let Some(location) = self.object_locations(&[id])?.remove(&id) else {
            return Ok(None);
        };
        let Some(Extraction::Metadata(pooled, entry, encoded)) = self.extract_record_group(
            location.pack,
            location.group,
            location.record,
            location.canonical_length,
            true,
            Some(budget),
        )?
        else {
            return Ok(None);
        };
        let decoded = self.decode_metadata_group(entry, encoded)?;
        let record = metadata_record(&decoded, location.record)?;
        drop(decoded);
        let mut pool = super::metadata::PoolRead::default();
        self.metadata_chain(id, location, pooled, record, Some(budget), &mut pool)
    }

    // Every intermediate canonical object is authenticated, including bytes that
    // a later COPY/INSERT does not use. Chronology makes cycles impossible; the
    // independent work bounds are checked before fetching another dependency.
    fn metadata_chain(
        &self,
        mut id: ObjectId,
        mut location: Location,
        pooled: bool,
        mut record: Vec<u8>,
        mut budget: Option<&mut HintReadBudget>,
        // One bounded pool reader per demand wave. A point read creates its own,
        // which is exactly the previous per-object behaviour.
        pool: &mut super::metadata::PoolRead,
    ) -> Result<Option<MetadataPredecessor>> {
        let target_id = id;
        let mut nodes = Vec::with_capacity(METADATA_EDGES);
        let mut canonical_closure = 0;
        let mut encoded_closure = 0;
        let mut bytes;
        loop {
            canonical_closure += location.canonical_length;
            encoded_closure += record.len();
            if location.canonical_length > 8192
                || canonical_closure > METADATA_CLOSURE
                || encoded_closure > 17 * 8193
                || record.len() > 8193
            {
                return Err(StoreError::Integrity("metadata chain work bound"));
            }
            match pack::record(&record)? {
                pack::Record::Full(canonical) => {
                    bytes = canonical.to_vec();
                    break;
                }
                pack::Record::Delta {
                    base,
                    output_length,
                    ..
                } => {
                    if nodes.len() == METADATA_EDGES
                        || output_length
                            != if pooled {
                                super::metadata::physical_length(location.canonical_length)?
                            } else {
                                location.canonical_length
                            }
                    {
                        return Err(StoreError::Integrity("metadata chain depth/length"));
                    }
                    let next = self
                        .object_locations(&[base])?
                        .remove(&base)
                        .ok_or(StoreError::Integrity("metadata base missing"))?;
                    if next.pack >= location.pack
                        || next.canonical_length > 8192
                        || canonical_closure + next.canonical_length > METADATA_CLOSURE
                    {
                        return Err(StoreError::Integrity("metadata base chronology/closure"));
                    }
                    self.note_physical(PhysicalStorageReceipt {
                        base_fetches: 1,
                        ..Default::default()
                    });
                    let extracted = self.extract_record_group(
                        next.pack,
                        next.group,
                        next.record,
                        next.canonical_length,
                        true,
                        budget.as_deref_mut(),
                    )?;
                    let Some(extracted) = extracted else {
                        return Ok(None);
                    };
                    let Extraction::Metadata(base_pooled, entry, encoded) = extracted else {
                        return Err(StoreError::Integrity("metadata base format"));
                    };
                    if base_pooled != pooled {
                        return Err(StoreError::Integrity("metadata base pool format"));
                    }
                    let decoded = self.decode_metadata_group(entry, encoded)?;
                    let next_record = metadata_record(&decoded, next.record)?;
                    nodes.push((id, location.canonical_length, record));
                    id = base;
                    location = next;
                    record = next_record;
                }
            }
        }
        let mut canonical = if pooled {
            let Some(canonical) = pool.expand(
                self,
                &bytes,
                location.canonical_length,
                budget.as_deref_mut(),
            )?
            else {
                return Ok(None);
            };
            canonical
        } else {
            bytes.clone()
        };
        authenticate_metadata(id, &canonical, location.canonical_length)?;
        let depth = nodes.len();
        while let Some((id, length, program)) = nodes.pop() {
            let pack::Record::Delta {
                instructions,
                count,
                output_length,
                ..
            } = pack::record(&program)?
            else {
                return Err(StoreError::Integrity("metadata chain program"));
            };
            bytes = pack::apply_delta(instructions, count, output_length, &bytes)?;
            canonical = if pooled {
                let Some(canonical) = pool.expand(self, &bytes, length, budget.as_deref_mut())?
                else {
                    return Ok(None);
                };
                canonical
            } else {
                bytes.clone()
            };
            authenticate_metadata(id, &canonical, length)?;
        }
        Ok(Some(MetadataPredecessor {
            canonical: CanonicalObject {
                id: target_id,
                bytes: canonical,
            },
            physical: bytes,
            pooled,
            depth,
            canonical_closure,
        }))
    }

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

    /// Complete pack grammar verification belongs to explicit integrity/accounting walks.
    pub(crate) fn validate_small_packs(&self) -> Result<()> {
        let mut after = 0i64;
        loop {
            let next = {
                let connection = self.reader()?;
                connection.query_row("SELECT pack_id, length(data) FROM object_packs WHERE pack_id > ?1 AND substr(data,9,4) IN (x'03000000', x'04000000') ORDER BY pack_id LIMIT 1", [after], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))).optional()?
            };
            let Some((id, length)) = next else {
                return Ok(());
            };
            let length = usize::try_from(length)
                .map_err(|_| StoreError::Integrity("SmallContent pack length"))?;
            if !self.small_content_format() || length > pack::PACK_LIMIT {
                return Err(StoreError::Integrity("SmallContent pack bound/format"));
            }
            let bytes = {
                let connection = self.reader()?;
                let blob = connection.blob_open("main", "object_packs", "data", id, true)?;
                if blob.len() != length {
                    return Err(StoreError::Integrity(
                        "pack changed during integrity traversal",
                    ));
                }
                let mut bytes = vec![0; length];
                blob.read_at_exact(&mut bytes, 0)?;
                bytes
            };
            if bytes.get(8..12) == Some(&[4, 0, 0, 0]) && !self.compact_framing() {
                return Err(StoreError::Integrity("compact framing requires schema 10"));
            }
            pack::validate_small_pack(&bytes)?;
            if !self.small_chain_format() {
                let count = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
                for directory in bytes[16..16 + 16 * count].chunks_exact(16) {
                    let start = u32::from_le_bytes(directory[..4].try_into().unwrap()) as usize;
                    if bytes[start] == 2 {
                        return Err(StoreError::Integrity(
                            "SmallContent chain requires schema 9",
                        ));
                    }
                }
            }
            after = id;
        }
    }

    pub(crate) fn small_physical_base(&self, id: ObjectId) -> Result<Option<ObjectId>> {
        let location = self
            .object_locations(&[id])?
            .remove(&id)
            .ok_or(StoreError::MissingObject(id))?;
        match self.extract_record_group(
            location.pack,
            location.group,
            location.record,
            location.canonical_length,
            true,
            None,
        )? {
            Some(Extraction::Whole(bytes)) => {
                Ok(super::whole::record(&bytes, location.canonical_length)?.base)
            }
            Some(Extraction::Small(bytes)) => Ok(super::delta::record(&bytes)?.base),
            Some(Extraction::Metadata(_, entry, encoded)) => {
                let decoded = self.decode_metadata_group(entry, encoded)?;
                let record = metadata_record(&decoded, location.record)?;
                Ok(match pack::record(&record)? {
                    pack::Record::Full(_) => None,
                    pack::Record::Delta { base, .. } => Some(base),
                })
            }
            _ => Ok(None),
        }
    }

    fn read_small_full(
        &self,
        id: ObjectId,
        location: Location,
        record: Vec<u8>,
    ) -> Result<CanonicalObject> {
        let parsed = super::delta::record(&record)?;
        if parsed.base.is_some()
            || location.record != 0
            || location.canonical_length != parsed.raw_length + 23
        {
            return Err(StoreError::Integrity("SmallContent FULL base role/length"));
        }
        let bytes = super::delta::decode(&record, None)?;
        authenticate(id, &bytes, location.canonical_length)?;
        self.note_physical(PhysicalStorageReceipt {
            decompression_calls: 1,
            decoded_read_bytes: bytes.len() as u64,
            ..Default::default()
        });
        Ok(CanonicalObject { id, bytes })
    }

    // The selected record establishes encoding; a DELTA's named base is required.
    pub(super) fn small_anchor(
        &self,
        predecessor: ObjectId,
        location: Option<Location>,
    ) -> Result<Option<(CanonicalObject, Location)>> {
        let Some(location) = location else {
            return Ok(None);
        };
        let Some(Extraction::Small(record)) = self.extract_record_group(
            location.pack,
            location.group,
            location.record,
            location.canonical_length,
            true,
            None,
        )?
        else {
            return Ok(None);
        };
        let parsed = super::delta::record(&record)?;
        if location.canonical_length != parsed.raw_length + 23 {
            return Err(StoreError::Integrity("SmallContent predecessor length"));
        }
        if let Some(base) = parsed.base {
            if base == predecessor {
                return Err(StoreError::Integrity("SmallContent dependency cycle"));
            }
            let base_location = self
                .object_locations(&[base])?
                .remove(&base)
                .ok_or(StoreError::Integrity("SmallContent base missing"))?;
            if base_location.pack > location.pack {
                return Err(StoreError::Integrity("SmallContent base chronology"));
            }
            let Some(Extraction::Small(record)) = self.extract_record_group(
                base_location.pack,
                base_location.group,
                base_location.record,
                base_location.canonical_length,
                true,
                None,
            )?
            else {
                return Err(StoreError::Integrity("SmallContent base encoding"));
            };
            return Ok(Some((
                self.read_small_full(base, base_location, record)?,
                base_location,
            )));
        }
        Ok(Some((
            self.read_small_full(predecessor, location, record)?,
            location,
        )))
    }

    /// The immediate predecessor is optional; selected physical dependencies are not.
    pub(super) fn small_predecessor(
        &self,
        id: ObjectId,
        location: Option<Location>,
        target_canonical_length: usize,
    ) -> Result<Option<SmallPredecessor>> {
        if !(24..content::SMALL_LIMIT + 23).contains(&target_canonical_length) {
            return Err(StoreError::Integrity(
                "SmallContent prospective target length",
            ));
        }
        let Some(location) = location else {
            return Ok(None);
        };
        let Some(Extraction::Small(record)) = self.extract_record_group(
            location.pack,
            location.group,
            location.record,
            location.canonical_length,
            true,
            None,
        )?
        else {
            return Ok(None);
        };
        let Some(prior) = self.small_chain(id, location, record, true, target_canonical_length)?
        else {
            return Ok(None);
        };
        if prior.depth + 1 > super::delta::CHAIN_EDGES
            || prior
                .canonical_closure
                .checked_add(target_canonical_length)
                .is_none_or(|size| size > super::delta::CHAIN_CANONICAL_LIMIT)
        {
            return Ok(None);
        }
        Ok(Some(prior))
    }

    fn small_chain(
        &self,
        target: ObjectId,
        target_location: Location,
        initial: Vec<u8>,
        optional: bool,
        retained: usize,
    ) -> Result<Option<SmallPredecessor>> {
        use super::delta::{CHAIN_CANONICAL_LIMIT, CHAIN_EDGES, CHAIN_ENCODED_LIMIT};
        let legacy_hint = optional && super::delta::record(&initial)?.kind != 2;
        let mut nodes = Vec::<NativeNode>::with_capacity(CHAIN_EDGES + 1);
        let mut id = target;
        let mut location = target_location;
        let mut record = initial;
        let mut canonical_closure = 0usize;
        let mut encoded_closure = 0usize;
        let mut owned_frames = 0usize;
        let mut require_full = false;
        loop {
            let parsed = super::delta::record(&record)?;
            if (parsed.kind == 2 && !self.small_chain_format())
                || location.record != 0
                || location.canonical_length != parsed.raw_length + 23
                || (require_full && parsed.kind != 0)
            {
                return Err(StoreError::Integrity("SmallContent chain role/locator"));
            }
            let base = parsed.base;
            require_full = parsed.kind == 1;
            canonical_closure += location.canonical_length;
            encoded_closure += record.len();
            owned_frames += record.capacity();
            // A valid old kind-1 predecessor may exceed the new chain budget.
            // That only makes the optional new encoding ineligible.
            if canonical_closure > CHAIN_CANONICAL_LIMIT || owned_frames > CHAIN_ENCODED_LIMIT {
                if legacy_hint {
                    return Ok(None);
                }
                return Err(StoreError::Integrity("SmallContent chain closure bound"));
            }
            let associations = nodes.capacity() * std::mem::size_of::<NativeNode>();
            if associations > 16 * 1024
                || retained
                    + owned_frames
                    + associations
                    + 1024 * 1024
                    + 4 * (content::SMALL_LIMIT + 23)
                    > 2 * 1024 * 1024
            {
                return Err(StoreError::Integrity("SmallContent chain scratch bound"));
            }
            nodes.push(NativeNode {
                id,
                location,
                record,
            });
            let Some(base) = base else {
                break;
            };
            if nodes.len() > CHAIN_EDGES || nodes.iter().any(|node| node.id == base) {
                return Err(StoreError::Integrity("SmallContent chain depth/cycle"));
            }
            let next = self
                .object_locations(&[base])?
                .remove(&base)
                .ok_or(StoreError::Integrity("SmallContent base missing"))?;
            if next.pack > location.pack {
                return Err(StoreError::Integrity("SmallContent base chronology"));
            }
            // Acquisition has no live decoder or decoded operands. Reserve the
            // largest selected group before fetching; the stricter retained-frame
            // cap is checked on the next iteration before any reconstruction.
            if retained + owned_frames + associations + 192 * 1024 + 16 * 1024 > 2 * 1024 * 1024 {
                return Err(StoreError::Integrity(
                    "SmallContent chain acquisition bound",
                ));
            }
            let Some(Extraction::Small(next_record)) = self.extract_record_group(
                next.pack,
                next.group,
                next.record,
                next.canonical_length,
                true,
                None,
            )?
            else {
                return Err(StoreError::Integrity("SmallContent base encoding"));
            };
            self.note_physical(PhysicalStorageReceipt {
                base_fetches: 1,
                ..Default::default()
            });
            id = base;
            location = next;
            record = next_record;
        }
        let depth = nodes.len() - 1;
        let mut canonical: Option<CanonicalObject> = None;
        while let Some(node) = nodes.pop() {
            let prefix = canonical
                .as_ref()
                .map(|base| content::small_bytes(&base.bytes))
                .transpose()?
                .flatten();
            let bytes = super::delta::decode(&node.record, prefix)?;
            if bytes.capacity() > content::SMALL_LIMIT + 23 {
                return Err(StoreError::Integrity("SmallContent canonical capacity"));
            }
            authenticate(node.id, &bytes, node.location.canonical_length)?;
            self.note_physical(PhysicalStorageReceipt {
                decompression_calls: 1,
                decoded_read_bytes: bytes.len() as u64,
                ..Default::default()
            });
            canonical = Some(CanonicalObject { id: node.id, bytes });
        }
        Ok(Some(SmallPredecessor {
            canonical: canonical.ok_or(StoreError::Integrity("SmallContent empty chain"))?,
            location: target_location,
            depth,
            canonical_closure,
            encoded_closure,
        }))
    }

    fn read_small(
        &self,
        id: ObjectId,
        location: Location,
        record: Vec<u8>,
        bases: &mut BTreeMap<ObjectId, CanonicalObject>,
    ) -> Result<CanonicalObject> {
        let parsed = super::delta::record(&record)?;
        if parsed.kind == 2 {
            bases.clear();
            return self
                .small_chain(id, location, record, false, 0)?
                .map(|prior| prior.canonical)
                .ok_or(StoreError::Integrity("SmallContent required chain"));
        }
        if location.canonical_length != parsed.raw_length + 23 || location.record != 0 {
            return Err(StoreError::Integrity("SmallContent locator length"));
        }
        let Some(base) = parsed.base else {
            return self.read_small_full(id, location, record);
        };
        if base == id {
            return Err(StoreError::Integrity("SmallContent dependency cycle"));
        }
        if !bases.contains_key(&base) {
            // The read wave retains at most 256 KiB of authenticated FULL operands.
            if bases.values().map(|v| v.bytes.capacity()).sum::<usize>() + content::SMALL_LIMIT + 23
                > 256 * 1024
            {
                bases.clear();
            }
            let base_location = self
                .object_locations(&[base])?
                .remove(&base)
                .ok_or(StoreError::Integrity("SmallContent base missing"))?;
            if base_location.pack > location.pack {
                return Err(StoreError::Integrity("SmallContent base chronology"));
            }
            let Some(Extraction::Small(base_record)) = self.extract_record_group(
                base_location.pack,
                base_location.group,
                base_location.record,
                base_location.canonical_length,
                true,
                None,
            )?
            else {
                return Err(StoreError::Integrity("SmallContent base encoding"));
            };
            bases.insert(
                base,
                self.read_small_full(base, base_location, base_record)?,
            );
            self.note_physical(PhysicalStorageReceipt {
                base_fetches: 1,
                ..Default::default()
            });
        }
        let raw = content::small_bytes(&bases[&base].bytes)?
            .ok_or(StoreError::Integrity("SmallContent base canonical role"))?;
        let bytes = super::delta::decode(&record, Some(raw))?;
        authenticate(id, &bytes, location.canonical_length)?;
        self.note_physical(PhysicalStorageReceipt {
            decompression_calls: 1,
            decoded_read_bytes: bytes.len() as u64,
            ..Default::default()
        });
        Ok(CanonicalObject { id, bytes })
    }

    /// The guard and Blob never leave this extraction boundary.
    fn extract_group(&self, pack_id: i64, group: usize) -> Result<(pack::GroupEntry, Vec<u8>)> {
        match self.extract_record_group(pack_id, group, 0, 0, false, None)? {
            Some(Extraction::Legacy(entry, encoded)) => Ok((entry, encoded)),
            _ => Err(StoreError::Integrity("legacy required group version")),
        }
    }

    fn extract_record_group(
        &self,
        pack_id: i64,
        group: usize,
        ordinal: usize,
        canonical_length: usize,
        native: bool,
        mut budget: Option<&mut HintReadBudget>,
    ) -> Result<Option<Extraction>> {
        if budget.as_deref_mut().is_some_and(|b| !b.charge(32, 0)) {
            return Ok(None);
        }
        let connection = self.reader()?;
        let blob = connection.blob_open("main", "object_packs", "data", pack_id, true)?;
        let length = blob.len();
        let mut header_bytes = [0; 16];
        blob.read_at_exact(&mut header_bytes, 0)?;
        if &header_bytes[..8] == super::whole::MAGIC {
            if !self.compact_namespace() {
                return Err(StoreError::Integrity("whole-file pack requires schema 10"));
            }
            if !native {
                return Ok(Some(Extraction::NativeUnsupported));
            }
            drop(blob);
            drop(connection);
            return Ok(self
                .whole_record(
                    Location {
                        canonical_length,
                        pack: pack_id,
                        group,
                        record: ordinal,
                    },
                    budget,
                )?
                .map(Extraction::Whole));
        }
        let header = pack::versioned_header(&header_bytes, length)?;
        if group >= header.group_count {
            return Err(StoreError::Integrity("object group locator"));
        }
        if header.version == pack::Version::CompactSmall
            && budget
                .as_deref_mut()
                .is_some_and(|b| !b.charge(4 * header.group_count, 8))
        {
            return Ok(None);
        }
        let entry = if header.version == pack::Version::CompactSmall {
            if !self.compact_framing() {
                return Err(StoreError::Integrity("compact framing requires schema 10"));
            }
            let mut starts = [0; 4 * pack::GROUP_COUNT_LIMIT];
            let starts = &mut starts[..4 * header.group_count];
            blob.read_at_exact(starts, 16)?;
            pack::compact_entry(starts, header, length, group)?
        } else {
            let mut directory = [0; 16];
            blob.read_at_exact(&mut directory, 16 + 16 * group)?;
            pack::versioned_entry(&directory, header, length)?
        };
        self.note_physical(PhysicalStorageReceipt {
            group_fetches: 1,
            blob_ranges: 2,
            ..Default::default()
        });
        if matches!(
            header.version,
            pack::Version::Legacy | pack::Version::Metadata | pack::Version::PooledMetadata
        ) {
            if header.version != pack::Version::Legacy && !self.compact_namespace() {
                return Err(StoreError::Integrity("metadata framing requires schema 10"));
            }
            if budget
                .as_deref_mut()
                .is_some_and(|b| !b.charge(entry.range.len(), entry.decoded_length))
            {
                return Ok(None);
            }
            if entry.oversized {
                return Ok(Some(Extraction::Legacy(entry, Vec::new())));
            }
            let mut encoded = vec![0; entry.range.len()];
            blob.read_at_exact(&mut encoded, entry.range.start)?;
            self.note_physical(PhysicalStorageReceipt {
                encoded_read_bytes: encoded.len() as u64,
                blob_ranges: 1,
                ..Default::default()
            });
            return Ok(Some(if header.version != pack::Version::Legacy {
                Extraction::Metadata(
                    header.version == pack::Version::PooledMetadata,
                    entry,
                    encoded,
                )
            } else {
                Extraction::Legacy(entry, encoded)
            }));
        }
        if matches!(
            header.version,
            pack::Version::Small | pack::Version::CompactSmall
        ) {
            if !self.small_content_format() || ordinal != 0 {
                return Err(StoreError::Integrity("SmallContent format/ordinal"));
            }
            if !native {
                return Ok(Some(Extraction::NativeUnsupported));
            }
            if budget
                .as_deref_mut()
                .is_some_and(|b| !b.charge(entry.range.len(), entry.range.len()))
            {
                return Ok(None);
            }
            let mut record = Vec::with_capacity(entry.range.len() + 8);
            record.resize(entry.range.len(), 0);
            blob.read_at_exact(&mut record, entry.range.start)?;
            let encoded_length = record.len();
            if header.version == pack::Version::CompactSmall {
                super::delta::expand_compact(&mut record, canonical_length)?;
            }
            if super::delta::record(&record)?.kind == 2 && !self.small_chain_format() {
                return Err(StoreError::Integrity(
                    "SmallContent chain requires schema 9",
                ));
            }
            self.note_physical(PhysicalStorageReceipt {
                encoded_read_bytes: encoded_length as u64,
                blob_ranges: 1,
                ..Default::default()
            });
            return Ok(Some(Extraction::Small(record)));
        }
        self.note_physical(PhysicalStorageReceipt {
            native_record_fetches: 1,
            native_request_bytes: 32,
            ..Default::default()
        });
        if !native {
            return Ok(Some(Extraction::NativeUnsupported));
        }
        if entry.range.len() < 8 {
            return Err(StoreError::Integrity("native group framing"));
        }
        if budget.as_deref_mut().is_some_and(|b| !b.charge(4, 4)) {
            return Ok(None);
        }
        let mut count_bytes = [0; 4];
        blob.read_at_exact(&mut count_bytes, entry.range.start)?;
        self.note_native_range(4);
        let count = u32::from_le_bytes(count_bytes) as usize;
        if !(1..=pack::RECORD_COUNT_LIMIT).contains(&count) || 4 + 4 * count >= entry.range.len() {
            return Err(StoreError::Integrity("native record count"));
        }
        if budget
            .as_deref_mut()
            .is_some_and(|b| !b.charge(4 * count, 4 * count))
        {
            return Ok(None);
        }
        let mut ends = vec![0; 4 * count];
        blob.read_at_exact(&mut ends, entry.range.start + 4)?;
        self.note_native_range(ends.len());
        let range = pack::native_record_range(count, &ends, entry.range.len(), ordinal)?;
        drop(ends);
        if range.len() > 37 + pack::NATIVE_FRAME_LIMIT {
            return Err(StoreError::Integrity("native record bound"));
        }
        if budget.is_some_and(|b| !b.charge(range.len(), range.len())) {
            return Ok(None);
        }
        let mut record = vec![0; range.len()];
        blob.read_at_exact(&mut record, entry.range.start + range.start)?;
        self.note_native_range(record.len());
        pack::native_record(&record)?;
        let parsed = 4 + 4 * count + record.len();
        Ok(Some(Extraction::Native {
            record,
            requested: 32 + parsed,
            parsed,
        }))
    }

    // One demanded group only; no unrelated record bodies or persistent cache.
    // SQLite ownership ends before any returned frame is decoded/authenticated.
    fn extract_demanded_group(
        &self,
        pack_id: i64,
        group: usize,
        targets: &[(ObjectId, Location)],
    ) -> Result<Vec<Extraction>> {
        if targets.is_empty() || targets.len() > OBJECT_PAGE_COUNT {
            return Err(StoreError::Integrity("native batch request bound"));
        }
        if targets.len() == 1 {
            return Ok(vec![self
                .extract_record_group(
                    pack_id,
                    group,
                    targets[0].1.record,
                    targets[0].1.canonical_length,
                    true,
                    None,
                )?
                .ok_or(StoreError::Integrity("required extraction"))?]);
        }
        let connection = self.reader()?;
        let blob = connection.blob_open("main", "object_packs", "data", pack_id, true)?;
        let length = blob.len();
        let mut header_bytes = [0; 16];
        blob.read_at_exact(&mut header_bytes, 0)?;
        if &header_bytes[..8] == super::whole::MAGIC {
            return Err(StoreError::Integrity("whole-file record locator alias"));
        }
        let header = pack::versioned_header(&header_bytes, length)?;
        if group >= header.group_count {
            return Err(StoreError::Integrity("object group locator"));
        }
        let entry = if header.version == pack::Version::CompactSmall {
            if !self.compact_framing() {
                return Err(StoreError::Integrity("compact framing requires schema 10"));
            }
            let mut starts = [0; 4 * pack::GROUP_COUNT_LIMIT];
            let starts = &mut starts[..4 * header.group_count];
            blob.read_at_exact(starts, 16)?;
            pack::compact_entry(starts, header, length, group)?
        } else {
            let mut directory = [0; 16];
            blob.read_at_exact(&mut directory, 16 + 16 * group)?;
            pack::versioned_entry(&directory, header, length)?
        };
        self.note_physical(PhysicalStorageReceipt {
            group_fetches: 1,
            blob_ranges: 2,
            ..Default::default()
        });
        if matches!(
            header.version,
            pack::Version::Legacy | pack::Version::Metadata | pack::Version::PooledMetadata
        ) {
            if header.version != pack::Version::Legacy && !self.compact_namespace() {
                return Err(StoreError::Integrity("metadata framing requires schema 10"));
            }
            if entry.oversized {
                return Ok(vec![Extraction::Legacy(entry, Vec::new())]);
            }
            let mut encoded = vec![0; entry.range.len()];
            blob.read_at_exact(&mut encoded, entry.range.start)?;
            self.note_physical(PhysicalStorageReceipt {
                encoded_read_bytes: encoded.len() as u64,
                blob_ranges: 1,
                ..Default::default()
            });
            return Ok(vec![if header.version != pack::Version::Legacy {
                Extraction::Metadata(
                    header.version == pack::Version::PooledMetadata,
                    entry,
                    encoded,
                )
            } else {
                Extraction::Legacy(entry, encoded)
            }]);
        }
        if matches!(
            header.version,
            pack::Version::Small | pack::Version::CompactSmall
        ) {
            if !self.small_content_format() || targets.len() != 1 || targets[0].1.record != 0 {
                return Err(StoreError::Integrity("SmallContent selected locator"));
            }
            let mut record = Vec::with_capacity(entry.range.len() + 8);
            record.resize(entry.range.len(), 0);
            blob.read_at_exact(&mut record, entry.range.start)?;
            let encoded_length = record.len();
            if header.version == pack::Version::CompactSmall {
                super::delta::expand_compact(&mut record, targets[0].1.canonical_length)?;
            }
            if super::delta::record(&record)?.kind == 2 && !self.small_chain_format() {
                return Err(StoreError::Integrity(
                    "SmallContent chain requires schema 9",
                ));
            }
            self.note_physical(PhysicalStorageReceipt {
                encoded_read_bytes: encoded_length as u64,
                blob_ranges: 1,
                ..Default::default()
            });
            return Ok(vec![Extraction::Small(record)]);
        }
        if entry.range.len() < 8 || entry.range.len() > pack::GROUP_LIMIT {
            return Err(StoreError::Integrity("native group framing"));
        }
        let mut count_bytes = [0; 4];
        blob.read_at_exact(&mut count_bytes, entry.range.start)?;
        self.note_native_range(4);
        let count = u32::from_le_bytes(count_bytes) as usize;
        if !(1..=pack::RECORD_COUNT_LIMIT).contains(&count) || 4 + 4 * count >= entry.range.len() {
            return Err(StoreError::Integrity("native record count"));
        }
        let mut ends = vec![0; 4 * count];
        blob.read_at_exact(&mut ends, entry.range.start + 4)?;
        self.note_native_range(ends.len());
        let mut extracted = Vec::with_capacity(targets.len());
        let mut owned = 0;
        for (_, location) in targets {
            let range =
                pack::native_record_range(count, &ends, entry.range.len(), location.record)?;
            if range.len() > 37 + pack::NATIVE_FRAME_LIMIT {
                return Err(StoreError::Integrity("native record bound"));
            }
            let mut record = vec![0; range.len()];
            owned += record.capacity();
            if owned > pack::GROUP_LIMIT {
                return Err(StoreError::Integrity("native batch frame bound"));
            }
            blob.read_at_exact(&mut record, entry.range.start + range.start)?;
            self.note_native_range(record.len());
            pack::native_record(&record)?;
            // Per-target work bounds remain conservative as in the point route;
            // actual I/O telemetry counts the shared directory only once.
            let parsed = 4 + 4 * count + record.len();
            extracted.push(Extraction::Native {
                record,
                requested: 32 + parsed,
                parsed,
            });
        }
        self.note_physical(PhysicalStorageReceipt {
            native_record_fetches: targets.len() as u64,
            native_request_bytes: 32,
            ..Default::default()
        });
        Ok(extracted)
    }

    fn note_native_range(&self, bytes: usize) {
        self.note_physical(PhysicalStorageReceipt {
            encoded_read_bytes: bytes as u64,
            blob_ranges: 1,
            native_request_bytes: bytes as u64,
            native_parser_bytes: bytes as u64,
            ..Default::default()
        });
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
        let Some(extracted) = self.extract_record_group(
            location.pack,
            location.group,
            location.record,
            location.canonical_length,
            false,
            Some(budget),
        )?
        else {
            return Ok(None);
        };
        let Extraction::Legacy(entry, encoded) = extracted else {
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

    /// The caller owns begin_target; this shares the legacy optional-work budget.
    pub(super) fn read_native_prior(
        &self,
        id: ObjectId,
        budget: &mut HintReadBudget,
    ) -> Result<NativePriorOutcome> {
        if !self.native_lookup_allowed(Some(budget)) {
            return Ok(NativePriorOutcome::Budget);
        }
        let Some(location) = self.object_locations(&[id])?.remove(&id) else {
            return Ok(NativePriorOutcome::Unavailable);
        };
        self.native_chain(id, location, None, Some(budget))
    }

    fn native_lookup_allowed(&self, budget: Option<&mut HintReadBudget>) -> bool {
        if let Some(budget) = budget {
            if budget.exhausted || budget.target_fetches >= 8 {
                budget.exhausted = true;
                return false;
            }
            budget.target_fetches += 1;
        }
        self.note_physical(PhysicalStorageReceipt {
            base_fetches: 1,
            ..Default::default()
        });
        true
    }

    fn native_chain(
        &self,
        target: ObjectId,
        target_location: Location,
        initial: Option<Extraction>,
        budget: Option<&mut HintReadBudget>,
    ) -> Result<NativePriorOutcome> {
        self.native_chain_with_retained(target, target_location, initial, budget, 0)
    }

    fn native_chain_with_retained(
        &self,
        target: ObjectId,
        target_location: Location,
        mut initial: Option<Extraction>,
        mut budget: Option<&mut HintReadBudget>,
        retained: usize,
    ) -> Result<NativePriorOutcome> {
        let optional = budget.is_some();
        let mut nodes = Vec::<NativeNode>::with_capacity(5);
        let mut id = target;
        let mut location = target_location;
        let mut raw_closure = 0usize;
        let mut encoded_work = 0usize;
        let mut decoded_work = 0usize;
        let mut owned_frames = 0usize;
        let mut canonical;
        loop {
            let extracted = match initial.take() {
                Some(extracted) => extracted,
                None => match self.extract_record_group(
                    location.pack,
                    location.group,
                    location.record,
                    location.canonical_length,
                    true,
                    budget.as_deref_mut(),
                )? {
                    Some(extracted) => extracted,
                    None => return Ok(NativePriorOutcome::Budget),
                },
            };
            if matches!(
                &extracted,
                Extraction::Small(_) | Extraction::Whole(_) | Extraction::Metadata(..)
            ) {
                if optional && nodes.is_empty() {
                    return Ok(NativePriorOutcome::UnsupportedRole);
                }
                return Err(StoreError::Integrity("native dependency is SmallContent"));
            }
            if !(21..=21 + pack::NATIVE_RAW_LIMIT).contains(&location.canonical_length) {
                if optional && nodes.is_empty() && matches!(&extracted, Extraction::Legacy(..)) {
                    return Ok(NativePriorOutcome::UnsupportedRole);
                }
                return Err(StoreError::Integrity("native dependency canonical length"));
            }
            let (requested, parsed) = match &extracted {
                Extraction::Legacy(entry, _) => (32 + entry.range.len(), entry.decoded_length),
                Extraction::Native {
                    requested, parsed, ..
                } => (*requested, *parsed),
                Extraction::Small(_)
                | Extraction::Whole(_)
                | Extraction::Metadata(..)
                | Extraction::NativeUnsupported => unreachable!(),
            };
            encoded_work += requested;
            decoded_work += parsed + location.canonical_length;
            if encoded_work > 393_216 || decoded_work > 524_288 {
                return Err(StoreError::Integrity("native chain work bound"));
            }
            if budget
                .as_deref_mut()
                .is_some_and(|b| !b.charge(0, location.canonical_length))
            {
                return Ok(NativePriorOutcome::Budget);
            }
            raw_closure += location.canonical_length - 21;
            if raw_closure > 1_048_576 {
                return Err(StoreError::Integrity("native closure bound"));
            }
            match extracted {
                Extraction::Legacy(entry, encoded) => {
                    if entry.oversized {
                        return Err(StoreError::Integrity("native oversized legacy base"));
                    }
                    self.note_physical(PhysicalStorageReceipt {
                        decoded_read_bytes: entry.decoded_length as u64,
                        decompression_calls: u64::from(entry.codec == pack::Codec::Zstandard),
                        ..Default::default()
                    });
                    let decoded = pack::decode_group(entry, encoded)?;
                    let mut full = None;
                    let mut delta = false;
                    pack::visit_records(&decoded, false, |index, record| {
                        if index == location.record {
                            match record {
                                pack::Record::Full(bytes) => {
                                    authenticate(id, bytes, location.canonical_length)?;
                                    full = Some(bytes.to_vec());
                                }
                                pack::Record::Delta { .. } => delta = true,
                            }
                        }
                        Ok(())
                    })?;
                    if delta {
                        if optional && nodes.is_empty() {
                            return Ok(NativePriorOutcome::UnsupportedLegacyDelta);
                        }
                        return Err(StoreError::Integrity("native dependency legacy DELTA"));
                    }
                    canonical =
                        full.ok_or(StoreError::Integrity("native legacy record locator"))?;
                    if chunk_payload(&canonical).is_err() {
                        if optional && nodes.is_empty() {
                            return Ok(NativePriorOutcome::UnsupportedRole);
                        }
                        return Err(StoreError::Integrity("native dependency role"));
                    }
                    break;
                }
                Extraction::Native { record, .. } => {
                    let (raw_length, base) = match pack::native_record(&record)? {
                        pack::NativeRecord::Full { raw_length, .. } => (raw_length, None),
                        pack::NativeRecord::Prefix {
                            raw_length, base, ..
                        } => (raw_length, Some(base)),
                    };
                    if raw_length + 21 != location.canonical_length {
                        return Err(StoreError::Integrity("native canonical length"));
                    }
                    owned_frames += record.capacity();
                    // Worst simultaneous ownership includes both legacy group buffers,
                    // two raw outputs plus canonical framing, directory, and static codec.
                    if retained
                        + owned_frames
                        + nodes.capacity() * std::mem::size_of::<NativeNode>()
                        + 2 * pack::GROUP_LIMIT
                        + 3 * (pack::NATIVE_RAW_LIMIT + 21)
                        + 4 * pack::RECORD_COUNT_LIMIT
                        + pack::NATIVE_DECODE_WORKSPACE
                        > VALIDATION_RESERVE
                    {
                        return Err(StoreError::Integrity("native chain scratch bound"));
                    }
                    nodes.push(NativeNode {
                        id,
                        location,
                        record,
                    });
                    let Some(base) = base else {
                        canonical = Vec::new();
                        break;
                    };
                    if nodes.len() >= 5 || nodes.iter().any(|node| node.id == base) {
                        return Err(StoreError::Integrity("native dependency depth or cycle"));
                    }
                    if !self.native_lookup_allowed(budget.as_deref_mut()) {
                        return Ok(NativePriorOutcome::Budget);
                    }
                    let next = self
                        .object_locations(&[base])?
                        .remove(&base)
                        .ok_or(StoreError::Integrity("native dependency missing"))?;
                    if next.pack >= location.pack {
                        return Err(StoreError::Integrity("native dependency chronology"));
                    }
                    self.note_physical(PhysicalStorageReceipt {
                        native_dependency_edges: 1,
                        ..Default::default()
                    });
                    id = base;
                    location = next;
                }
                Extraction::Small(_)
                | Extraction::Whole(_)
                | Extraction::Metadata(..)
                | Extraction::NativeUnsupported => unreachable!(),
            }
        }
        let depth = if canonical.is_empty() {
            nodes.len() - 1
        } else {
            nodes.len()
        };
        for node in nodes.iter().rev() {
            let (raw_length, frame, prefix) = match pack::native_record(&node.record)? {
                pack::NativeRecord::Full { raw_length, frame } => (raw_length, frame, None),
                pack::NativeRecord::Prefix {
                    raw_length, frame, ..
                } => (raw_length, frame, Some(chunk_payload(&canonical)?)),
            };
            let started = std::time::Instant::now();
            let raw = pack::native_decompress(frame, raw_length, prefix);
            self.note_physical(PhysicalStorageReceipt {
                native_decode_calls: 1,
                native_decode_ns: started.elapsed().as_nanos().min(u64::MAX as u128) as u64,
                native_raw_decoded_bytes: if raw.is_ok() { raw_length as u64 } else { 0 },
                ..Default::default()
            });
            let raw = raw?;
            canonical = layerfs_content::file::extent_codec::encode_chunk_object(&raw)?;
            authenticate(node.id, &canonical, node.location.canonical_length)?;
        }
        let mut stats = PhysicalStorageReceipt::default();
        match depth {
            0 => stats.native_depth_0 = 1,
            1 => stats.native_depth_1 = 1,
            2 => stats.native_depth_2 = 1,
            3 => stats.native_depth_3 = 1,
            4 => stats.native_depth_4 = 1,
            _ => return Err(StoreError::Integrity("native depth bound")),
        }
        self.note_physical(stats);
        Ok(NativePriorOutcome::Available {
            canonical: CanonicalObject {
                id: target,
                bytes: canonical,
            },
            location: target_location,
            depth: depth as u8,
            raw_closure,
        })
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
        // Canonical length does not identify the physical grammar: upper-range
        // SmallContent uses pack v3, while legacy oversized objects stream below.
        let entry = match self.extract_record_group(
            location.pack,
            location.group,
            location.record,
            location.canonical_length,
            true,
            None,
        )? {
            Some(Extraction::Whole(record)) => {
                let object = self.read_whole(
                    id,
                    location,
                    record,
                    &mut super::whole::OwnerCache::default(),
                )?;
                return if object.bytes == canonical {
                    Ok(())
                } else {
                    Err(StoreError::Integrity("object collision"))
                };
            }
            Some(Extraction::Small(record)) => {
                let object = self.read_small(id, location, record, &mut BTreeMap::new())?;
                return if object.bytes == canonical {
                    Ok(())
                } else {
                    Err(StoreError::Integrity("object collision"))
                };
            }
            Some(Extraction::Legacy(entry, _)) => entry,
            _ => return Err(StoreError::Integrity("oversized comparison group version")),
        };
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
        emit: impl FnMut(CanonicalObject) -> Result<()>,
    ) -> Result<()> {
        self.visit_locations_with_reserve(locations, VALIDATION_RESERVE, emit)
    }

    pub(super) fn visit_locations_with_reserve(
        &self,
        locations: &mut [(ObjectId, Location)],
        reserve_limit: usize,
        mut emit: impl FnMut(CanonicalObject) -> Result<()>,
    ) -> Result<()> {
        let mut emit = |object: CanonicalObject| {
            self.check_canonical_format(&object.bytes)?;
            emit(object)
        };
        if reserve_limit > VALIDATION_RESERVE {
            return Err(StoreError::Integrity("packed read reserve ceiling"));
        }
        locations
            .sort_unstable_by_key(|(_, location)| (location.pack, location.group, location.record));
        let mut start = 0;
        while start < locations.len() {
            let mut end = start;
            let mut reserve = 0;
            while end < locations.len() && end - start < OBJECT_PAGE_COUNT {
                let next = validation_reserve(locations[end].1.canonical_length);
                if reserve + next > reserve_limit {
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
        let mut small_bases = BTreeMap::new();
        let mut whole_owners = super::whole::OwnerCache::default();
        for ((pack_id, group), targets) in groups {
            // The selected entry decides the 65528..65536 RAW/DELTA overlap.
            // Do not fetch its directory a second time just to choose the route.
            let mut ordered = targets;
            ordered.sort_unstable_by_key(|(_, location)| location.record);
            if ordered
                .windows(2)
                .any(|pair| pair[0].1.record == pair[1].1.record)
            {
                return Err(StoreError::Integrity(
                    "distinct object IDs alias one record",
                ));
            }
            let mut extracted = self.extract_demanded_group(pack_id, group, &ordered)?;
            let first = extracted.remove(0);
            let (entry, encoded) = match first {
                Extraction::Whole(record) => {
                    let [(id, location)] = ordered.as_slice() else {
                        return Err(StoreError::Integrity("whole-file locator alias"));
                    };
                    emit(self.read_whole(*id, *location, record, &mut whole_owners)?)?;
                    continue;
                }
                Extraction::Small(record) => {
                    let [(id, location)] = ordered.as_slice() else {
                        return Err(StoreError::Integrity("SmallContent locator alias"));
                    };
                    emit(self.read_small(*id, *location, record, &mut small_bases)?)?;
                    continue;
                }
                Extraction::Metadata(pooled, entry, encoded) => {
                    let decoded = self.decode_metadata_group(entry, encoded)?;
                    // One bounded pool reader serves every demand of this record
                    // group. Its value cache, retention bound, eviction and
                    // decoded-work ceiling are unchanged; only the scope of the
                    // reuse changes, from one object to this bounded wave.
                    let mut wave_pool = super::metadata::PoolRead::default();
                    for (id, location) in ordered {
                        let record = metadata_record(&decoded, location.record)?;
                        emit(
                            self.metadata_chain(
                                id,
                                location,
                                pooled,
                                record,
                                None,
                                &mut wave_pool,
                            )?
                            .ok_or(StoreError::Integrity("required metadata chain"))?
                            .canonical,
                        )?;
                    }
                    continue;
                }
                Extraction::Legacy(entry, encoded) => (entry, encoded),
                first @ Extraction::Native { .. } => {
                    // Charge all surviving prefetched frames/associations against
                    // the same 1 MiB active-chain scratch ceiling.
                    extracted.insert(0, first);
                    let associations = extracted.capacity() * std::mem::size_of::<Extraction>()
                        + ordered.capacity() * std::mem::size_of::<(ObjectId, Location)>();
                    let mut remaining = extracted
                        .iter()
                        .map(|e| match e {
                            Extraction::Native { record, .. } => record.capacity(),
                            _ => 0,
                        })
                        .sum::<usize>();
                    for ((id, location), initial) in ordered.into_iter().zip(extracted) {
                        if let Extraction::Native { record, .. } = &initial {
                            remaining -= record.capacity();
                        }
                        let result = self.native_chain_with_retained(
                            id,
                            location,
                            Some(initial),
                            None,
                            remaining + associations,
                        )?;
                        let NativePriorOutcome::Available { canonical, .. } = result else {
                            return Err(StoreError::Integrity("required native chain"));
                        };
                        emit(canonical)?;
                    }
                    continue;
                }
                Extraction::NativeUnsupported => unreachable!(),
            };
            if entry.oversized {
                let [(id, location)] = ordered.as_slice() else {
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
            let mut requested = record_slots(ordered)?;
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

fn authenticate_metadata(id: ObjectId, bytes: &[u8], length: usize) -> Result<()> {
    authenticate(id, bytes, length)?;
    if !metadata_leaf(bytes) {
        return Err(StoreError::Integrity("metadata dependency role"));
    }
    layerfs_content::tree::compact::decode_inode(bytes)?;
    Ok(())
}

pub(super) fn authenticate(id: ObjectId, bytes: &[u8], length: usize) -> Result<()> {
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

#[cfg(test)]
mod native_tests;
