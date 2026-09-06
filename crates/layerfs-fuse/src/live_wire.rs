//! Live-owner transport values. These carry resolved facts, never host-side POSIX edits.
use layerfs_content::file::rope::FileStateRoot;
use layerfs_content::tree::directory::DirectoryStateRoot;
use layerfs_content::tree::inode::InodeId;
use layerfs_content::{CanonicalName, CanonicalPath, ObjectId};
use layerfs_workspace_core::backing::{BackingId, BackingRef};
use layerfs_workspace_core::file_edit::{Piece, PieceTree};
use layerfs_workspace_core::{Data, DirectoryData, FileData, Node, NodeId};
use std::io::{self, Read, Write};

pub const FACT_PAGE_BYTES: usize = 64 * 1024;
pub const FACT_PAGE_NODES: usize = 128;

pub const MAX_FRAME: usize = 1024 * 1024 + 64 * 1024;
pub const SEED: u8 = 1;
pub const LOOKUP: u8 = 2;
pub const RESERVE: u8 = 3;
pub const APPEND: u8 = 4;
pub const READ_BACKING: u8 = 5;
pub const READ_BASE: u8 = 6;
pub const CHECK: u8 = 7;
pub const RELEASE: u8 = 8;
pub const FACTS_BEGIN: u8 = 9;
pub const FACTS_NODE: u8 = 10;
pub const FACTS_END: u8 = 11;
pub const CANCEL_RESERVATION: u8 = 12;
pub const DIRECTORY_PAGE: u8 = 13;

pub fn invalid() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "live owner transport")
}
pub fn u64_out(out: &mut Vec<u8>, n: u64) {
    out.extend_from_slice(&n.to_be_bytes());
}
pub fn bytes_out(out: &mut Vec<u8>, bytes: &[u8]) -> io::Result<()> {
    let size = u32::try_from(bytes.len()).map_err(|_| invalid())?;
    out.extend_from_slice(&size.to_be_bytes());
    out.extend_from_slice(bytes);
    Ok(())
}
pub fn frame_out(out: &mut impl Write, bytes: &[u8]) -> io::Result<()> {
    if bytes.is_empty() || bytes.len() > MAX_FRAME {
        return Err(invalid());
    }
    out.write_all(&(bytes.len() as u32).to_be_bytes())?;
    out.write_all(bytes)
}
pub fn frame_in(input: &mut impl Read) -> io::Result<Vec<u8>> {
    let mut length = [0; 4];
    input.read_exact(&mut length)?;
    let length = u32::from_be_bytes(length) as usize;
    if length == 0 || length > MAX_FRAME {
        return Err(invalid());
    }
    let mut bytes = vec![0; length];
    input.read_exact(&mut bytes)?;
    Ok(bytes)
}
pub struct Input<'a>(pub &'a [u8]);
impl<'a> Input<'a> {
    pub fn raw(&mut self, count: usize) -> io::Result<&'a [u8]> {
        if count > self.0.len() {
            return Err(invalid());
        }
        let (value, rest) = self.0.split_at(count);
        self.0 = rest;
        Ok(value)
    }
    pub fn byte(&mut self) -> io::Result<u8> {
        Ok(self.raw(1)?[0])
    }
    pub fn u32(&mut self) -> io::Result<u32> {
        Ok(u32::from_be_bytes(self.raw(4)?.try_into().unwrap()))
    }
    pub fn u64(&mut self) -> io::Result<u64> {
        Ok(u64::from_be_bytes(self.raw(8)?.try_into().unwrap()))
    }
    pub fn object(&mut self) -> io::Result<ObjectId> {
        ObjectId::from_bytes(self.raw(32)?).map_err(|_| invalid())
    }
    pub fn bytes(&mut self) -> io::Result<&'a [u8]> {
        let n = self.u32()? as usize;
        self.raw(n)
    }
    pub fn done(self) -> io::Result<()> {
        if self.0.is_empty() {
            Ok(())
        } else {
            Err(invalid())
        }
    }
    fn count(&mut self, minimum: usize) -> io::Result<usize> {
        let count = self.u32()? as usize;
        if count > self.0.len() / minimum {
            return Err(invalid());
        }
        Ok(count)
    }
}

