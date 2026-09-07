//! Disposable immutable bytes. One cache is shared by a daemon's schedulers;
//! distinct owner scopes keep authenticated acquisitions private.
use layerfs_content::ObjectId;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

const CACHE_BYTES: usize = 32 * 1024 * 1024;
// Charge both ordered indexes and allocation headers as well as payload bytes.
const ENTRY_BYTES: usize = 384;
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Key {
    Content(u64, ObjectId, u64),
    Name(u64, ObjectId, ObjectId, Vec<u8>),
    Page(u64, ObjectId, ObjectId, Vec<u8>),
}

impl Key {
    fn scope(&self) -> u64 {
        match self {
            Self::Content(scope, ..) | Self::Name(scope, ..) | Self::Page(scope, ..) => *scope,
        }
    }

    fn charge(&self) -> usize {
        ENTRY_BYTES
            + match self {
                Self::Name(_, _, _, name) | Self::Page(_, _, _, name) => 2 * name.len(),
                _ => 0,
            }
    }
}

pub struct ImmutableReadCache {
    limit: usize,
    state: Mutex<State>,
}

#[derive(Default)]
struct State {
    entries: BTreeMap<Key, (Arc<[u8]>, u64)>,
    oldest: BTreeMap<u64, Key>,
    next: u64,
    bytes: usize,
}

impl Default for ImmutableReadCache {
    fn default() -> Self {
        Self {
            limit: CACHE_BYTES,
            state: Mutex::new(State::default()),
        }
    }
}

