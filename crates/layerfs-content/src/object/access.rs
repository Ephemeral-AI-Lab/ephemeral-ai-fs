use crate::{decode_bytes_object, CoreError, CoreResult, ObjectId};

pub trait ObjectRead {
    fn get(&self, id: ObjectId) -> CoreResult<Vec<u8>>;

    fn with_authenticated_canonical<T, F>(&self, id: ObjectId, callback: F) -> CoreResult<T>
    where
        F: FnOnce(&[u8]) -> CoreResult<T>,
    {
        let bytes = self.get(id)?;
        if ObjectId::for_bytes(&bytes) != id {
            return Err(CoreError::IdentityMismatch);
        }
        callback(&bytes)
    }

    fn get_authenticated_batch<F>(&self, ids: &[ObjectId], mut callback: F) -> CoreResult<()>
    where
        F: FnMut(ObjectId, &[u8]) -> CoreResult<()>,
    {
        for id in ids {
            self.with_authenticated_canonical(*id, |canonical| {
                callback(*id, decode_bytes_object(canonical)?)
            })?;
        }
        Ok(())
    }

    fn get_authenticated_payload_lengths_batch<F>(
        &self,
        ids: &[ObjectId],
        mut callback: F,
    ) -> CoreResult<()>
    where
        F: FnMut(ObjectId, u32) -> CoreResult<()>,
    {
        self.get_authenticated_batch(ids, |id, payload| {
            let payload = crate::file::extent_codec::decode_chunk_payload(payload)?;
            callback(
                id,
                u32::try_from(payload.len()).map_err(|_| CoreError::LengthOverflow)?,
            )
        })
    }
}

pub trait ObjectStore {
    fn small_content_format(&self) -> bool {
        false
    }
    fn compact_namespace(&self) -> bool {
        false
    }
    fn allocate_inode_serial(
        &mut self,
        _scope: ObjectId,
    ) -> CoreResult<crate::tree::compact::InodeSerial> {
        Err(CoreError::Unsupported)
    }

    fn get(&self, id: ObjectId) -> CoreResult<Vec<u8>>;
    fn put(&mut self, canonical: &[u8]) -> CoreResult<ObjectId>;

    fn put_owned(&mut self, canonical: Vec<u8>) -> CoreResult<ObjectId> {
        self.put(&canonical)
    }

    /// Optional physical origin; the default preserves ordinary publication.
    #[doc(hidden)]
    fn put_tree_origin(
        &mut self,
        canonical: Vec<u8>,
        _origin: Option<ObjectId>,
    ) -> CoreResult<ObjectId> {
        self.put_owned(canonical)
    }

    /// Scope physical FILE provenance to a regular-file producer. Generic ropes
    /// also store metadata values; their payload chunks must remain unmarked.
    #[doc(hidden)]
    fn set_file_payload_context(&mut self, _enabled: bool) -> bool {
        false
    }

    #[doc(hidden)]
    fn put_file_payload(
        &mut self,
        canonical: Vec<u8>,
        _start: u64,
        _len: u32,
    ) -> CoreResult<ObjectId> {
        self.put_owned(canonical)
    }

    #[doc(hidden)]
    fn note_transient_owned_bytes(&mut self, _bytes: u64) -> CoreResult<()> {
        Ok(())
    }

    fn with_authenticated_canonical<T, F>(&self, id: ObjectId, callback: F) -> CoreResult<T>
    where
        F: FnOnce(&[u8]) -> CoreResult<T>,
    {
        let bytes = self.get(id)?;
        if ObjectId::for_bytes(&bytes) != id {
            return Err(CoreError::IdentityMismatch);
        }
        callback(&bytes)
    }
}

impl<T: ObjectStore> ObjectRead for T {
    fn get(&self, id: ObjectId) -> CoreResult<Vec<u8>> {
        ObjectStore::get(self, id)
    }

    fn with_authenticated_canonical<U, F>(&self, id: ObjectId, callback: F) -> CoreResult<U>
    where
        F: FnOnce(&[u8]) -> CoreResult<U>,
    {
        ObjectStore::with_authenticated_canonical(self, id, callback)
    }
}
