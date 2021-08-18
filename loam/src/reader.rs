use crate::error::{Error, Result};
use bincode::Options;
use memmap2::Mmap;
use serde::Deserialize;
use std::convert::TryInto;
use std::fs::File;
use std::path::Path;

pub struct Reader {
    mmap: Mmap,
    len: usize,
}

const HEADER_SZ: usize = 8;
const CHECKPOINT_SZ: usize = 13;

impl Reader {
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

    pub fn root(&self) -> Result<u64> {
        if self.len >= HEADER_SZ + CHECKPOINT_SZ {
            let base = self.len - CHECKPOINT_SZ;
            if self.mmap[base] == 8 {
                // todo: check crc
                let buf = &self.mmap[base + 1..base + 9];
                let id = u64::from_le_bytes(buf.try_into().unwrap());
                return Ok(id);
            }
        }
        Err(Error::MissingCheckpoint)
    }

    pub fn lookup<'de, D: Deserialize<'de>>(&'de self, id: u64) -> Result<D> {
        let base = id as usize;
        if self.len >= HEADER_SZ + CHECKPOINT_SZ
            && base >= HEADER_SZ
            && base < self.len
        {
            let options = bincode::DefaultOptions::new().allow_trailing_bytes();
            let dlen: u64 = options.deserialize(&self.mmap[base..])?;
            let offset = options.serialized_size(&dlen)? as usize;
            let data = options.deserialize(&self.mmap[base + offset..])?;
            return Ok(data);
        }
        Err(Error::InvalidId)
    }
}
