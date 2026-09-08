//! Private derived spill ownership, seen/location pages and sealed construction order.
use super::*;
use rusqlite::limits::Limit;
use std::io::BufWriter;

pub(super) struct SpillObjects {
    pub(super) writer: Option<std::fs::File>,
    pub(super) reader: Mutex<std::fs::File>,
    pub(super) path: PathBuf,
    pub(super) pending: Vec<u8>,
    pub(super) pending_index: BTreeMap<ObjectId, (usize, usize)>,
    pub(super) end: u64,
    pub(super) index: Option<BTreeMap<ObjectId, (u64, u64)>>,
    pub(super) index_bytes: usize,
    pub(super) disk_index: Option<Box<SpillDiskIndex>>,
    pub(super) index_limit: usize,
    pub(super) buffer_bytes: usize,
    pub(super) order_memory_bytes: usize,
    pub(super) failed: bool,
}

pub(super) struct SpillDiskIndex {
    // Drop the connection before its owned temporary path.
    connection: Mutex<Connection>,
    _path: TempPath,
}

impl Drop for SpillObjects {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

pub(super) enum IdOrder {
    Memory(Vec<ObjectId>),
    Spill {
        writer: Option<BufWriter<std::fs::File>>,
        path: TempPath,
    },
}

pub(super) struct TempPath(pub(super) PathBuf);

impl Drop for TempPath {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

pub(super) const ID_BUFFER_BYTES: usize = 64 * 1024;

impl IdOrder {
    pub(super) fn empty() -> Self {
        Self::Memory(Vec::new())
    }

    pub(super) fn push_bounded(&mut self, id: ObjectId, limit: usize) -> Result<()> {
        if matches!(self, Self::Memory(ids) if ids.len().saturating_add(1).saturating_mul(32) > limit)
        {
            let Self::Memory(ids) = std::mem::replace(self, Self::Memory(Vec::new())) else {
                unreachable!()
            };
            let (file, path) = temporary_file("candidate-order")?;
            let path = TempPath(path);
            let mut writer = BufWriter::with_capacity(ID_BUFFER_BYTES, file);
            for id in ids {
                writer.write_all(id.as_bytes())?;
            }
            *self = Self::Spill {
                writer: Some(writer),
                path,
            };
        }
        match self {
            Self::Memory(ids) => ids.push(id),
            Self::Spill { writer, path } => {
                if writer.is_none() {
                    *writer = Some(BufWriter::with_capacity(
                        ID_BUFFER_BYTES,
                        std::fs::OpenOptions::new().append(true).open(&path.0)?,
                    ));
                }
                writer
                    .as_mut()
                    .expect("order writer")
                    .write_all(id.as_bytes())?;
            }
        }
        Ok(())
    }

    pub(super) fn seal(&mut self) -> Result<()> {
        if let Self::Spill { writer, .. } = self {
            if let Some(mut writer) = writer.take() {
                writer.flush()?;
                writer.into_inner().map_err(|error| error.into_error())?;
            }
        }
        Ok(())
    }

    pub(super) fn visit(&self, mut visitor: impl FnMut(ObjectId) -> Result<()>) -> Result<()> {
        match self {
            Self::Memory(ids) => {
                for id in ids {
                    visitor(*id)?;
                }
            }
            Self::Spill { writer, path } => {
                if writer.is_some() {
                    return Err(StoreError::Integrity("unsealed candidate order"));
                }
                let mut file =
                    BufReader::with_capacity(ID_BUFFER_BYTES, std::fs::File::open(&path.0)?);
                loop {
                    let mut bytes = [0; 32];
                    // Only zero bytes before a record is clean EOF; a partial ID fails.
                    if file.read(&mut bytes[..1])? == 0 {
                        break;
                    }
                    file.read_exact(&mut bytes[1..])?;
                    visitor(ObjectId::from_bytes(&bytes)?)?;
                }
            }
        }
        Ok(())
    }

