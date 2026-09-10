use super::super::metadata;
use super::*;

pub(super) struct PreparedValueGroup {
    pub first: u64,
    pub count: usize,
    pub pack: usize,
    pub group: usize,
    pub digest: ObjectId,
}

pub(super) struct PreparedValues {
    pub physical: Vec<Vec<u8>>,
    pub groups: Vec<(PreparedValueGroup, pack::EncodedGroup)>,
}

impl PreparedValues {
    pub(super) fn backing(&self) -> usize {
        self.physical.capacity() * std::mem::size_of::<Vec<u8>>()
            + self.physical.iter().map(Vec::capacity).sum::<usize>()
            + self.groups.capacity()
                * std::mem::size_of::<(PreparedValueGroup, pack::EncodedGroup)>()
            + self
                .groups
                .iter()
                .map(|(_, group)| group.bytes.capacity())
                .sum::<usize>()
    }
}

pub(super) fn prepare_values(
    db: &StoreDb,
    objects: &[AuthenticatedCanonicalObject],
    next: Option<u64>,
    pending: &mut BTreeMap<[u8; 73], u32>,
    stats: &mut crate::PhysicalStorageReceipt,
) -> Result<PreparedValues> {
    if objects
        .iter()
        .map(|object| object.bytes.len())
        .sum::<usize>()
        > 128 * 1024
    {
        return Err(StoreError::Integrity("metadata pool preparation bound"));
    }
    let mut index = db.metadata_index()?;
    if index.is_none() {
        *index = Some(metadata::ValueIndex::new()?);
    }
    let index = index.as_mut().unwrap();
    index.sync(db)?;
    let first = next.unwrap_or(db.next_metadata_ordinal()?);
    // Pending values are private to this prepared publication. At most one per
    // 81-byte canonical row in the <=512-KiB ordinary admission batch; charge a
    // conservative whole B-tree node per entry within the existing index budget.
    if (pending.len()
        + objects
            .iter()
            .map(|object| (object.bytes.len() - 44) / 81)
            .sum::<usize>())
        * 256
        > super::super::CANDIDATE_INDEX_BYTES
    {
        return Err(StoreError::Integrity("metadata pending index bound"));
    }
    let mut values = Vec::new();
    let mut physical = Vec::with_capacity(objects.len());
    for object in objects {
        let length = metadata::physical_length(object.bytes.len())?;
        let mut bytes = Vec::with_capacity(length);
        bytes.extend_from_slice(&object.bytes[..44]);
        for row in object.bytes[44..].chunks_exact(81) {
            let value: [u8; 73] = row[8..].try_into().unwrap();
            let ordinal = if let Some(ordinal) = pending.get(&value) {
                *ordinal
            } else if let Some(ordinal) = index.find(&value)? {
                ordinal
            } else {
                let ordinal = u32::try_from(first + values.len() as u64)
                    .map_err(|_| StoreError::Integrity("metadata ordinal exhausted"))?;
                pending.insert(value, ordinal);
                values.push(value);
                ordinal
            };
            bytes.extend_from_slice(&row[..8]);
            bytes.extend_from_slice(&ordinal.to_be_bytes());
        }
        physical.push(bytes);
    }
    let mut groups = Vec::new();
    for (number, values) in values.chunks(metadata::VALUES_PER_GROUP).enumerate() {
        let canonical = values
            .iter()
            .map(metadata::value_canonical)
            .collect::<Result<Vec<_>>>()?;
        let slices = canonical.iter().map(Vec::as_slice).collect::<Vec<_>>();
        let (group, mixed) = pack::encode_group(&slices, &vec![None; values.len()], stats)?;
        if mixed || group.decoded_length > 16 * 1024 {
            return Err(StoreError::Integrity("metadata value group construction"));
        }
        let body = pack::decode_group(
            pack::GroupEntry {
                range: 0..group.bytes.len(),
                decoded_length: group.decoded_length,
                codec: group.codec,
                oversized: false,
            },
            group.bytes.clone(),
        )?;
        groups.push((
            PreparedValueGroup {
                first: first + (number * metadata::VALUES_PER_GROUP) as u64,
                count: values.len(),
                pack: 0,
                group: 0,
                digest: ObjectId::for_bytes(&body),
            },
            group,
        ));
    }
    Ok(PreparedValues { physical, groups })
}
