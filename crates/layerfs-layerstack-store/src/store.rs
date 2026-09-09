use crate::{Result, StoreError};
use std::path::Path;

#[derive(Clone)]
pub struct LayerStackStore {
    pub(crate) db: crate::schema::StoreDb,
}

impl LayerStackStore {
    /// Promote a closed schema-7 Store without rewriting payloads or page layout.
    pub fn upgrade_format(path: impl AsRef<Path>) -> Result<()> {
        crate::schema::upgrade_format(path.as_ref())
    }

    pub fn create(path: impl AsRef<Path>) -> Result<Self> {
        Ok(Self {
            db: crate::schema::StoreDb::create(path)?,
        })
    }

    pub fn connect(path: impl AsRef<Path>) -> Result<Self> {
        Ok(Self {
            db: crate::schema::StoreDb::connect(path)?,
        })
    }

    /// Cumulative diagnostics shared by clones, including host read/admission threads.
    #[doc(hidden)]
    pub fn physical_storage_receipt(&self) -> crate::PhysicalStorageReceipt {
        self.db.physical_storage_receipt()
    }

    pub fn path(&self) -> &Path {
        self.db.path()
    }

    #[doc(hidden)]
    pub fn data_version(&self) -> Result<u64> {
        self.db.data_version()
    }

    #[doc(hidden)]
    pub fn ensure_writable(&self) -> Result<()> {
        self.db.writer().map(drop).map_err(|error| match error {
            StoreError::StoreBusy => StoreError::StoreBusy,
            error => error,
        })
    }

    #[doc(hidden)]
    pub fn inspect_connection<T>(
        &self,
        inspect: impl FnOnce(&rusqlite::Connection) -> T,
    ) -> Result<T> {
        let connection = self.db.reader()?;
        Ok(inspect(&connection))
    }
}
