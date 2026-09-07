use crate::handles::Handles;
use crate::inode_table::InodeTable;
use crate::{Attr, Kind, NodeId, PortError, SharedPort};
use fuser::{FileAttr, FileHandle, FileType, INodeNo, ReplyEmpty};
use std::time::{Duration, UNIX_EPOCH};

pub(crate) const TTL: Duration = Duration::from_secs(1);
pub(crate) const O_TRUNC: i32 = 0o1000;
pub(crate) const O_ACCMODE: i32 = 0o3;
pub(crate) const O_WRONLY: i32 = 0o1;
pub(crate) const O_RDWR: i32 = 0o2;

#[derive(Clone)]
pub struct LayerFs {
    pub(crate) port: SharedPort,
    pub(crate) inodes: InodeTable,
    pub(crate) handles: std::sync::Arc<Handles>,
    pub(crate) uid: u32,
    pub(crate) gid: u32,
    pub(crate) stateless_open: bool,
}

impl LayerFs {
    pub(crate) fn dispatch(&self, future: impl std::future::Future<Output = ()> + Send + 'static) {
        if let Ok(runtime) = crate::live_runtime::LiveRuntime::shared() {
            runtime.scheduler().submit(future);
        }
        // On runtime creation failure, dropping the owned reply returns EIO.
    }

    pub fn new(port: SharedPort, uid: u32, gid: u32) -> Self {
        Self {
            port,
            inodes: InodeTable,
            handles: std::sync::Arc::new(Handles::default()),
            uid,
            gid,
            stateless_open: false,
        }
    }

    pub(crate) fn node(&self, ino: INodeNo) -> std::result::Result<NodeId, fuser::Errno> {
        self.inodes.node(ino.0).ok_or(fuser::Errno::ENOENT)
    }

    pub(crate) fn attr(&self, attr: Attr) -> std::result::Result<FileAttr, fuser::Errno> {
        let ino = self.inodes.kernel(attr.node);
        let time = if attr.mtime_seconds >= 0 {
            UNIX_EPOCH + Duration::new(attr.mtime_seconds as u64, attr.mtime_nanoseconds)
        } else {
            UNIX_EPOCH
        };
        Ok(FileAttr {
            ino: INodeNo(ino),
            size: attr.size,
            blocks: attr.size.div_ceil(512),
            atime: time,
            mtime: time,
            ctime: time,
            crtime: UNIX_EPOCH,
            kind: file_type(attr.kind),
            perm: attr.mode as u16,
            nlink: attr.links,
            uid: self.uid,
            gid: self.gid,
            rdev: 0,
            blksize: 4096,
            flags: 0,
        })
    }

    pub(crate) async fn open_handle_async(
        &self,
        node: NodeId,
        truncate: bool,
        writable: bool,
    ) -> std::result::Result<u64, fuser::Errno> {
        self.port
            .pin_async(node, truncate, writable)
            .await
            .map_err(errno)?;
        Ok(self.handles.insert(node, writable))
    }

    pub(crate) fn file_node(
        &self,
        ino: INodeNo,
        handle: FileHandle,
    ) -> Result<NodeId, fuser::Errno> {
        if self.stateless_open {
            self.node(ino)
        } else {
            self.handle(handle)
        }
    }

    pub(crate) async fn entry_async(
        &self,
        parent: NodeId,
        name: &[u8],
        operation: crate::KernelEntry,
    ) -> Result<(FileAttr, crate::KernelReferences), fuser::Errno> {
        let (attr, references) = if self.stateless_open {
            self.port
                .kernel_entry_async(parent, name, operation)
                .await
                .map_err(errno)?
        } else {
            let attr = match operation {
                crate::KernelEntry::Lookup => self.port.lookup_async(parent, name).await,
                crate::KernelEntry::Create { mode } => {
                    self.port.create_file_async(parent, name, mode).await
                }
                crate::KernelEntry::Mkdir { mode } => {
                    self.port.mkdir_async(parent, name, mode).await
                }
                crate::KernelEntry::Symlink { target } => {
                    self.port.symlink_async(parent, name, target).await
                }
                crate::KernelEntry::Link { node } => self.port.link_async(node, parent, name).await,
            }
            .map_err(errno)?;
            (attr, crate::KernelReferences::default())
        };
        Ok((self.attr(attr)?, references))
    }

    pub(crate) fn handle(&self, handle: FileHandle) -> std::result::Result<NodeId, fuser::Errno> {
        self.handles
            .get(handle.0)
            .map(|handle| handle.node)
            .ok_or(fuser::Errno::EBADF)
    }
}

pub(crate) fn empty_reply(result: std::result::Result<(), fuser::Errno>, reply: ReplyEmpty) {
    match result {
        Ok(()) => reply.ok(),
        Err(error) => reply.error(error),
    }
}

pub(crate) fn file_type(kind: Kind) -> FileType {
    match kind {
        Kind::File => FileType::RegularFile,
        Kind::Directory => FileType::Directory,
        Kind::Symlink => FileType::Symlink,
    }
}

pub(crate) fn errno(error: PortError) -> fuser::Errno {
    match error {
        PortError::NotFound => fuser::Errno::ENOENT,
        PortError::NotEmpty => fuser::Errno::ENOTEMPTY,
        PortError::Exists => fuser::Errno::EEXIST,
        PortError::NoSpace => fuser::Errno::ENOSPC,
        PortError::ReadOnly => fuser::Errno::EROFS,
        PortError::Busy => fuser::Errno::EBUSY,
        PortError::Invalid => fuser::Errno::EINVAL,
        PortError::Io => fuser::Errno::EIO,
    }
}
