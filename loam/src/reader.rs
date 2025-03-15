// reader.rs    Reader module.
//
// Copyright (c) 2021-2025  Douglas P Lau
//
use crate::common::{CRC_SZ, Error, HEADER, Id, Result};
use bincode::Options;
use memmap2::{Mmap, MmapMut};
use serde::de::DeserializeOwned;
use std::fs::File;
use std::path::Path;

/// Reader for __loam__ files
pub struct Reader {
    /// Memory map of loam file
    mmap: Mmap,

    /// Length of memory map
    len: usize,
}

/// Size of checkpoint chunk in bytes
const CHECKPOINT_SZ: usize = 9 + CRC_SZ;

impl Reader {
    /// Create a new empty Reader
    pub fn new_empty() -> Result<Self> {
        let len = 1;
        let mmap = MmapMut::map_anon(len)?.make_read_only()?;
        Ok(Self { mmap, len })
    }

    /// Create a new Reader
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path)?;
        // This is safe as long as the file is not modified by another process.
        // Needless to say, don't do that!
        let mmap = unsafe { Mmap::map(&file)? };
        let len = mmap.len();
        if len >= HEADER.len() && HEADER == &mmap[..HEADER.len()] {
            Ok(Reader { mmap, len })
        } else {
            Err(Error::InvalidHeader)
        }
    }

    /// Get the root chunk `Id` from the last checkpoint.
    pub fn root(&self) -> Result<Id> {
        if self.len >= HEADER.len() + CHECKPOINT_SZ {
            let base = self.len - CHECKPOINT_SZ;
            let id = Id::from_usize(base);
            let bytes: [u8; 8] = self.lookup(id)?;
            return Ok(Id::from_le_bytes(bytes));
        }
        Err(Error::InvalidCheckpoint)
    }

    /// Lookup data for the given chunk `Id`
    pub fn lookup<D>(&self, id: Id) -> Result<D>
    where
        D: DeserializeOwned,
    {
        let base = id.to_usize();
        if self.len >= HEADER.len() + CHECKPOINT_SZ
            && base >= HEADER.len()
            && base < self.len
        {
            let options = bincode::DefaultOptions::new().allow_trailing_bytes();
            let dlen: u64 = options.deserialize(&self.mmap[base..])?;
            #[cfg(feature = "crc")]
            {
                let crcoff = base + dlen as usize + 1;
                let chunk = &self.mmap[base..crcoff];
                if let Some(checksum) = crate::common::checksum(chunk) {
                    let calced = &checksum.to_le_bytes()[..];
                    let stored = &self.mmap[crcoff..crcoff + CRC_SZ];
                    if calced != stored {
                        return Err(Error::InvalidCrc(id));
                    }
                }
            }
            let offset = options.serialized_size(&dlen)? as usize;
            return Ok(options.deserialize(&self.mmap[base + offset..])?);
        }
        Err(Error::InvalidId(id))
    }
}