    pub(super) fn visit_pages(
        &self,
        mut visitor: impl FnMut(&[ObjectId]) -> Result<()>,
    ) -> Result<()> {
        let mut page = Vec::with_capacity(OBJECT_PAGE_COUNT);
        self.visit(|id| {
            page.push(id);
            if page.len() == OBJECT_PAGE_COUNT {
                visitor(&page)?;
                page.clear();
            }
            Ok(())
        })?;
        if !page.is_empty() {
            visitor(&page)?;
        }
        Ok(())
    }
}

pub struct SpillableObjectSet {
    pub(super) storage: SeenStorage,
    pub(super) count: usize,
    pub(super) memory_limit: usize,
    failed: bool,
}

// Preserve inline spill ownership; Connection/Mutex layout varies by platform.
#[allow(clippy::large_enum_variant)]
pub(super) enum SeenStorage {
    Memory(BTreeSet<ObjectId>),
    Spill {
        connection: Mutex<Connection>,
        _path: TempPath,
    },
}

// Fixed page-sized statements also respect an unusually small effective SQLite limit.
pub(super) fn sql_rows(
    connection: &Connection,
    parameters: usize,
    prefix: usize,
    row_sql: usize,
) -> Result<usize> {
    let variables = usize::try_from(connection.limit(Limit::SQLITE_LIMIT_VARIABLE_NUMBER)?)
        .map_err(|_| StoreError::Integrity("SQLite parameter limit"))?;
    let sql = usize::try_from(connection.limit(Limit::SQLITE_LIMIT_SQL_LENGTH)?)
        .map_err(|_| StoreError::Integrity("SQLite statement limit"))?;
    let rows = OBJECT_PAGE_COUNT
        .min(variables / parameters)
        .min(sql.saturating_sub(prefix + 1) / row_sql);
    if rows == 0 {
        return Err(StoreError::Integrity("SQLite limit below one row"));
    }
    Ok(rows)
}

fn insert_seen(connection: &mut Connection, ids: &[ObjectId]) -> Result<Vec<ObjectId>> {
    let rows = sql_rows(connection, 1, 96, 4)?;
    let transaction = connection.transaction()?;
    let mut inserted = BTreeSet::new();
    for page in ids.chunks(rows) {
        let sql = format!(
            "INSERT OR IGNORE INTO seen(id) VALUES {} RETURNING id",
            vec!["(?)"; page.len()].join(",")
        );
        let mut statement = transaction.prepare(&sql)?;
        for id in statement.query_map(
            params_from_iter(page.iter().map(|id| id.as_bytes().as_slice())),
            |row| row.get::<_, Vec<u8>>(0),
        )? {
            inserted.insert(ObjectId::from_bytes(&id?)?);
        }
    }
    transaction.commit()?;
    // RETURNING order is unspecified. Return first occurrences in caller order.
    Ok(ids
        .iter()
        .copied()
        .filter(|id| inserted.remove(id))
        .collect())
}

impl SpillableObjectSet {
    pub fn empty() -> Result<Self> {
        Self::bounded(CANDIDATE_INDEX_BYTES)
    }
    pub(super) fn bounded(memory_limit: usize) -> Result<Self> {
        Ok(Self {
            storage: SeenStorage::Memory(BTreeSet::new()),
            count: 0,
            memory_limit,
            failed: false,
        })
    }
    fn healthy(&self) -> Result<()> {
        if self.failed {
            Err(StoreError::Integrity("invalidated candidate seen index"))
        } else {
            Ok(())
        }
    }
    pub fn contains(&self, id: ObjectId) -> Result<bool> {
        Ok(self.membership(&[id])?.contains(&id))
    }
    pub(super) fn membership(&self, ids: &[ObjectId]) -> Result<BTreeSet<ObjectId>> {
        self.healthy()?;
        match &self.storage {
            SeenStorage::Memory(known) => Ok(ids
                .iter()
                .filter(|id| known.contains(id))
                .copied()
                .collect()),
            SeenStorage::Spill { connection, .. } => {
                let connection = connection
                    .lock()
                    .map_err(|_| StoreError::Integrity("candidate seen index"))?;
                let count = sql_rows(&connection, 1, 64, 2)?;
                let mut found = BTreeSet::new();
                for page in ids.chunks(count) {
                    let sql = format!(
                        "SELECT id FROM seen WHERE id IN ({})",
                        vec!["?"; page.len()].join(",")
                    );
                    let mut statement = connection.prepare(&sql)?;
                    for id in statement.query_map(
                        params_from_iter(page.iter().map(|id| id.as_bytes().as_slice())),
                        |row| row.get::<_, Vec<u8>>(0),
                    )? {
                        found.insert(ObjectId::from_bytes(&id?)?);
                    }
                }
                Ok(found)
            }
        }
    }
    pub(super) fn spill(&mut self) -> Result<()> {
        self.healthy()?;
        let SeenStorage::Memory(known) = &self.storage else {
            return Ok(());
        };
        let (mut connection, path) = scratch_index(
            "candidate-seen",
            "CREATE TABLE seen (id BLOB PRIMARY KEY CHECK(length(id)=32)) WITHOUT ROWID;",
        )?;
        let mut page = Vec::with_capacity(OBJECT_PAGE_COUNT);
        for &id in known {
            page.push(id);
            if page.len() == OBJECT_PAGE_COUNT {
                insert_seen(&mut connection, &page)?;
                page.clear();
            }
        }
        if !page.is_empty() {
            insert_seen(&mut connection, &page)?;
        }
        self.storage = SeenStorage::Spill {
            connection: Mutex::new(connection),
            _path: path,
        };
        Ok(())
    }
    pub fn insert_page(&mut self, ids: &[ObjectId]) -> Result<Vec<ObjectId>> {
        self.healthy()?;
        let result = (|| {
            // Reserve the existing scratch cache. A conservative page estimate avoids per-ID probes.
            if matches!(self.storage, SeenStorage::Memory(_))
                && self.count.saturating_add(ids.len()).saturating_mul(48)
                    > self.memory_limit.saturating_sub(4 * 1024 * 1024)
            {
                self.spill()?;
            }
            let inserted = match &mut self.storage {
                SeenStorage::Memory(known) => {
                    ids.iter().copied().filter(|id| known.insert(*id)).collect()
                }
                SeenStorage::Spill { connection, .. } => insert_seen(
                    connection
                        .get_mut()
                        .map_err(|_| StoreError::Integrity("candidate seen index"))?,
                    ids,
                )?,
            };
            Ok(inserted)
        })();
        match result {
            Ok(inserted) => {
                self.count += inserted.len();
                Ok(inserted)
            }
            Err(error) => {
                self.failed = true;
                Err(error)
            } // OFF-journal mutation is not rollback-safe.
        }
    }
}

impl SpillObjects {
    fn healthy(&self) -> Result<()> {
        if self.failed {
            Err(StoreError::Integrity("invalidated candidate spool"))
        } else {
            Ok(())
        }
    }

