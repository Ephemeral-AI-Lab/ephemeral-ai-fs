use super::{InodeMutation, LogicalCounters};
use crate::file::rope::build_bytes;
use crate::filesystem;
use crate::object::access::ObjectStore;
use crate::tree::inode::{InodeId, InodeKind, InodeRecordV1};
use crate::tree::metadata::{
    build_metadata_tree, replace_metadata_entry, MetadataCounters, MetadataEntryV1, MetadataKey,
    PortableMetadataV1,
};
use crate::{CanonicalPath, CoreError, CoreResult, ObjectId};
use std::io::{Cursor, Read};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContentChange {
    Write {
        path: String,
        bytes: Vec<u8>,
        mode: u32,
    },
    Splice {
        path: String,
        start: u64,
        delete_len: u64,
        replacement: Vec<u8>,
    },
    Mkdir {
        path: String,
        mode: u32,
    },
    Symlink {
        path: String,
        target: Vec<u8>,
    },
    HardLink {
        source: String,
        target: String,
    },
    Rename {
        source: String,
        target: String,
    },
    Remove {
        path: String,
    },
    SetMode {
        path: String,
        mode: u32,
    },
    SetMtime {
        path: String,
        seconds: i64,
        nanoseconds: u32,
    },
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ApplyCounters {
    pub cdc_bytes_scanned: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AppliedRoot {
    pub root_id: ObjectId,
    pub counters: ApplyCounters,
}

pub fn apply_changes<S: ObjectStore>(
    store: &mut S,
    base_root: ObjectId,
    changes: &[ContentChange],
    seed: [u8; 32],
) -> CoreResult<AppliedRoot> {
    let mut root = base_root;
    let mut cdc_bytes_scanned = 0_u64;
    for change in changes {
        let candidate = apply_change(store, root, change, seed)?;
        cdc_bytes_scanned = cdc_bytes_scanned
            .checked_add(candidate.counters().rope.cdc_bytes_scanned)
            .ok_or(CoreError::LengthOverflow)?;
        root = candidate.root();
    }
    Ok(AppliedRoot {
        root_id: root,
        counters: ApplyCounters { cdc_bytes_scanned },
    })
}

pub fn write_file<S: ObjectStore, R: Read>(
    store: &mut S,
    root: ObjectId,
    path: &CanonicalPath,
    bytes: R,
    mode: u32,
    seed: [u8; 32],
) -> CoreResult<filesystem::CandidateRoot> {
    let candidate = filesystem::replace_file(store, root, path, bytes, |store| {
        let inode = allocate_for_root(store, root, seed, path)?;
        Ok((inode, metadata(store, InodeKind::RegularFile, mode)?))
    })?;
    set_mode(store, candidate.root(), path, mode)?.after(candidate)
}

fn apply_change<S: ObjectStore>(
    store: &mut S,
    root: ObjectId,
    change: &ContentChange,
    seed: [u8; 32],
) -> CoreResult<filesystem::CandidateRoot> {
    Ok(match change {
        ContentChange::Write {
            path: value,
            bytes,
            mode,
        } => {
            let path = path(value)?;
            let candidate =
                filesystem::replace_file(store, root, &path, Cursor::new(bytes), |store| {
                    let inode = allocate_for_root(store, root, seed, &path)?;
                    Ok((inode, metadata(store, InodeKind::RegularFile, *mode)?))
                })?;
            set_mode(store, candidate.root(), &path, *mode)?.after(candidate)?
        }
        ContentChange::Splice {
            path: value,
            start,
            delete_len,
            replacement,
        } => filesystem::replace_range(
            store,
            root,
            &path(value)?,
            *start,
            *delete_len,
            Cursor::new(replacement),
        )?,
        ContentChange::Mkdir { path: value, mode } => {
            let metadata = metadata(store, InodeKind::Directory, *mode)?;
            let inode = allocate_for_root(store, root, seed, &path(value)?)?;
            filesystem::create_directory(
                store,
                root,
                &path(value)?,
                inode,
                metadata,
            )?
        }
        ContentChange::Symlink {
            path: value,
            target,
        } => {
            let metadata = metadata(store, InodeKind::Symlink, 0o777)?;
            let inode = allocate_for_root(store, root, seed, &path(value)?)?;
            filesystem::create_symlink(
                store,
                root,
                &path(value)?,
                inode,
                target.clone(),
                metadata,
            )?
        }
        ContentChange::HardLink { source, target } => {
            filesystem::hard_link(store, root, &path(source)?, &path(target)?)?
        }
        ContentChange::Rename { source, target } => {
            let mut counters = LogicalCounters::default();
            let (left, _) = filesystem::resolve_parent(store, root, &path(source)?, &mut counters)?;
            let (right, _) =
                filesystem::resolve_parent(store, root, &path(target)?, &mut counters)?;
            filesystem::rename(
                store,
                root,
                &path(source)?,
                &path(target)?,
                left.record.metadata_root,
                right.record.metadata_root,
            )?
        }
        ContentChange::Remove { path: value } => {
            filesystem::remove_path(store, root, &path(value)?)?
        }
        ContentChange::SetMode { path: value, mode } => {
            set_mode(store, root, &path(value)?, *mode)?
        }
        ContentChange::SetMtime {
            path: value,
            seconds,
            nanoseconds,
        } => set_mtime(store, root, &path(value)?, *seconds, *nanoseconds)?,
    })
}

pub fn set_mtime<S: ObjectStore>(
    store: &mut S,
    root: ObjectId,
    path: &CanonicalPath,
    seconds: i64,
    nanoseconds: u32,
) -> CoreResult<filesystem::CandidateRoot> {
    let mut counters = LogicalCounters::default();
    let resolved = filesystem::resolve(store, root, path, &mut counters)?;
    let portable = PortableMetadataV1 {
        permission_mode: 0,
        mtime_seconds: seconds,
        mtime_nanoseconds: nanoseconds,
    };
    let mtime = portable.mtime_bytes()?;
    let (mtime_root, _) = build_bytes(store, &mtime)?;
    let metadata_root = replace_metadata_entry(
        store,
        resolved.record.metadata_root,
        MetadataEntryV1 {
            key: MetadataKey::new("portable".to_owned(), b"mtime".to_vec())?,
            value_file_root: mtime_root.0,
        },
        &mut MetadataCounters::default(),
    )?;
    filesystem::apply_inode_mutations(
        store,
        root,
        [InodeMutation::Upsert {
            inode: resolved.inode,
            record: InodeRecordV1 {
                metadata_root,
                ..resolved.record
            },
        }],
    )
}

pub fn set_mode<S: ObjectStore>(
    store: &mut S,
    root: ObjectId,
    path: &CanonicalPath,
    mode: u32,
) -> CoreResult<filesystem::CandidateRoot> {
    let mut counters = LogicalCounters::default();
    let resolved = filesystem::resolve(store, root, path, &mut counters)?;
    let portable = PortableMetadataV1 {
        permission_mode: mode
            & if resolved.record.kind == InodeKind::Directory {
                0o1777
            } else {
                0o777
            },
        mtime_seconds: 0,
        mtime_nanoseconds: 0,
    };
    let mode = portable.mode_bytes(resolved.record.kind)?;
    let (mode_root, _) = build_bytes(store, &mode)?;
    let metadata_root = replace_metadata_entry(
        store,
        resolved.record.metadata_root,
        MetadataEntryV1 {
            key: MetadataKey::new("portable".to_owned(), b"mode".to_vec())?,
            value_file_root: mode_root.0,
        },
        &mut MetadataCounters::default(),
    )?;
    filesystem::apply_inode_mutations(
        store,
        root,
        [InodeMutation::Upsert {
            inode: resolved.inode,
            record: InodeRecordV1 {
                metadata_root,
                ..resolved.record
            },
        }],
    )
}

pub(super) fn metadata<S: ObjectStore>(
    store: &mut S,
    kind: InodeKind,
    mode: u32,
) -> CoreResult<ObjectId> {
    build_portable_metadata(store, kind, mode, 0, 0)
}

/// Eight exact metadata results owned by one construction attempt.
///
/// The cache must not outlive the private output store that receives misses.
const PORTABLE_METADATA_CACHE_CAPACITY: usize = 8;

pub struct PortableMetadataCache {
    entries: [Option<(InodeKind, PortableMetadataV1, ObjectId, ObjectId)>;
        PORTABLE_METADATA_CACHE_CAPACITY],
    next: usize,
}

impl Default for PortableMetadataCache {
    fn default() -> Self {
        Self {
            entries: [None; Self::CAPACITY],
            next: 0,
        }
    }
}

impl PortableMetadataCache {
    pub const CAPACITY: usize = PORTABLE_METADATA_CACHE_CAPACITY;

    pub fn entry_count(&self) -> usize {
        self.entries.iter().flatten().count()
    }

    /// Reuse a checked result from this construction when handing off final records.
    pub fn get_by_root(&self, kind: InodeKind, root: ObjectId) -> Option<PortableMetadataV1> {
        self.entries
            .iter()
            .flatten()
            .find_map(|(cached_kind, metadata, cached_root, _)| {
                (*cached_kind == kind && *cached_root == root).then_some(*metadata)
            })
    }

    /// Returns the canonical metadata root and whether it was reused.
    pub fn get_or_build<S: ObjectStore>(
        &mut self,
        store: &mut S,
        kind: InodeKind,
        mode: u32,
        mtime_seconds: i64,
        mtime_nanoseconds: u32,
    ) -> CoreResult<(ObjectId, bool)> {
        let portable = portable_metadata(kind, mode, mtime_seconds, mtime_nanoseconds);
        if let Some((_, _, root, _)) = self
            .entries
            .iter()
            .flatten()
            .find(|(cached_kind, cached, _, _)| *cached_kind == kind && *cached == portable)
        {
            return Ok((*root, true));
        }
        let mode_root = self
            .entries
            .iter()
            .flatten()
            .find_map(|(_, cached, _, mode)| {
                (cached.permission_mode == portable.permission_mode).then_some(*mode)
            });
        let (root, mode_root) = build_portable_metadata_value(store, kind, portable, mode_root)?;
        self.entries[self.next] = Some((kind, portable, root, mode_root));
        self.next = (self.next + 1) % Self::CAPACITY;
        Ok((root, false))
    }
}

#[doc(hidden)]
pub fn build_portable_metadata<S: ObjectStore>(
    store: &mut S,
    kind: InodeKind,
    mode: u32,
    mtime_seconds: i64,
    mtime_nanoseconds: u32,
) -> CoreResult<ObjectId> {
    build_portable_metadata_value(
        store,
        kind,
        portable_metadata(kind, mode, mtime_seconds, mtime_nanoseconds),
        None,
    )
    .map(|(root, _)| root)
}

fn portable_metadata(
    kind: InodeKind,
    mode: u32,
    mtime_seconds: i64,
    mtime_nanoseconds: u32,
) -> PortableMetadataV1 {
    PortableMetadataV1 {
        permission_mode: mode
            & if kind == InodeKind::Directory {
                0o1777
            } else {
                0o777
            },
        mtime_seconds,
        mtime_nanoseconds,
    }
}

fn build_portable_metadata_value<S: ObjectStore>(
    store: &mut S,
    kind: InodeKind,
    portable: PortableMetadataV1,
    mode_root: Option<ObjectId>,
) -> CoreResult<(ObjectId, ObjectId)> {
    let mode_bytes = portable.mode_bytes(kind)?;
    let mtime_bytes = portable.mtime_bytes()?;
    let mode_root = match mode_root {
        Some(root) => root,
        None => {
            let (mode, _) = build_bytes(store, &mode_bytes)?;
            mode.0
        }
    };
    let (mtime, _) = build_bytes(store, &mtime_bytes)?;
    let root = build_metadata_tree(
        store,
        &[
            MetadataEntryV1 {
                key: MetadataKey::new("portable".to_owned(), b"mode".to_vec())?,
                value_file_root: mode_root,
            },
            MetadataEntryV1 {
                key: MetadataKey::new("portable".to_owned(), b"mtime".to_vec())?,
                value_file_root: mtime.0,
            },
        ],
    )?;
    Ok((root, mode_root))
}

fn path(value: &str) -> CoreResult<CanonicalPath> {
    let path = CanonicalPath::new(value.strip_prefix('/').unwrap_or(value))?;
    if path.is_root() {
        Err(CoreError::RootMutation)
    } else {
        Ok(path)
    }
}

#[doc(hidden)]
fn allocate_for_root<S: ObjectStore>(store: &mut S, root: ObjectId, seed: [u8; 32], path: &CanonicalPath) -> CoreResult<InodeId> {
    match filesystem::namespace(store, root)?.scope {
        Some(scope) => store.allocate_inode_serial(scope).map(|serial| serial.inode_key()),
        None => Ok(allocated_inode(seed, path)),
    }
}

pub fn allocated_inode(seed: [u8; 32], path: &CanonicalPath) -> InodeId {
    allocated_inode_text(seed, path.as_str())
}

fn allocated_inode_text(seed: [u8; 32], path: &str) -> InodeId {
    let mut bytes = Vec::with_capacity(seed.len() + path.len());
    bytes.extend_from_slice(&seed);
    bytes.extend_from_slice(path.as_bytes());
    InodeId::allocate(ObjectId::for_bytes(&bytes).to_bytes(), 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[derive(Default)]
    struct MemoryStore {
        objects: BTreeMap<ObjectId, Vec<u8>>,
        puts: usize,
    }

    impl ObjectStore for MemoryStore {
        fn get(&self, id: ObjectId) -> CoreResult<Vec<u8>> {
            self.objects
                .get(&id)
                .cloned()
                .ok_or(CoreError::PathNotFound)
        }

        fn put(&mut self, canonical: &[u8]) -> CoreResult<ObjectId> {
            let id = ObjectId::for_bytes(canonical);
            self.objects.insert(id, canonical.to_vec());
            self.puts += 1;
            Ok(id)
        }
    }

    #[test]
    fn portable_metadata_cache_is_exact_normalized_and_bounded() {
        let mut store = MemoryStore::default();
        let mut cache = PortableMetadataCache::default();
        let (first, hit) = cache
            .get_or_build(&mut store, InodeKind::RegularFile, 0o100644, 1, 0)
            .unwrap();
        assert!(!hit);
        let puts = store.puts;
        assert_eq!(
            cache
                .get_or_build(&mut store, InodeKind::RegularFile, 0o644, 1, 0)
                .unwrap(),
            (first, true)
        );
        assert_eq!(store.puts, puts);

        for seconds in 2..=9 {
            assert!(
                !cache
                    .get_or_build(&mut store, InodeKind::RegularFile, 0o644, seconds, 0)
                    .unwrap()
                    .1
            );
        }
        assert_eq!(cache.entry_count(), PortableMetadataCache::CAPACITY);
        assert!(
            !cache
                .get_or_build(&mut store, InodeKind::RegularFile, 0o644, 1, 0)
                .unwrap()
                .1
        );
    }

    #[test]
    fn portable_metadata_cache_reuses_modes_with_unique_mtimes() {
        let mut store = MemoryStore::default();
        let mut uncached = MemoryStore::default();
        let mut cache = PortableMetadataCache::default();
        for seconds in 0..32 {
            let kind = if seconds % 2 == 0 {
                InodeKind::RegularFile
            } else {
                InodeKind::Directory
            };
            let puts = store.puts;
            let (root, hit) = cache
                .get_or_build(&mut store, kind, 0o100755, seconds, 123)
                .unwrap();
            assert!(!hit);
            assert_eq!(store.puts - puts, if seconds == 0 { 7 } else { 4 });
            assert_eq!(
                root,
                build_portable_metadata(&mut uncached, kind, 0o755, seconds, 123).unwrap()
            );
            assert_eq!(store.objects, uncached.objects);
            assert_eq!(
                cache.get_by_root(kind, root),
                Some(portable_metadata(kind, 0o755, seconds, 123))
            );
        }
        assert_eq!(cache.entry_count(), PortableMetadataCache::CAPACITY);

        for (kind, mode) in [
            (InodeKind::Directory, 0o41777),
            (InodeKind::RegularFile, 0o101777),
            (InodeKind::Symlink, 0o120777),
        ] {
            let root = cache.get_or_build(&mut store, kind, mode, 35, 0).unwrap().0;
            assert_eq!(
                root,
                build_portable_metadata(&mut uncached, kind, mode, 35, 0).unwrap()
            );
            assert_eq!(store.objects, uncached.objects);
        }

        // A cached regular-file mode must not bypass symlink or timestamp validation.
        let puts = store.puts;
        for (kind, nanos) in [
            (InodeKind::Symlink, 0),
            (InodeKind::RegularFile, 1_000_000_000),
        ] {
            assert!(cache
                .get_or_build(&mut store, kind, 0o755, 40, nanos)
                .is_err());
        }
        assert_eq!(store.puts, puts);

        // Evict every 0755 entry, including its retained mode root.
        for seconds in 40..48 {
            cache
                .get_or_build(&mut store, InodeKind::RegularFile, 0o644, seconds, 0)
                .unwrap();
        }
        let puts = store.puts;
        let (root, hit) = cache
            .get_or_build(&mut store, InodeKind::RegularFile, 0o755, 50, 0)
            .unwrap();
        assert!(!hit);
        assert_eq!(store.puts - puts, 7);
        assert_eq!(
            root,
            build_portable_metadata(&mut uncached, InodeKind::RegularFile, 0o755, 50, 0).unwrap()
        );
    }
}