pub fn node_encoded_bound(node: &Node) -> io::Result<usize> {
    let paths = node
        .paths
        .iter()
        .try_fold(0usize, |n, path| n.checked_add(4 + path.len()))
        .ok_or_else(invalid)?;
    let data = match &node.data {
        Data::File(FileData::Edited { pieces, .. }) => {
            pieces.count().checked_mul(49).and_then(|n| {
                usize::try_from(pieces.inline_len())
                    .ok()
                    .and_then(|inline| n.checked_add(inline))
            })
        }
        Data::Directory(directory) => directory
            .changes
            .keys()
            .try_fold(0usize, |n, name| n.checked_add(12 + name.len())),
        Data::Symlink(target) => Some(target.len()),
        Data::File(_) => Some(0),
    }
    .ok_or_else(invalid)?;
    let capacity = 128usize
        .checked_add(paths)
        .and_then(|n| n.checked_add(data))
        .filter(|n| *n <= MAX_FRAME)
        .ok_or_else(invalid)?;
    Ok(capacity)
}

pub fn node_out(id: NodeId, node: &Node) -> io::Result<Vec<u8>> {
    let mut out = Vec::with_capacity(node_encoded_bound(node)?);
    u64_out(&mut out, id.0);
    u64_out(&mut out, node.revision);
    out.push(u8::from(node.canonical.is_some()));
    if let Some(inode) = node.canonical {
        out.extend_from_slice(inode.as_bytes());
    }
    out.extend_from_slice(&node.mode.to_be_bytes());
    out.extend_from_slice(&node.links.to_be_bytes());
    out.extend_from_slice(&node.pins.to_be_bytes());
    out.extend_from_slice(&node.mtime_seconds.to_be_bytes());
    out.extend_from_slice(&node.mtime_nanoseconds.to_be_bytes());
    out.extend_from_slice(&(u32::try_from(node.paths.len()).map_err(|_| invalid())?).to_be_bytes());
    for path in &node.paths {
        bytes_out(&mut out, path.as_bytes())?;
    }
    match &node.data {
        Data::File(FileData::Base { root, len }) => {
            out.push(0);
            out.extend_from_slice(root.0.as_bytes());
            u64_out(&mut out, *len);
        }
        Data::File(FileData::Edited {
            base,
            spool_high_water,
            pieces,
            edits,
        }) => {
            out.push(1);
            out.push(u8::from(base.is_some()));
            if let Some((root, len)) = base {
                out.extend_from_slice(root.0.as_bytes());
                u64_out(&mut out, *len);
            }
            u64_out(&mut out, *spool_high_water);
            out.extend_from_slice(&edits.to_be_bytes());
            out.extend_from_slice(
                &(u32::try_from(pieces.count()).map_err(|_| invalid())?).to_be_bytes(),
            );
            for piece in pieces.pieces() {
                match piece {
                    Piece::Base { root, offset, len } => {
                        out.push(0);
                        out.extend_from_slice(root.0.as_bytes());
                        u64_out(&mut out, offset);
                        u64_out(&mut out, len);
                    }
                    Piece::Spool {
                        segment,
                        offset,
                        len,
                    } => {
                        out.push(1);
                        u64_out(&mut out, segment.id().0);
                        u64_out(&mut out, offset);
                        u64_out(&mut out, len);
                    }
                    Piece::Zero { len } => {
                        out.push(2);
                        u64_out(&mut out, len);
                    }
                    Piece::Inline { bytes, offset, len } => {
                        out.push(3);
                        let start = usize::try_from(offset).map_err(|_| invalid())?;
                        let end = usize::try_from(offset.checked_add(len).ok_or_else(invalid)?)
                            .map_err(|_| invalid())?;
                        bytes_out(&mut out, bytes.get(start..end).ok_or_else(invalid)?)?;
                    }
                }
            }
        }
        Data::Directory(directory) => {
            out.push(2);
            out.push(u8::from(directory.base.is_some()));
            if let Some(base) = directory.base {
                out.extend_from_slice(base.0.as_bytes());
            }
            out.extend_from_slice(
                &(u32::try_from(directory.changes.len()).map_err(|_| invalid())?).to_be_bytes(),
            );
            for (name, child) in &directory.changes {
                bytes_out(&mut out, name)?;
                u64_out(&mut out, child.map_or(0, |id| id.0));
            }
        }
        Data::Symlink(target) => {
            out.push(3);
            bytes_out(&mut out, target)?;
        }
    }
    if out.len() > MAX_FRAME {
        return Err(invalid());
    }
    Ok(out)
}