    pub(super) fn spill_index(&mut self) -> Result<()> {
        self.healthy()?;
        let result = (|| {
            let start = self
                .end
                .checked_sub(self.pending.len() as u64)
                .ok_or(StoreError::Integrity("candidate pending origin"))?;
            let mut old = self
                .index
                .take()
                .ok_or(StoreError::Integrity("candidate index transfer"))?;
            let mut pending = std::mem::take(&mut self.pending_index);
            let mut disk = SpillDiskIndex::new()?;
            let mut page = Vec::with_capacity(OBJECT_PAGE_COUNT);
            while let Some((id, location)) = old.pop_first() {
                if let Some((relative, length)) = pending.remove(&id) {
                    let absolute = start
                        .checked_add(relative as u64)
                        .ok_or(StoreError::Integrity("candidate offset overflow"))?;
                    if location != (absolute, length as u64) {
                        return Err(StoreError::Integrity(
                            "unequal candidate location duplicate",
                        ));
                    }
                }
                page.push((id, location.0, location.1));
                if page.len() == OBJECT_PAGE_COUNT {
                    disk.insert_page(&page)?;
                    page.clear();
                }
            }
            // This includes the record that triggered spilling before it entered the old map.
            for (id, (relative, length)) in pending {
                let absolute = start
                    .checked_add(relative as u64)
                    .ok_or(StoreError::Integrity("candidate offset overflow"))?;
                if absolute
                    .checked_add(length as u64)
                    .is_none_or(|end| end > self.end)
                {
                    return Err(StoreError::Integrity("candidate pending range"));
                }
                page.push((id, absolute, length as u64));
                if page.len() == OBJECT_PAGE_COUNT {
                    disk.insert_page(&page)?;
                    page.clear();
                }
            }
            if !page.is_empty() {
                disk.insert_page(&page)?;
            }
            self.flush()?;
            self.index_bytes = 0;
            self.disk_index = Some(Box::new(disk));
            Ok(())
        })();
        if result.is_err() {
            self.failed = true;
        }
        result
    }

