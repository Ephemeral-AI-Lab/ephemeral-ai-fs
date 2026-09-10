//! Supported, source-preserving physical compaction of an entire Store. All
//! candidate selection uses authenticated, already published product state.
use super::{
    pack, small_candidates, spill,
    whole::{self, Role},
    CoreReader,
};
use crate::{LayerStackStore, Result, StoreError};
use layerfs_content::{
    file::{content, rope},
    tree::inode::InodeKind,
    ObjectId,
};
use rusqlite::{Connection, OptionalExtension};
use std::{
    path::{Path, PathBuf},
    time::Instant,
};

#[derive(Clone, Copy, Debug)]
pub struct CompactionOptions {
    /// Budget for private work files, output construction and VACUUM output.
    /// Source storage and caller-owned caches are reported separately.
    pub temporary_byte_limit: u64,
}
impl Default for CompactionOptions {
    fn default() -> Self {
        Self {
            temporary_byte_limit: 4 * 1024 * 1024 * 1024,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct CompactionReceipt {
    pub published: bool,
    pub cleanup_complete: bool,
    pub directory_synced: bool,
    pub publication_notes: Vec<String>,
    pub source_allocated_bytes: u64,
    pub final_allocated_bytes: u64,
    pub final_apparent_bytes: u64,
    /// Peak of named owned work files, sampled at file-growth boundaries. A
    /// process resource sampler must additionally account for SQLite sort spills.
    pub named_temporary_peak_bytes: u64,
    pub original_objects_verified: u64,
    pub added_owners_verified: u64,
    pub whole_owners: u64,
    pub small_full: u64,
    pub small_prefix: u64,
    pub whole_full: u64,
    pub whole_prefix: u64,
    pub native_slices: u64,
    pub native_full: u64,
    pub candidate_trials: u64,
    pub max_depth: u64,
    pub max_canonical_closure: u64,
    pub max_encoded_closure: u64,
    pub inventory_ns: u64,
    pub owner_ns: u64,
    pub encoding_ns: u64,
    pub vacuum_ns: u64,
    pub verification_ns: u64,
    pub publication_ns: u64,
    pub total_ns: u64,
}

impl LayerStackStore {
    /// Write a self-contained compacted Store to a new destination. The source
    /// and all of its canonical identities remain unchanged. The destination is
    /// published only after complete canonical and logical-record verification.
    /// Future writes/Commits use the ordinary product path; compaction may be
    /// repeated to reselect bases from the then-published history.
    pub fn compact_into(
        &self,
        destination: impl AsRef<Path>,
        options: CompactionOptions,
    ) -> Result<CompactionReceipt> {
        let started = Instant::now();
        let _permit = self.db.enter_operation()?;
        if !self.db.compact_namespace() {
            return Err(StoreError::InvalidInput("compaction requires schema 10"));
        }
        if !self.db.reader()?.is_autocommit() {
            return Err(StoreError::Integrity("compaction during publication"));
        }
        let destination = destination.as_ref();
        let name = destination
            .file_name()
            .ok_or(StoreError::InvalidInput("compaction destination"))?;
        let parent = destination
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let parent = std::fs::canonicalize(parent)?;
        let destination = parent.join(name);
        if destination.symlink_metadata().is_ok() {
            return Err(StoreError::StoreAlreadyExists);
        }
        let pages = options.temporary_byte_limit / 4 / 4096;
        if pages < 256 || std::fs::metadata(self.path())?.len() > pages * 4096 {
            return Err(StoreError::InvalidInput("compaction temporary budget"));
        }
        let mut receipt = CompactionReceipt {
            source_allocated_bytes: allocated(self.path())?,
            ..Default::default()
        };
        let (file, working_path) = spill::temporary_file_in(&parent, "compact-working")?;
        drop(file);
        let working_path = spill::TempPath(working_path);
        std::fs::copy(self.path(), &working_path.0)?;
        let working = LayerStackStore::connect(&working_path.0)?;
        spill::configure_scratch(&*working.db.writer()?)?;
        let source_page_size: u64 =
            self.db
                .reader()?
                .pragma_query_value(None, "page_size", |row| row.get::<_, i64>(0))?
                as u64;
        let working_pages = pages * 4096 / source_page_size;
        working
            .db
            .writer()?
            .execute_batch(&format!("PRAGMA max_page_count={working_pages};"))?;
        let (work, work_path) = spill::scratch_index("compact-index", &format!(
            "PRAGMA page_size=4096; PRAGMA max_page_count={pages};
             CREATE TABLE content(id BLOB PRIMARY KEY,role INTEGER NOT NULL,length INTEGER NOT NULL,signature BLOB,data BLOB) WITHOUT ROWID;
             CREATE INDEX content_order ON content(role,length DESC,id);
             CREATE TABLE file_roots(id BLOB PRIMARY KEY) WITHOUT ROWID;
             CREATE TABLE slices(id BLOB PRIMARY KEY,owner BLOB NOT NULL,offset INTEGER NOT NULL,length INTEGER NOT NULL) WITHOUT ROWID;
             CREATE TABLE selected(id BLOB PRIMARY KEY,role INTEGER NOT NULL,depth INTEGER NOT NULL,canonical_work INTEGER NOT NULL,encoded_work INTEGER NOT NULL) WITHOUT ROWID;
             CREATE TABLE signatures(signature BLOB NOT NULL,id BLOB NOT NULL,PRIMARY KEY(signature,id)) WITHOUT ROWID;"))?;
        crate::schema::fail_transaction_statement(u64::MAX - 10)?;
        let checkpoint = Instant::now();
        inventory(self, &work)?;
        receipt.inventory_ns = super::elapsed_ns(checkpoint);
        let checkpoint = Instant::now();
        owners(self, &work, &mut receipt)?;
        receipt.owner_ns = super::elapsed_ns(checkpoint);
        sample(&mut receipt, &[&working_path.0, &work_path.0])?;
        let checkpoint = Instant::now();
        encode_content(self, &working, &work, &mut receipt)?;
        receipt.encoding_ns = super::elapsed_ns(checkpoint);
        crate::schema::fail_transaction_statement(u64::MAX - 11)?;
        sample(&mut receipt, &[&working_path.0, &work_path.0])?;
        working.db.writer()?.execute("DELETE FROM object_packs WHERE pack_id NOT IN (SELECT pack_id FROM objects UNION SELECT pack_id FROM metadata_value_groups)", [])?;
        // Apple SQLite requires an absent output, including when an existing
        // file has zero length. Reserve the private directory atomically instead.
        let final_directory = OutputDirectory::new(&parent)?;
        let final_path = spill::TempPath(final_directory.0.join("store.sqlite"));
        let checkpoint = Instant::now();
        working
            .db
            .writer()?
            .pragma_update(None, "page_size", 4096)?;
        working.db.writer()?.execute(
            "VACUUM main INTO ?1",
            [final_path
                .0
                .to_str()
                .ok_or(StoreError::InvalidInput("compaction path encoding"))?],
        )?;
        receipt.vacuum_ns = super::elapsed_ns(checkpoint);
        sample(
            &mut receipt,
            &[&working_path.0, &work_path.0, &final_path.0],
        )?;
        let compacted = LayerStackStore::connect(&final_path.0)?;
        let checkpoint = Instant::now();
        verify(self, &compacted, &work, &mut receipt)?;
        receipt.verification_ns = super::elapsed_ns(checkpoint);
        drop(compacted);
        drop(working);
        drop(work);
        crate::schema::fail_transaction_statement(u64::MAX - 12)?;
        let checkpoint = Instant::now();
        std::fs::set_permissions(&final_path.0, std::fs::metadata(self.path())?.permissions())?;
        std::fs::File::open(&final_path.0)?.sync_all()?;
        receipt.final_allocated_bytes = allocated(&final_path.0)?;
        receipt.final_apparent_bytes = std::fs::metadata(&final_path.0)?.len();
        // Atomic no-clobber publication. Both paths are on the same filesystem.
        std::fs::hard_link(&final_path.0, &destination).map_err(|error| {
            if error.kind() == std::io::ErrorKind::AlreadyExists {
                StoreError::StoreAlreadyExists
            } else {
                error.into()
            }
        })?;
        receipt.published = true;
        receipt.cleanup_complete = true;
        for path in [&final_path.0, &working_path.0, &work_path.0] {
            if let Err(error) = std::fs::remove_file(path) {
                receipt.cleanup_complete = false;
                receipt.publication_notes.push(format!(
                    "published; temporary cleanup failed for {}: {error}",
                    path.display()
                ));
            }
        }
        if let Err(error) = std::fs::remove_dir(&final_directory.0) {
            receipt.cleanup_complete = false;
            receipt.publication_notes.push(format!(
                "published; temporary directory cleanup failed: {error}"
            ));
        }
        let directory_sync = crate::schema::fail_transaction_statement(u64::MAX - 13)
            .and_then(|()| Ok(std::fs::File::open(&parent)?.sync_all()?));
        match directory_sync {
            Ok(()) => receipt.directory_synced = true,
            Err(error) => receipt.publication_notes.push(format!(
                "published; destination directory sync failed: {error}"
            )),
        }
        receipt.publication_ns = super::elapsed_ns(checkpoint);
        receipt.total_ns = super::elapsed_ns(started);
        Ok(receipt)
    }
}

struct OutputDirectory(PathBuf);
impl OutputDirectory {
    fn new(parent: &Path) -> Result<Self> {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT: AtomicU64 = AtomicU64::new(0);
        for _ in 0..32 {
            let path = parent.join(format!(
                "layerfs-compact-output-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            let mut builder = std::fs::DirBuilder::new();
            #[cfg(unix)]
            {
                use std::os::unix::fs::DirBuilderExt;
                builder.mode(0o700);
            }
            match builder.create(&path) {
                Ok(()) => return Ok(Self(path)),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => (),
                Err(error) => return Err(error.into()),
            }
        }
        Err(StoreError::Integrity("compaction private directory"))
    }
}
impl Drop for OutputDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn allocated(path: &Path) -> Result<u64> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        Ok(std::fs::metadata(path)?.blocks() * 512)
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        Err(StoreError::InvalidInput(
            "allocated storage measurement requires Unix",
        ))
    }
}
fn sample(receipt: &mut CompactionReceipt, paths: &[&PathBuf]) -> Result<()> {
    let bytes = paths.iter().try_fold(0u64, |sum, path| {
        Ok::<_, StoreError>(sum + allocated(path)?)
    })?;
    receipt.named_temporary_peak_bytes = receipt.named_temporary_peak_bytes.max(bytes);
    Ok(())
}

fn object_page(store: &LayerStackStore, after: &[u8]) -> Result<Vec<ObjectId>> {
    let connection = store.db.reader()?;
    let mut statement = connection
        .prepare("SELECT object_id FROM objects WHERE object_id>?1 ORDER BY object_id LIMIT 128")?;
    let rows = statement
        .query_map([after], |row| row.get::<_, Vec<u8>>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    rows.iter()
        .map(|bytes| Ok(ObjectId::from_bytes(bytes)?))
        .collect()
}

fn signature(raw: &[u8]) -> Vec<u8> {
    small_candidates::signature(raw)
        .iter()
        .flat_map(|hash| hash.to_le_bytes())
        .collect()
}
fn add_content(
    work: &Connection,
    id: ObjectId,
    role: Role,
    canonical: &[u8],
    owned: bool,
) -> Result<usize> {
    let raw = role.raw(canonical)?;
    Ok(work.execute(
        "INSERT OR IGNORE INTO content(id,role,length,signature,data) VALUES (?1,?2,?3,?4,?5)",
        rusqlite::params![
            id.as_bytes().as_slice(),
            role as i64,
            canonical.len() as i64,
            if role == Role::Native {
                None
            } else {
                Some(signature(raw))
            },
            owned.then_some(canonical)
        ],
    )?)
}

fn inventory(source: &LayerStackStore, work: &Connection) -> Result<()> {
    let mut after = Vec::new();
    loop {
        let ids = object_page(source, &after)?;
        if ids.is_empty() {
            break;
        }
        for id in &ids {
            let canonical = source.db.read_object_row(*id)?;
            let value = match layerfs_content::decode_bytes_object(&canonical) {
                Ok(value) => value,
                Err(_)
                    if canonical.get(4)
                        == Some(&(layerfs_content::ObjectKind::Directory as u8)) =>
                {
                    layerfs_content::decode_object(&canonical)?;
                    continue;
                }
                Err(error) => return Err(error.into()),
            };
            let role = match value.get(..8) {
                Some(b"LFS5SML\0") => Some(Role::Small),
                Some(b"LFSWFL1\0") => Some(Role::Whole),
                Some(b"LFS4CHK\0") => Some(Role::Native),
                _ => None,
            };
            if let Some(role) = role {
                add_content(work, *id, role, &canonical, false)?;
            }
            let root = |record: layerfs_content::tree::inode::InodeRecordV1| -> Result<()> {
                if record.kind == InodeKind::RegularFile {
                    work.execute(
                        "INSERT OR IGNORE INTO file_roots(id) VALUES (?1)",
                        [record.content_root.as_bytes().as_slice()],
                    )?;
                }
                Ok(())
            };
            match value.get(..8) {
                Some(b"LFS6INT\0") => {
                    if let layerfs_content::tree::compact::InodeNode::Leaf(rows) =
                        layerfs_content::tree::compact::decode_inode(&canonical)?
                    {
                        for (_, record) in rows {
                            root(record)?;
                        }
                    }
                }
                Some(b"LFS4INO\0") => root(
                    layerfs_content::tree::inode::codec::decode_inode_record(&canonical)?,
                )?,
                _ => (),
            }
        }
        after = ids.last().unwrap().as_bytes().to_vec();
    }
    Ok(())
}

fn owners(
    source: &LayerStackStore,
    work: &Connection,
    receipt: &mut CompactionReceipt,
) -> Result<()> {
    let mut after = Vec::new();
    loop {
        let Some(id) = work
            .query_row(
                "SELECT id FROM file_roots WHERE id>?1 ORDER BY id LIMIT 1",
                [&after],
                |row| row.get::<_, Vec<u8>>(0),
            )
            .optional()?
        else {
            break;
        };
        after = id.clone();
        let root = content::FileContentRoot(ObjectId::from_bytes(&id)?);
        let state = content::inspect(&CoreReader(source), root)?;
        let length = state.logical_len();
        if !(content::SMALL_LIMIT as u64..=content::WHOLE_LIMIT as u64).contains(&length) {
            continue;
        }
        let mut raw = Vec::with_capacity(length as usize);
        content::read_all_bounded(
            &CoreReader(source),
            root,
            content::WHOLE_LIMIT as u64,
            &mut raw,
        )?;
        if raw.len() as u64 != length {
            return Err(StoreError::Integrity("whole-file construction length"));
        }
        let canonical = Role::Whole.canonical(&raw)?;
        let owner = ObjectId::for_bytes(&canonical);
        add_content(work, owner, Role::Whole, &canonical, true)?;
        let mut offset = 0usize;
        let mut failure = None;
        let result = rope::visit_extents(
            &CoreReader(source),
            rope::FileStateRoot(root.0),
            |extents| {
                let result = (|| -> Result<()> {
                    for extent in extents {
                        let end = offset
                            .checked_add(extent.logical_length as usize)
                            .ok_or(StoreError::Integrity("whole-file slice position"))?;
                        let bytes = raw
                            .get(offset..end)
                            .ok_or(StoreError::Integrity("whole-file slice extent"))?;
                        // Only complete original native chunks become adapters. Partial
                        // extents and uncovered/oversized-file chunks retain FULLs.
                        if extent.source_offset == 0
                            && bytes.len() <= pack::NATIVE_RAW_LIMIT
                            && ObjectId::for_bytes(&Role::Native.canonical(bytes)?)
                                == extent.payload_object_id
                        {
                            work.execute("INSERT INTO slices(id,owner,offset,length) VALUES (?1,?2,?3,?4) ON CONFLICT(id) DO UPDATE SET owner=excluded.owner,offset=excluded.offset,length=excluded.length WHERE excluded.owner < slices.owner OR (excluded.owner=slices.owner AND excluded.offset<slices.offset)", rusqlite::params![extent.payload_object_id.as_bytes().as_slice(), owner.as_bytes().as_slice(), offset as i64, bytes.len() as i64])?;
                        }
                        offset = end;
                    }
                    Ok(())
                })();
                result.map_err(|error| {
                    failure = Some(error);
                    layerfs_content::CoreError::Io
                })
            },
        );
        if let Some(error) = failure {
            return Err(error);
        }
        result?;
        if offset != raw.len() {
            return Err(StoreError::Integrity("whole-file extent coverage"));
        }
    }
    receipt.whole_owners = work.query_row(
        "SELECT count(*) FROM content WHERE role=?1",
        [Role::Whole as i64],
        |row| row.get::<_, i64>(0),
    )? as u64;
    Ok(())
}

fn canonical(source: &LayerStackStore, work: &Connection, id: ObjectId) -> Result<Vec<u8>> {
    let bytes: Option<Vec<u8>> = work.query_row(
        "SELECT data FROM content WHERE id=?1",
        [id.as_bytes().as_slice()],
        |row| row.get(0),
    )?;
    let bytes = match bytes {
        Some(bytes) => bytes,
        None => source.db.read_object_row(id)?,
    };
    layerfs_content::authenticate_identity(&bytes, id)?;
    Ok(bytes)
}

#[derive(Default)]
struct PackWriter {
    records: Vec<Vec<u8>>,
    objects: Vec<(ObjectId, usize)>,
    bytes: usize,
}
impl PackWriter {
    fn push(
        &mut self,
        target: &LayerStackStore,
        id: ObjectId,
        canonical_length: usize,
        record: Vec<u8>,
    ) -> Result<()> {
        whole::record(&record, canonical_length)?;
        if !self.records.is_empty()
            && (self.records.len() == 256
                || 16 + 4 * (self.records.len() + 1) + self.bytes + record.len()
                    > whole::PACK_LIMIT)
        {
            self.flush(target)?;
        }
        self.bytes += record.len();
        self.records.push(record);
        self.objects.push((id, canonical_length));
        Ok(())
    }
    fn flush(&mut self, target: &LayerStackStore) -> Result<()> {
        if self.records.is_empty() {
            return Ok(());
        }
        let bytes = whole::assemble(&self.records)?;
        let mut connection = target.db.writer()?;
        let transaction = connection.transaction()?;
        let pack: i64 = transaction.query_row(
            "SELECT COALESCE(MAX(pack_id),0)+1 FROM object_packs",
            [],
            |row| row.get(0),
        )?;
        transaction.execute(
            "INSERT INTO object_packs(pack_id,data) VALUES (?1,?2)",
            rusqlite::params![pack, bytes],
        )?;
        for (number, (id, length)) in self.objects.iter().enumerate() {
            transaction.execute("INSERT INTO objects(object_id,canonical_length,pack_id,group_number,record_number) VALUES (?1,?2,?3,?4,0) ON CONFLICT(object_id) DO UPDATE SET canonical_length=excluded.canonical_length,pack_id=excluded.pack_id,group_number=excluded.group_number,record_number=0", rusqlite::params![id.as_bytes().as_slice(), *length as i64, pack, number as i64])?;
        }
        crate::schema::fail_transaction_statement(u64::MAX - 14)?;
        transaction.commit()?;
        self.records.clear();
        self.objects.clear();
        self.bytes = 0;
        Ok(())
    }
}

fn encode_content(
    source: &LayerStackStore,
    target: &LayerStackStore,
    work: &Connection,
    receipt: &mut CompactionReceipt,
) -> Result<()> {
    let mut packs = PackWriter::default();
    for role in [Role::Whole, Role::Small, Role::Native] {
        let mut encoder = if role == Role::Whole {
            pack::NativeEncoder::new_whole()?
        } else {
            pack::NativeEncoder::new_small()?
        };
        // Largest-first order makes every selected base already available at this
        // compaction step. Global bounded-disk signatures supply four candidates;
        // ranking uses shared min-hashes, then complete ObjectId, never Git/path data.
        let mut after_length = i64::MAX;
        let mut after_id = Vec::new();
        loop {
            let next = work.query_row("SELECT id,length,signature FROM content WHERE role=?1 AND (length<?2 OR (length=?2 AND id>?3)) ORDER BY length DESC,id LIMIT 1", rusqlite::params![role as i64, after_length, after_id], |row| Ok((row.get::<_, Vec<u8>>(0)?, row.get::<_, i64>(1)?, row.get::<_, Option<Vec<u8>>>(2)?))).optional()?;
            let Some((id_bytes, length, signature)) = next else {
                break;
            };
            after_id = id_bytes.clone();
            after_length = length;
            let id = ObjectId::from_bytes(&id_bytes)?;
            let canonical = canonical(source, work, id)?;
            if canonical.len() != length as usize {
                return Err(StoreError::Integrity("compaction canonical length"));
            }
            let raw = role.raw(&canonical)?;
            if role == Role::Native {
                let owner = work
                    .query_row(
                        "SELECT owner,offset,length FROM slices WHERE id=?1",
                        [&id_bytes],
                        |row| {
                            Ok((
                                row.get::<_, Vec<u8>>(0)?,
                                row.get::<_, i64>(1)?,
                                row.get::<_, i64>(2)?,
                            ))
                        },
                    )
                    .optional()?;
                let record = if let Some((owner, offset, count)) = owner {
                    if count as usize != raw.len() {
                        return Err(StoreError::Integrity("native slice length"));
                    }
                    receipt.native_slices += 1;
                    whole::slice(
                        ObjectId::from_bytes(&owner)?,
                        offset as usize,
                        count as usize,
                    )?
                } else {
                    receipt.native_full += 1;
                    whole::encode(role, raw.len(), None, &encoder.compress(raw, None)?)?
                };
                packs.push(target, id, canonical.len(), record)?;
                continue;
            }
            let full = encoder.compress(raw, None)?;
            let mut record = whole::encode(role, raw.len(), None, &full)?;
            let mut depth = 0usize;
            let mut canonical_work = canonical.len();
            let mut encoded_work = record.len();
            let signature =
                signature.ok_or(StoreError::Integrity("compaction signature missing"))?;
            let hashes = signature
                .chunks_exact(8)
                .filter(|hash| *hash != u64::MAX.to_le_bytes())
                .map(|hash| hash.to_vec())
                .collect::<Vec<_>>();
            if signature.len() != 64 {
                return Err(StoreError::Integrity("compaction signature length"));
            }
            if hashes.len() >= 2 && full.len() > 33 {
                let sql = format!("SELECT s.id,s.depth,s.canonical_work,s.encoded_work,count(*) AS overlap FROM signatures h JOIN selected s USING(id) WHERE s.role=? AND h.signature IN ({}) GROUP BY s.id HAVING count(*)>=2 ORDER BY overlap DESC,s.id LIMIT 4", vec!["?"; hashes.len()].join(","));
                let values = std::iter::once(rusqlite::types::Value::Integer(role as i64))
                    .chain(hashes.iter().cloned().map(rusqlite::types::Value::Blob));
                let rows = work
                    .prepare(&sql)?
                    .query_map(rusqlite::params_from_iter(values), |row| {
                        Ok((
                            row.get::<_, Vec<u8>>(0)?,
                            row.get::<_, i64>(1)?,
                            row.get::<_, i64>(2)?,
                            row.get::<_, i64>(3)?,
                        ))
                    })?
                    .collect::<rusqlite::Result<Vec<_>>>()?;
                for (base, base_depth, base_canonical, base_encoded) in rows {
                    if base_depth >= whole::EDGES as i64
                        || base_canonical as usize + canonical.len() > whole::CLOSURE
                    {
                        continue;
                    }
                    let base = ObjectId::from_bytes(&base)?;
                    let base_canonical_bytes = self::canonical(source, work, base)?;
                    receipt.candidate_trials += 1;
                    let prefix = encoder.compress(raw, Some(role.raw(&base_canonical_bytes)?))?;
                    let candidate = whole::encode(role, raw.len(), Some(base), &prefix)?;
                    if candidate.len() < record.len()
                        && base_encoded as usize + candidate.len() <= whole::CLOSURE
                    {
                        depth = base_depth as usize + 1;
                        canonical_work = base_canonical as usize + canonical.len();
                        encoded_work = base_encoded as usize + candidate.len();
                        record = candidate;
                    }
                }
            }
            match (role, depth == 0) {
                (Role::Whole, true) => receipt.whole_full += 1,
                (Role::Whole, false) => receipt.whole_prefix += 1,
                (Role::Small, true) => receipt.small_full += 1,
                (Role::Small, false) => receipt.small_prefix += 1,
                _ => unreachable!(),
            }
            receipt.max_depth = receipt.max_depth.max(depth as u64);
            receipt.max_canonical_closure =
                receipt.max_canonical_closure.max(canonical_work as u64);
            receipt.max_encoded_closure = receipt.max_encoded_closure.max(encoded_work as u64);
            work.execute("INSERT INTO selected(id,role,depth,canonical_work,encoded_work) VALUES (?1,?2,?3,?4,?5)", rusqlite::params![id_bytes, role as i64, depth as i64, canonical_work as i64, encoded_work as i64])?;
            for hash in hashes {
                work.execute(
                    "INSERT OR IGNORE INTO signatures(signature,id) VALUES (?1,?2)",
                    rusqlite::params![hash, id.as_bytes().as_slice()],
                )?;
            }
            packs.push(target, id, canonical.len(), record)?;
        }
        packs.flush(target)?;
    }
    Ok(())
}

fn logical_digest(store: &LayerStackStore) -> Result<blake3::Hash> {
    use rusqlite::types::ValueRef;
    let mut hash = blake3::Hasher::new();
    let connection = store.db.reader()?;
    for table in [
        "branches",
        "commits",
        "layer_stacks",
        "layers",
        "metadata_value_groups",
        "scope_allocator",
        "workspace_stages",
    ] {
        hash.update(table.as_bytes());
        let mut statement = connection.prepare(&format!("SELECT * FROM {table} ORDER BY 1"))?;
        let count = statement.column_count();
        let mut rows = statement.query([])?;
        while let Some(row) = rows.next()? {
            for index in 0..count {
                match row.get_ref(index)? {
                    ValueRef::Null => {
                        hash.update(&[0]);
                    }
                    ValueRef::Integer(value) => {
                        hash.update(&[1]);
                        hash.update(&value.to_le_bytes());
                    }
                    ValueRef::Real(value) => {
                        hash.update(&[2]);
                        hash.update(&value.to_bits().to_le_bytes());
                    }
                    ValueRef::Text(value) | ValueRef::Blob(value) => {
                        hash.update(&[if matches!(row.get_ref(index)?, ValueRef::Text(_)) {
                            3
                        } else {
                            4
                        }]);
                        hash.update(&(value.len() as u64).to_le_bytes());
                        hash.update(value);
                    }
                }
            }
            hash.update(&[255]);
        }
    }
    Ok(hash.finalize())
}

fn verify(
    source: &LayerStackStore,
    target: &LayerStackStore,
    work: &Connection,
    receipt: &mut CompactionReceipt,
) -> Result<()> {
    if target
        .db
        .reader()?
        .pragma_query_value(None, "page_size", |row| row.get::<_, i64>(0))?
        != 4096
    {
        return Err(StoreError::Integrity("compaction page layout"));
    }
    if logical_digest(source)? != logical_digest(target)? {
        return Err(StoreError::Integrity("compaction logical records changed"));
    }
    target.db.validate_metadata_groups()?;
    let mut after = Vec::new();
    loop {
        let ids = object_page(source, &after)?;
        if ids.is_empty() {
            break;
        }
        for id in &ids {
            if source.db.read_object_row(*id)? != target.db.read_object_row(*id)? {
                return Err(StoreError::Integrity("compaction canonical bytes changed"));
            }
            receipt.original_objects_verified += 1;
        }
        after = ids.last().unwrap().as_bytes().to_vec();
    }
    after.clear();
    loop {
        let next = work
            .query_row(
                "SELECT id,data FROM content WHERE data IS NOT NULL AND id>?1 ORDER BY id LIMIT 1",
                [&after],
                |row| Ok((row.get::<_, Vec<u8>>(0)?, row.get::<_, Vec<u8>>(1)?)),
            )
            .optional()?;
        let Some((id, bytes)) = next else {
            break;
        };
        after = id.clone();
        if target.db.read_object_row(ObjectId::from_bytes(&id)?)? != bytes {
            return Err(StoreError::Integrity("compaction owner bytes changed"));
        }
        receipt.added_owners_verified += 1;
    }
    if target.store_counts()?.objects
        != receipt.original_objects_verified + receipt.added_owners_verified
    {
        return Err(StoreError::Integrity("compaction object cardinality"));
    }
    target.reachable_storage()?;
    Ok(())
}