pub fn node_in(
    bytes: &[u8],
    mut backing: impl FnMut(BackingId, u64, u64) -> io::Result<BackingRef>,
) -> io::Result<(NodeId, Node)> {
    if bytes.len() > MAX_FRAME {
        return Err(invalid());
    }
    let mut input = Input(bytes);
    let id = NodeId(input.u64()?);
    if id.0 == 0 {
        return Err(invalid());
    }
    let revision = input.u64()?;
    let canonical = match input.byte()? {
        0 => None,
        1 => Some(InodeId(input.raw(32)?.try_into().unwrap())),
        _ => return Err(invalid()),
    };
    let mode = input.u32()?;
    let links = input.u32()?;
    let pins = input.u32()?;
    let mtime_seconds = input.u64()? as i64;
    let mtime_nanoseconds = input.u32()?;
    if mtime_nanoseconds >= 1_000_000_000 {
        return Err(invalid());
    }
    let count = input.count(4)?;
    let mut paths = std::collections::BTreeSet::new();
    for _ in 0..count {
        let path = std::str::from_utf8(input.bytes()?).map_err(|_| invalid())?;
        CanonicalPath::new(path).map_err(|_| invalid())?;
        if !paths.insert(path.to_owned()) {
            return Err(invalid());
        }
    }
    let data = match input.byte()? {
        0 => Data::File(FileData::Base {
            root: FileStateRoot(input.object()?),
            len: input.u64()?,
        }),
        1 => {
            let base = match input.byte()? {
                0 => None,
                1 => Some((FileStateRoot(input.object()?), input.u64()?)),
                _ => return Err(invalid()),
            };
            let spool_high_water = input.u64()?;
            let edits = input.u32()?;
            if edits > layerfs_workspace_core::file_edit::MAX_EDITS_PER_FILE {
                return Err(invalid());
            }
            let count = input.count(5)?;
            if count > layerfs_workspace_core::file_edit::MAX_PIECES_PER_FILE {
                return Err(invalid());
            }
            let mut pieces = Vec::with_capacity(count);
            for _ in 0..count {
                pieces.push(match input.byte()? {
                    0 => Piece::Base {
                        root: FileStateRoot(input.object()?),
                        offset: input.u64()?,
                        len: input.u64()?,
                    },
                    1 => {
                        let id = BackingId(input.u64()?);
                        let offset = input.u64()?;
                        let len = input.u64()?;
                        offset.checked_add(len).ok_or_else(invalid)?;
                        Piece::Spool {
                            segment: backing(id, offset, len)?,
                            offset,
                            len,
                        }
                    }
                    2 => Piece::Zero { len: input.u64()? },
                    3 => {
                        let bytes: std::sync::Arc<[u8]> = input.bytes()?.into();
                        let len = bytes.len() as u64;
                        Piece::Inline {
                            bytes,
                            offset: 0,
                            len,
                        }
                    }
                    _ => return Err(invalid()),
                });
            }
            Data::File(FileData::Edited {
                base,
                spool_high_water,
                edits,
                pieces: PieceTree::empty()
                    .replace(0, 0, pieces)
                    .map_err(|_| invalid())?,
            })
        }
        2 => {
            let base = match input.byte()? {
                0 => None,
                1 => Some(DirectoryStateRoot(input.object()?)),
                _ => return Err(invalid()),
            };
            let count = input.count(12)?;
            let mut changes = std::collections::BTreeMap::new();
            for _ in 0..count {
                let name = input.bytes()?;
                CanonicalName::from_bytes(name).map_err(|_| invalid())?;
                let node = input.u64()?;
                if changes
                    .insert(name.to_vec(), (node != 0).then_some(NodeId(node)))
                    .is_some()
                {
                    return Err(invalid());
                }
            }
            Data::Directory(DirectoryData { base, changes })
        }
        3 => {
            let target = input.bytes()?;
            if target.len() > 4096 || target.contains(&0) {
                return Err(invalid());
            }
            Data::Symlink(target.to_vec())
        }
        _ => return Err(invalid()),
    };
    input.done()?;
    Ok((
        id,
        Node {
            revision,
            canonical,
            paths,
            mode,
            links,
            pins,
            mtime_seconds,
            mtime_nanoseconds,
            data,
        },
    ))
}

pub const FREEZE: u8 = 32;
pub const RESUME: u8 = 33;
pub const OBSERVE: u8 = 34;
pub const INSTALL_BEGIN: u8 = 35;
pub const INSTALL_NODE: u8 = 36;
pub const INSTALL_END: u8 = 37;
pub const SHUTDOWN: u8 = 38;
pub const WRITE_METRICS: u8 = 39;
pub const READ_METRICS: u8 = 40;
pub const INVALIDATE: u8 = 41;

pub const EDIT_BEGIN: u8 = 42;
pub const EDIT_PART: u8 = 43;
pub const EDIT_END: u8 = 44;