    pub(super) fn seal(&mut self) -> Result<()> {
        self.flush()?;
        self.writer = None;
        self.pending = Vec::new();
        #[cfg(unix)]
        std::fs::remove_file(&self.path)?;
        Ok(())
    }

    pub(super) fn flush(&mut self) -> Result<()> {
        self.healthy()?;
        if self.pending.is_empty() {
            return Ok(());
        }
        let result = (|| {
            let start = self
                .end
                .checked_sub(self.pending.len() as u64)
                .ok_or(StoreError::Integrity("candidate pending origin"))?;
            self.writer
                .as_mut()
                .ok_or(StoreError::Integrity("sealed candidate spool"))?
                .write_all(&self.pending)?;
            if let Some(disk) = self.disk_index.as_mut() {
                let mut page = Vec::with_capacity(OBJECT_PAGE_COUNT);
                for (&id, &(offset, length)) in &self.pending_index {
                    page.push((
                        id,
                        start
                            .checked_add(offset as u64)
                            .ok_or(StoreError::Integrity("candidate offset overflow"))?,
                        length as u64,
                    ));
                    if page.len() == OBJECT_PAGE_COUNT {
                        disk.insert_page(&page)?;
                        page.clear();
                    }
                }
                if !page.is_empty() {
                    disk.insert_page(&page)?;
                }
            }
            self.pending.clear();
            self.pending_index.clear();
            Ok(())
        })();
        if result.is_err() {
            self.failed = true;
        }
        result
    }

    pub(super) fn put(&mut self, object: &AuthenticatedCanonicalObject) -> Result<()> {
        let id = object.id;
        let canonical = &object.bytes;
        self.healthy()?;
        let row_len = canonical
            .len()
            .checked_add(184)
            .ok_or(StoreError::Integrity("candidate object length"))?;
        if !self.pending.is_empty()
            && self.pending.len().saturating_add(row_len) > self.buffer_bytes
        {
            self.flush()?;
        }
        let start = self.end;
        let pending_offset = self.pending.len() + 184;
        self.pending.extend_from_slice(id.as_bytes());
        self.pending
            .extend_from_slice(&(canonical.len() as u64).to_le_bytes());
        let mut hints = [0u8; 144];
        if let Some((start, len)) = object.1.first_span {
            hints[..8].copy_from_slice(&start.to_le_bytes());
            hints[8..12].copy_from_slice(&len.to_le_bytes());
        }
        hints[12] = u8::from(object.1.has_predecessor);
        for (slot, id) in object.1.prior_ids.iter().enumerate() {
            if let Some(id) = id {
                hints[16 + slot * 32..48 + slot * 32].copy_from_slice(id.as_bytes());
            }
        }
        self.pending.extend_from_slice(&hints);
        self.pending.extend_from_slice(canonical);
        self.pending_index
            .insert(id, (pending_offset, canonical.len()));
        self.end = self
            .end
            .checked_add(row_len as u64)
            .ok_or(StoreError::Integrity("candidate object length"))?;
        let index_limit = self.index_limit;
        if let Some(index) = &mut self.index {
            if self.index_bytes.saturating_add(64) > index_limit {
                self.spill_index()?;
            } else {
                index.insert(id, (start + 184, canonical.len() as u64));
                self.index_bytes += 64;
            }
        }
        // Disk index receives a page only when pending data flushes; pending locations serve reads now.
        if self.pending.len() >= self.buffer_bytes {
            self.flush()?;
        }
        Ok(())
    }

