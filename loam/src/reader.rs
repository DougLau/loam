// reader.rs    Reader module.
//
// Copyright (c) 2021  Douglas P Lau
//
use crate::common::{Error, Id, Result};
use bincode::Options;
use memmap2::Mmap;
use serde::Deserialize;
use std::fs::File;
use std::path::Path;

/// Reader for `loam` files
pub struct Reader {
    mmap: Mmap,
    len: usize,
}

const HEADER_SZ: usize = 8;
const CHECKPOINT_SZ: usize = 13;

impl Reader {
    /// Create a new Reader
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };
        let len = mmap.len();
        if len < HEADER_SZ {
            // todo: check header
            return Err(Error::InvalidHeader);
        }
        Ok(Reader { mmap, len })
    }

    /// Get the root `Id` from the last checkpoint.
    pub fn root(&self) -> Result<Id> {
        if self.len >= HEADER_SZ + CHECKPOINT_SZ {
            let base = self.len - CHECKPOINT_SZ;
            if self.mmap[base] == 8 {
                // todo: check crc
                let buf = &self.mmap[base + 1..base + 9];
                let id = Id::from_le_slice(buf).ok_or(Error::InvalidCheckpoint);
                return id;
            }
        }
        Err(Error::InvalidCheckpoint)
    }

    /// Lookup data for the given `Id`
    pub fn lookup<'de, D: Deserialize<'de>>(&'de self, id: Id) -> Result<D> {
        let base = id.as_usize();
        if self.len >= HEADER_SZ + CHECKPOINT_SZ
            && base >= HEADER_SZ
            && base < self.len
        {
            let options = bincode::DefaultOptions::new().allow_trailing_bytes();
            let dlen: Id = options.deserialize(&self.mmap[base..])?;
            let offset = options.serialized_size(&dlen)? as usize;
            let data = options.deserialize(&self.mmap[base + offset..])?;
            return Ok(data);
        }
        Err(Error::InvalidId)
    }
}