impl ImmutableReadCache {
    /// Never reused, including across distinct daemon runtimes in one process.
    pub fn new_scope(&self) -> u64 {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        NEXT.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_add(1))
            .expect("immutable cache scope exhausted")
    }

    pub fn get(&self, scope: u64, root: ObjectId, offset: u64, len: usize) -> Option<Vec<u8>> {
        let end = offset.checked_add(u64::try_from(len).ok()?)?;
        let (bytes, start) = {
            let state = self.state.lock().ok()?;
            // ponytail: overlapping ranges scan backwards within one file;
            // normalize intervals only if measured fragmentation makes this costly.
            let (key, (bytes, _)) = state.entries.range(..=Key::Content(scope, root, offset))
                .rev()
                .take_while(|(key, _)| matches!(key, Key::Content(s, r, _) if *s == scope && *r == root))
                .find(|(key, (bytes, _))| matches!(key, Key::Content(_, _, start) if start.checked_add(bytes.len() as u64).is_some_and(|limit| end <= limit)))?;
            let Key::Content(_, _, start) = key else {
                return None;
            };
            (bytes.clone(), *start)
        };
        let start = usize::try_from(offset - start).ok()?;
        Some(bytes.get(start..start.checked_add(len)?)?.to_vec())
    }

    /// Optional admission: rejection or eviction never fails a filesystem read.
    pub fn insert(&self, scope: u64, root: ObjectId, offset: u64, bytes: Vec<u8>) -> bool {
        if offset.checked_add(bytes.len() as u64).is_none() {
            return false;
        }
        self.insert_key(Key::Content(scope, root, offset), bytes)
    }

    pub fn get_name(
        &self,
        scope: u64,
        namespace: ObjectId,
        directory: ObjectId,
        name: &[u8],
    ) -> Option<Vec<u8>> {
        let bytes = self
            .state
            .lock()
            .ok()?
            .entries
            .get(&Key::Name(scope, namespace, directory, name.to_vec()))?
            .0
            .clone();
        Some(bytes.to_vec())
    }

    /// Encoded authenticated positive inode facts only; partial pages never
    /// establish absence. Live mutations are applied by the owner on demand.
    pub fn insert_name(
        &self,
        scope: u64,
        namespace: ObjectId,
        directory: ObjectId,
        name: &[u8],
        bytes: Vec<u8>,
    ) -> bool {
        if name.is_empty() || name.len() > 255 {
            return false;
        }
        self.insert_key(Key::Name(scope, namespace, directory, name.to_vec()), bytes)
    }

    pub fn get_page(
        &self,
        scope: u64,
        namespace: ObjectId,
        directory: ObjectId,
        after: &[u8],
    ) -> Option<Vec<u8>> {
        let bytes = self
            .state
            .lock()
            .ok()?
            .entries
            .get(&Key::Page(scope, namespace, directory, after.to_vec()))?
            .0
            .clone();
        Some(bytes.to_vec())
    }

    /// Validated immutable metadata and its continuation, excluding prefetched
    /// content (charged separately) and current live-directory changes.
    pub fn insert_page(
        &self,
        scope: u64,
        namespace: ObjectId,
        directory: ObjectId,
        after: &[u8],
        bytes: Vec<u8>,
    ) -> bool {
        if after.len() > 255 {
            return false;
        }
        self.insert_key(
            Key::Page(scope, namespace, directory, after.to_vec()),
            bytes,
        )
    }

    fn insert_key(&self, key: Key, bytes: Vec<u8>) -> bool {
        let Some(charge) = bytes.len().checked_add(key.charge()) else {
            return false;
        };
        if bytes.is_empty() || charge > self.limit {
            return false;
        }
        let Ok(mut state) = self.state.lock() else {
            return false;
        };
        let Some(next) = state.next.checked_add(1) else {
            return false;
        };
        if state
            .entries
            .get(&key)
            .is_some_and(|(old, _)| old.len() >= bytes.len())
        {
            return true;
        }
        if let Some((old, stamp)) = state.entries.remove(&key) {
            state.bytes -= old.len() + key.charge();
            state.oldest.remove(&stamp);
        }
        // ponytail: FIFO eviction; add recency tracking only if measured reuse
        // is lost to churn. No acquisition or backing I/O occurs under this lock.
        while state.bytes + charge > self.limit {
            let Some((_, key)) = state.oldest.pop_first() else {
                return false;
            };
            if let Some((old, _)) = state.entries.remove(&key) {
                state.bytes -= old.len() + key.charge();
            }
        }
        state.next = next;
        state.bytes += charge;
        state.entries.insert(key.clone(), (Arc::from(bytes), next));
        state.oldest.insert(next, key);
        true
    }

    pub fn remove_scope(&self, scope: u64) {
        let Ok(mut state) = self.state.lock() else {
            return;
        };
        let State {
            entries,
            oldest,
            bytes,
            ..
        } = &mut *state;
        entries.retain(|key, (value, stamp)| {
            if key.scope() != scope {
                return true;
            }
            *bytes -= value.len() + key.charge();
            oldest.remove(stamp);
            false
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranges_scopes_replacement_and_eviction_are_bounded() {
        let cache = ImmutableReadCache {
            limit: 2 * (ENTRY_BYTES + 8),
            ..Default::default()
        };
        let first = cache.new_scope();
        let second = cache.new_scope();
        let root = ObjectId::for_bytes(b"immutable file");
        assert_ne!(first, second);
        assert!(cache.insert(first, root, 4, b"abcdefgh".to_vec()));
        assert_eq!(cache.get(first, root, 6, 3), Some(b"cde".to_vec()));
        assert_eq!(cache.get(first, root, 3, 3), None);
        assert_eq!(cache.get(first, root, 10, 3), None);
        assert_eq!(cache.get(first, root, u64::MAX, 2), None);
        assert_eq!(cache.get(second, root, 6, 3), None);
        assert_eq!(
            cache.get(first, ObjectId::for_bytes(b"new file"), 6, 3),
            None
        );
        assert!(!cache.insert(first, root, 0, vec![0; cache.limit]));
        assert!(!cache.insert(first, root, u64::MAX, vec![0]));
        assert!(cache.insert(second, root, 0, b"second".to_vec()));
        assert!(cache.insert(first, root, 4, b"abc".to_vec()));
        assert_eq!(cache.get(first, root, 4, 8), Some(b"abcdefgh".to_vec()));
        assert!(cache.insert(second, root, 0, b"second!!".to_vec()));
        assert!(cache.insert(second, root, 20, b"third!!!".to_vec()));
        assert_eq!(cache.get(first, root, 4, 8), None);
        assert_eq!(cache.get(second, root, 0, 8), Some(b"second!!".to_vec()));
        assert!(cache.state.lock().unwrap().bytes <= cache.limit);
        cache.remove_scope(first);
        assert_eq!(cache.get(second, root, 20, 8), Some(b"third!!!".to_vec()));
        cache.remove_scope(second);
        let state = cache.state.lock().unwrap();
        assert_eq!(state.bytes, 0);
        assert!(state.entries.is_empty() && state.oldest.is_empty());
    }

    #[test]
    fn names_share_the_budget_and_keep_namespace_and_directory_identity() {
        let cache = ImmutableReadCache {
            limit: 2 * (ENTRY_BYTES + 16),
            ..Default::default()
        };
        let scope = cache.new_scope();
        let root = ObjectId::for_bytes(b"namespace");
        let directory = ObjectId::for_bytes(b"directory");
        let other = ObjectId::for_bytes(b"other");
        assert!(cache.insert_name(scope, root, directory, b"name", b"inode".to_vec()));
        assert_eq!(
            cache.get_name(scope, root, directory, b"name"),
            Some(b"inode".to_vec())
        );
        assert_eq!(cache.get_name(scope + 1, root, directory, b"name"), None);
        assert_eq!(cache.get_name(scope, other, directory, b"name"), None);
        assert_eq!(cache.get_name(scope, root, other, b"name"), None);
        assert_eq!(cache.get_name(scope, root, directory, b"absent"), None);
        assert!(cache.insert(scope, other, 0, vec![1; 16]));
        assert!(cache.insert(scope, other, 16, vec![2; 16]));
        assert_eq!(cache.get_name(scope, root, directory, b"name"), None);
        assert_eq!(cache.get(scope, other, 16, 1), Some(vec![2]));
        assert!(cache.state.lock().unwrap().bytes <= cache.limit);
        cache.remove_scope(scope);
        assert_eq!(cache.state.lock().unwrap().bytes, 0);
    }

    #[test]
    fn pages_keep_cursor_identity_and_share_eviction_with_names_and_content() {
        let cache = ImmutableReadCache {
            limit: 2 * (ENTRY_BYTES + 16),
            ..Default::default()
        };
        let scope = cache.new_scope();
        let root = ObjectId::for_bytes(b"namespace");
        let directory = ObjectId::for_bytes(b"directory");
        assert!(cache.insert_page(scope, root, directory, b"", b"page1".to_vec()));
        assert_eq!(
            cache.get_page(scope, root, directory, b""),
            Some(b"page1".to_vec())
        );
        assert_eq!(cache.get_page(scope, root, directory, b"next"), None);
        assert_eq!(cache.get_page(scope + 1, root, directory, b""), None);
        assert_eq!(cache.get_page(scope, directory, root, b""), None);
        assert!(cache.insert_name(scope, root, directory, b"name", b"inode".to_vec()));
        assert!(cache.insert(scope, root, 0, vec![1; 16]));
        assert_eq!(cache.get_page(scope, root, directory, b""), None);
        assert_eq!(
            cache.get_name(scope, root, directory, b"name"),
            Some(b"inode".to_vec())
        );
        assert!(cache.state.lock().unwrap().bytes <= cache.limit);
        cache.remove_scope(scope);
        assert_eq!(cache.state.lock().unwrap().bytes, 0);
    }
}