    pub(super) fn visit_ids(&self, visitor: &mut dyn FnMut(ObjectId) -> Result<()>) -> Result<()> {
        self.healthy()?;
        if let Some(index) = &self.index {
            if index.len() <= self.order_memory_bytes / std::mem::size_of::<(u64, ObjectId)>() {
                let mut order = index
                    .iter()
                    .map(|(id, (offset, _))| (*offset, *id))
                    .collect::<Vec<_>>();
                order.sort_unstable_by_key(|(offset, _)| *offset);
                for (_, id) in order {
                    visitor(id)?;
                }
                return Ok(());
            }
        }
        let mut file = self
            .reader
            .lock()
            .map_err(|_| StoreError::Integrity("candidate spool lock"))?;
        file.seek(SeekFrom::Start(0))?;
        let mut file = BufReader::with_capacity(self.buffer_bytes, &mut *file);
        let mut position = 0_u64;
        while position < self.end {
            if self.end - position < 184 {
                return Err(StoreError::Integrity("truncated candidate frame"));
            }
            let mut frame = [0; 184];
            file.read_exact(&mut frame)?;
            let length = u64::from_le_bytes(frame[32..40].try_into().expect("frame length"));
            position = position
                .checked_add(184)
                .and_then(|offset| offset.checked_add(length))
                .filter(|&end| end <= self.end)
                .ok_or(StoreError::Integrity("candidate frame bounds"))?;
            visitor(ObjectId::from_bytes(&frame[..32])?)?;
            file.seek_relative(
                i64::try_from(length)
                    .map_err(|_| StoreError::Integrity("candidate object length"))?,
            )?;
        }
        Ok(())
    }

    pub(super) fn visit_ordered(
        &self,
        order: &IdOrder,
        visitor: &mut dyn FnMut(ObjectId, &mut Vec<u8>, PhysicalHints) -> Result<()>,
    ) -> Result<()> {
        self.healthy()?;
        let mut file = self
            .reader
            .lock()
            .map_err(|_| StoreError::Integrity("candidate spool lock"))?;
        file.seek(SeekFrom::Start(0))?;
        let mut file = BufReader::with_capacity(self.buffer_bytes, &mut *file);
        let mut position = 0_u64;
        let mut canonical = Vec::new();
        order.visit_pages(|ids| {
            let locations = self.locations(ids)?;
            for (&id, location) in ids.iter().zip(locations) {
                let (offset, length) = location.ok_or(StoreError::MissingObject(id))?;
                let length = usize::try_from(length)
                    .map_err(|_| StoreError::Integrity("candidate object length"))?;
                if length > OBJECT_PAGE_BYTES
                    || offset
                        .checked_add(length as u64)
                        .is_none_or(|end| end > self.end)
                {
                    return Err(StoreError::InvalidInput("candidate object page"));
                }
                let distance = i64::try_from(
                    i128::from(
                        offset
                            .checked_sub(144)
                            .ok_or(StoreError::Integrity("candidate hint offset"))?,
                    ) - i128::from(position),
                )
                .map_err(|_| StoreError::Integrity("candidate object offset"))?;
                file.seek_relative(distance)?;
                let mut encoded_hints = [0; 144];
                file.read_exact(&mut encoded_hints)?;
                let start = u64::from_le_bytes(encoded_hints[..8].try_into().unwrap());
                let len = u32::from_le_bytes(encoded_hints[8..12].try_into().unwrap());
                if encoded_hints[12] > 1 || encoded_hints[13..16] != [0; 3] {
                    return Err(StoreError::Integrity("candidate hint flags"));
                }
                let mut hints = PhysicalHints {
                    first_span: (len != 0).then_some((start, len)),
                    has_predecessor: encoded_hints[12] != 0,
                    ..PhysicalHints::default()
                };
                for slot in 0..4 {
                    let bytes = &encoded_hints[16 + slot * 32..48 + slot * 32];
                    if bytes.iter().any(|byte| *byte != 0) {
                        hints.prior_ids[slot] = Some(ObjectId::from_bytes(bytes)?);
                    }
                }
                canonical.resize(length, 0);
                file.read_exact(&mut canonical)?;
                position = offset + length as u64;
                visitor(id, &mut canonical, hints)?;
            }
            Ok(())
        })
    }

