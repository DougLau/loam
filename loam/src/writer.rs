// writer.rs      Writer module.
//
// Copyright (c) 2021  Douglas P Lau
//
use crate::common::{checksum, Error, Id, Result, CRC_SZ, HEADER};
use bincode::Options;
use serde::Serialize;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;

/// Writer for __loam__ files
///
/// The writer can be used to create or append to an existing file.
pub struct Writer {
    file: File,
}

impl Writer {
    /// Create a new Writer
    pub fn new<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let mut file = OpenOptions::new()
            .create_new(true)
            .append(true)
            .open(path)?;
        let len = file.metadata()?.len();
        if len > 0 && len < 8 {
            return Err(Error::InvalidHeader);
        }
        if len == 0 {
            file.write_all(HEADER)?;
        }
        Ok(Self { file })
    }

    /// Push a chunk of data to the end of the file.
    ///
    /// # Returns
    /// `Id` chunk identifier
    pub fn push<D>(&mut self, data: &D) -> Result<Id>
    where
        D: Serialize,
    {
        let len = self.file.metadata()?.len();
        let id = Id::new(len);
        let options = bincode::DefaultOptions::new();
        let len = options.serialized_size(data)? as usize;
        let lenlen = options.serialized_size(&len)? as usize;
        let mut buf = Vec::with_capacity(lenlen + len + CRC_SZ);
        options.serialize_into(&mut buf, &len)?;
        options.serialize_into(&mut buf, &data)?;
        if let Some(checksum) = checksum(&buf) {
            buf.extend(&checksum.to_le_bytes());
        }
        self.file.write_all(&buf)?;
        Ok(id)
    }

    /// Add a checkpoint to the file.  The `Id` commonly points to the root of a
    /// tree of nodes.
    ///
    /// In order to be read back, a file must end with a checkpoint.
    pub fn checkpoint(&mut self, id: Id) -> Result<()> {
        self.push(&id.to_le_bytes())?;
        self.file.sync_data()?;
        Ok(())
    }
}
