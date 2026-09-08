//! Selected-path observer: existing product reader + exact content APIs only.
use layerfs_content::file::rope::{visit_extents, FileStateRoot};
use layerfs_content::object::access::ObjectRead;
use layerfs_content::{CanonicalPath, CoreError, CoreResult, ObjectId};
use layerfs_layerstack_store::{LayerStackStore, ObjectSource};
use serde_json::json;
use std::{cell::Cell, path::Path, time::Instant};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn unhex(s: &str) -> Result<Vec<u8>> {
    if s.len() % 2 != 0
        || !s
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err("lowercase hex required".into());
    }
    Ok((0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect::<std::result::Result<Vec<_>, _>>()?)
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
struct Bounded<'a> {
    store: &'a LayerStackStore,
    reads: Cell<u64>,
    bytes: Cell<u64>,
    started: Instant,
}
impl ObjectRead for Bounded<'_> {
    fn get(&self, id: ObjectId) -> CoreResult<Vec<u8>> {
        if self.reads.get() >= 4096 || self.started.elapsed().as_secs() >= 120 {
            return Err(CoreError::InvalidRecord("path proof read/time budget"));
        }
        let length = self
            .store
            .inspect_connection(|db| {
                db.query_row(
                    "SELECT canonical_length FROM objects WHERE object_id=?",
                    [id.as_bytes().as_slice()],
                    |r| r.get::<_, i64>(0),
                )
            })
            .map_err(|_| CoreError::InvalidRecord("path proof connection"))?
            .map_err(|_| CoreError::MissingObject)?;
        let length = u64::try_from(length)
            .map_err(|_| CoreError::InvalidRecord("negative canonical length"))?;
        if length > 131136 || self.bytes.get() + length > 16 * 1024 * 1024 {
            return Err(CoreError::InvalidRecord("path proof canonical byte budget"));
        }
        self.reads.set(self.reads.get() + 1);
        self.bytes.set(self.bytes.get() + length);
        let bytes = self
            .store
            .read_object(id)
            .map_err(|_| CoreError::InvalidRecord("product canonical read failed"))?;
        layerfs_content::authenticate_identity(&bytes, id)?;
        if bytes.len() as u64 != length {
            return Err(CoreError::InvalidRecord("path proof canonical length"));
        }
        Ok(bytes)
    }
}
fn main() {
    if let Err(e) = run() {
        eprintln!("path proof failed: {e}");
        std::process::exit(1);
    }
}
fn run() -> Result<()> {
    let a = std::env::args().skip(1).collect::<Vec<_>>();
    let [copy, commit, path_hex, file_length, target_id, offset] = a.as_slice() else {
        return Err("DISPOSABLE_STORE COMMIT33HEX PATHHEX FILE_LENGTH TARGETID32HEX OFFSET".into());
    };
    let path_bytes = unhex(path_hex)?;
    if path_bytes.is_empty()
        || path_bytes[0] == b'/'
        || path_bytes.contains(&0)
        || path_bytes
            .split(|b| *b == b'/')
            .any(|c| c.is_empty() || c == b"." || c == b"..")
    {
        return Err("unsafe path".into());
    }
    let path = CanonicalPath::from_bytes(&path_bytes)?;
    let file_length = file_length.parse::<u64>()?;
    let offset = offset.parse::<u64>()?;
    if !(4096..=8388608).contains(&file_length)
        || offset.checked_add(4096).is_none_or(|end| end > file_length)
    {
        return Err("range/file bound".into());
    }
    let target = ObjectId::from_bytes(&unhex(target_id)?)?;
    let commit = unhex(commit)?;
    if commit.len() != 33 || commit[0] != 0x12 {
        return Err("commit typed identity".into());
    }
    // Root exclusively supplies independent disposable copies. No source/snapshot constructor.
    let store = LayerStackStore::connect(Path::new(copy))?;
    let root = store.inspect_connection(|db| {
        db.query_row(
            "SELECT root_id FROM commits WHERE commit_id=?",
            [commit.as_slice()],
            |row| row.get::<_, Vec<u8>>(0),
        )
    })??;
    let root = ObjectId::from_bytes(&root)?;
    let bounded = Bounded {
        store: &store,
        reads: Cell::new(0),
        bytes: Cell::new(0),
        started: Instant::now(),
    };
    let (stat, _) = layerfs_content::filesystem::stat(&bounded, root, &path)?;
    if stat.kind != layerfs_content::tree::inode::InodeKind::RegularFile {
        return Err("selected path not regular".into());
    }
    let mut logical = 0u64;
    let mut descriptors = Vec::new();
    let mut selected = None;
    let (state, _) = visit_extents(&bounded, FileStateRoot(stat.content_root), |extents| {
        for extent in extents {
            if descriptors.len() >= 4096 {
                return Err(CoreError::InvalidRecord("path proof extent budget"));
            }
            let end = logical
                .checked_add(u64::from(extent.logical_length))
                .ok_or(CoreError::LengthOverflow)?;
            if extent.payload_object_id == target && offset >= logical && offset + 4096 <= end {
                selected = Some(
                    json!({"file_offset":logical,"source_offset":extent.source_offset,"logical_length":extent.logical_length,"range_source_offset":u64::from(extent.source_offset)+offset-logical}),
                );
            }
            descriptors.push(json!({"id":hex(extent.payload_object_id.as_bytes()),"file_offset":logical,"source_offset":extent.source_offset,"logical_length":extent.logical_length}));
            logical = end;
        }
        Ok(())
    })?;
    if state.logical_len != file_length || logical != file_length {
        return Err("retained file length mismatch".into());
    }
    let selected = selected.ok_or("selected target range not wholly within authenticated span")?;
    println!(
        "{}",
        json!({"schema":"issue88-depth-path-proof-v1","status":"PASS","root_id":hex(root.as_bytes()),"file_state_id":hex(stat.content_root.as_bytes()),"metadata_root":hex(stat.metadata_root.as_bytes()),"file_length_bytes":logical,"path_hex":path_hex,"target_id":target_id,"offset_bytes":offset,"range_length_bytes":4096,"selected_span":selected,"descriptors":descriptors,"structural_reads_count":bounded.reads.get(),"structural_canonical_bytes":bounded.bytes.get(),"observer_elapsed_ns":u64::try_from(bounded.started.elapsed().as_nanos())?,"scope":"exact product reader authenticates retained structural path and ordered extent spans; no payload read/digest in this helper"})
    );
    Ok(())
}