    pub(super) fn locations(&self, ids: &[ObjectId]) -> Result<Vec<Option<(u64, u64)>>> {
        self.healthy()?;
        let start = self
            .end
            .checked_sub(self.pending.len() as u64)
            .ok_or(StoreError::Integrity("candidate pending origin"))?;
        let mut output = vec![None; ids.len()];
        let mut missing = Vec::new();
        let mut slots = Vec::new();
        for (slot, &id) in ids.iter().enumerate() {
            if let Some(&(offset, length)) = self.pending_index.get(&id) {
                output[slot] = Some((
                    start
                        .checked_add(offset as u64)
                        .ok_or(StoreError::Integrity("candidate offset overflow"))?,
                    length as u64,
                ));
            } else if let Some(index) = &self.index {
                output[slot] = index.get(&id).copied();
            } else {
                missing.push(id);
                slots.push(slot);
            }
        }
        if !missing.is_empty() {
            let disk = self
                .disk_index
                .as_ref()
                .ok_or(StoreError::Integrity("candidate spill index unavailable"))?;
            for (slot, location) in slots.into_iter().zip(disk.locations(&missing)?) {
                output[slot] = location;
            }
        }
        Ok(output)
    }

    pub(super) fn location(&self, id: ObjectId) -> Result<Option<(u64, u64)>> {
        Ok(self.locations(&[id])?.pop().expect("location slot"))
    }

    pub(super) fn get(&self, id: ObjectId) -> Result<Option<Vec<u8>>> {
        self.healthy()?;
        if let Some((offset, length)) = self.pending_index.get(&id) {
            return Ok(Some(self.pending[*offset..*offset + *length].to_vec()));
        }
        let Some((offset, length)) = self.location(id)? else {
            return Ok(None);
        };
        let mut file = self
            .reader
            .lock()
            .map_err(|_| StoreError::Integrity("candidate spool lock"))?;
        file.seek(SeekFrom::Start(offset))?;
        let mut bytes = vec![
            0;
            usize::try_from(length)
                .map_err(|_| StoreError::Integrity("candidate object length"))?
        ];
        file.read_exact(&mut bytes)?;
        Ok(Some(bytes))
    }

