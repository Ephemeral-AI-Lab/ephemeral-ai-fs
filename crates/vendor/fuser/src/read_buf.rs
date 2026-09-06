use std::mem::align_of;

use crate::ll::fuse_abi as abi;
#[cfg(not(target_os = "linux"))]
use crate::session::MAX_WRITE_SIZE;

// Before INIT Linux accepts its minimum receive size and sends only INIT.
#[cfg(target_os = "linux")]
pub(crate) const INIT_BUFFER_SIZE: usize = 8192;
#[cfg(not(target_os = "linux"))]
pub(crate) const INIT_BUFFER_SIZE: usize = MAX_WRITE_SIZE + 4096;

/// Includes writes/retrieve replies, ioctl page payloads, xattrs and name/header
/// room. Batch forget is sized by the kernel to the supplied receive capacity.
#[cfg(any(target_os = "linux", test))]
pub(crate) fn receive_size(
    max_write: u32,
    max_pages: u16,
    page_size: usize,
) -> std::io::Result<usize> {
    let page_bytes = usize::from(max_pages)
        .max(32)
        .checked_mul(page_size)
        .ok_or_else(|| std::io::Error::other("FUSE receive size overflow"))?;
    (max_write as usize)
        .max(page_bytes)
        .max(64 * 1024)
        .checked_add(4096)
        .ok_or_else(|| std::io::Error::other("FUSE receive size overflow"))
}

/// A buffer that provides an aligned sub-slice for FUSE operations.
///
/// This struct wraps a `Vec<u8>` and provides access to an aligned portion
/// of the buffer, ensuring proper alignment for `fuse_in_header`.
#[derive(Debug)]
pub(crate) struct FuseReadBuf {
    buffer: Vec<u8>,
}

impl FuseReadBuf {
    /// Creates a new `FuseReadBuf` with the requested usable buffer size.
    ///
    /// The actual buffer may be slightly larger to accommodate alignment requirements.
    pub(crate) fn new(size: usize) -> Self {
        Self {
            buffer: vec![0; size + align_of::<abi::fuse_in_header>() - 1],
        }
    }

    /// Returns a mutable reference to the aligned portion of the buffer.
    pub(crate) fn as_mut(&mut self) -> &mut [u8] {
        let alignment = align_of::<abi::fuse_in_header>();
        let off = (alignment - (self.buffer.as_ptr() as usize) % alignment) % alignment;
        let size = self.buffer.len() - (alignment - 1);
        &mut self.buffer[off..off + size]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aligned_buffer_preserves_the_full_requested_capacity() {
        for size in [8192, 64 * 1024 + 4096, 1024 * 1024 + 4096] {
            let mut buffer = FuseReadBuf::new(size);
            assert_eq!(buffer.as_mut().len(), size);
            assert_eq!(
                buffer.as_mut().as_ptr() as usize % align_of::<abi::fuse_in_header>(),
                0
            );
            assert_eq!(
                buffer.buffer.len(),
                size + align_of::<abi::fuse_in_header>() - 1
            );
        }
    }

    #[test]
    fn negotiated_capacity_covers_all_payload_classes_and_checked_sizes() {
        assert_eq!(
            receive_size(1024 * 1024, 256, 4096).unwrap(),
            1024 * 1024 + 4096
        );
        assert_eq!(receive_size(4096, 1, 4096).unwrap(), 128 * 1024 + 4096);
        assert_eq!(
            receive_size(4096, 1, 65536).unwrap(),
            2 * 1024 * 1024 + 4096
        );
        assert_eq!(
            receive_size(16 * 1024 * 1024, 4096, 4096).unwrap(),
            16 * 1024 * 1024 + 4096
        );
        assert!(receive_size(1, u16::MAX, usize::MAX).is_err());
    }
}
