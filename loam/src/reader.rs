// reader.rs    Reader module.
//
// Copyright (c) 2021  Douglas P Lau
//
use crate::common::{Error, Id, Result, CRC_SZ};
use bincode::Options;
use memmap2::Mmap;
use serde::Deserialize;
use std::fs::File;
use std::path::Path;

/// Reader for __loam__ files
pub struct Reader {
    /// Memory map of loam file
    mmap: Mmap,

    /// Length of memory map
    len: usize,
}

/// File header
const HEADER: &[u8; 8] = b"loam0000";

/// Size of checkpoint chunk in bytes
const CHECKPOINT_SZ: usize = 9 + CRC_SZ;

impl Reader {
    /// Create a new Reader
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path)?;
        // This is safe as long as the file is not modified by another process.
        // Needless to say, don't do that!
        let mmap = unsafe { Mmap::map(&file)? };
        let len = mmap.len();
        if len >= HEADER.len() {
            if &mmap[..HEADER.len()] == HEADER {
                return Ok(Reader { mmap, len });
            }
        }
        Err(Error::InvalidHeader)
    }

    /// Get the root chunk `Id` from the last checkpoint.
    pub fn root(&self) -> Result<Id> {
        if self.len >= HEADER.len() + CHECKPOINT_SZ {
            let base = self.len - CHECKPOINT_SZ;
            if let Some(cid) = Id::new(base as u64) {
                let bytes: [u8; 8] = self.lookup(cid)?;
                if let Some(id) = Id::from_le_bytes(bytes) {
                    return Ok(id);
                }
            }
        }
        Err(Error::InvalidCheckpoint)
    }

    /// Lookup data for the given chunk `Id`
    pub fn lookup<'de, D: Deserialize<'de>>(&'de self, id: Id) -> Result<D> {
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