    pub(super) fn encoded_length(&self, id: ObjectId) -> Result<u64> {
        self.location(id)?
            .map(|(_, length)| length)
            .ok_or(StoreError::MissingObject(id))
    }
}

fn scratch_index(label: &str, schema: &str) -> Result<(Connection, TempPath)> {
    let (temporary, path) = temporary_file(label)?;
    let path = TempPath(path);
    drop(temporary);
    let connection = Connection::open(&path.0)?;
    // Derived private scratch, with the same bounded cache and no Store policy changes.
    connection.execute_batch(
        "PRAGMA journal_mode=OFF; PRAGMA synchronous=OFF;
        PRAGMA temp_store=FILE; PRAGMA cache_size=-4096; PRAGMA cache_spill=ON;
        PRAGMA mmap_size=0; PRAGMA locking_mode=EXCLUSIVE;",
    )?;
    connection.execute_batch(schema)?;
    Ok((connection, path))
}

impl SpillDiskIndex {
    fn new() -> Result<Self> {
        let (connection, path) = scratch_index("candidate-index",
            "CREATE TABLE offsets (id BLOB PRIMARY KEY CHECK(length(id)=32), offset INTEGER NOT NULL CHECK(offset>=0), length INTEGER NOT NULL CHECK(length>=0)) WITHOUT ROWID;")?;
        Ok(Self {
            connection: Mutex::new(connection),
            _path: path,
        })
    }

    fn insert_page(&mut self, entries: &[(ObjectId, u64, u64)]) -> Result<()> {
        if entries.is_empty() {
            return Ok(());
        }
        let connection = self
            .connection
            .get_mut()
            .map_err(|_| StoreError::Integrity("candidate index lock"))?;
        let count = sql_rows(connection, 3, 80, 8)?;
        let transaction = connection.transaction()?;
        for page in entries.chunks(count) {
            let sql = format!(
                "INSERT INTO offsets(id,offset,length) VALUES {}",
                vec!["(?,?,?)"; page.len()].join(",")
            );
            let mut values = Vec::with_capacity(page.len() * 3);
            for &(id, offset, length) in page {
                values.push(Value::Blob(id.as_bytes().to_vec()));
                values
                    .push(Value::Integer(i64::try_from(offset).map_err(|_| {
                        StoreError::Integrity("candidate index offset")
                    })?));
                values
                    .push(Value::Integer(i64::try_from(length).map_err(|_| {
                        StoreError::Integrity("candidate index length")
                    })?));
            }
            transaction.execute(&sql, params_from_iter(values))?;
        }
        transaction.commit()?;
        Ok(())
    }

    fn locations(&self, ids: &[ObjectId]) -> Result<Vec<Option<(u64, u64)>>> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| StoreError::Integrity("candidate index lock"))?;
        let count = sql_rows(&connection, 1, 80, 2)?;
        let mut rows = BTreeMap::new();
        for page in ids.chunks(count) {
            let sql = format!(
                "SELECT id,offset,length FROM offsets WHERE id IN ({})",
                vec!["?"; page.len()].join(",")
            );
            let mut statement = connection.prepare(&sql)?;
            for row in statement.query_map(
                params_from_iter(page.iter().map(|id| id.as_bytes().as_slice())),
                |row| {
                    Ok((
                        row.get::<_, Vec<u8>>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, i64>(2)?,
                    ))
                },
            )? {
                let (id, offset, length) = row?;
                rows.insert(
                    ObjectId::from_bytes(&id)?,
                    (
                        u64::try_from(offset)
                            .map_err(|_| StoreError::Integrity("candidate index offset"))?,
                        u64::try_from(length)
                            .map_err(|_| StoreError::Integrity("candidate index length"))?,
                    ),
                );
            }
        }
        Ok(ids.iter().map(|id| rows.get(id).copied()).collect())
    }
}

pub(super) fn temporary_file(label: &str) -> Result<(std::fs::File, PathBuf)> {
    static SERIAL: AtomicU64 = AtomicU64::new(0);
    let directory = std::env::temp_dir();
    for _ in 0..32 {
        let path = directory.join(format!(
            "layerfs-{label}-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        let mut options = std::fs::OpenOptions::new();
        options.create_new(true).read(true).write(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        match options.open(&path) {
            Ok(file) => return Ok((file, path)),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error.into()),
        }
    }
    Err(StoreError::Integrity("candidate temporary file"))
}
