use crate::error::Result;
use bincode::Options;
use crc32fast::Hasher;
use serde::Serialize;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;

pub struct Writer {
    file: File,
}

impl Writer {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file =
            OpenOptions::new().create(true).append(true).open(path)?;
        let len = file.metadata()?.len();
        assert!(len == 0 || len > 8);
        if len == 0 {
            file.write_all(b"loam0000")?;
        }
        Ok(Writer { file })
    }

    pub fn push<D: Serialize>(&mut self, data: &D) -> Result<u64> {
        let id = self.file.metadata()?.len();
        let options = bincode::DefaultOptions::new();
        let len = options.serialized_size(data)? as usize;
        let lenlen = options.serialized_size(&len)? as usize;
        let mut buf = Vec::with_capacity(lenlen + len + 4);
        options.serialize_into(&mut buf, &len)?;
        options.serialize_into(&mut buf, &data)?;
        let mut hasher = Hasher::new();
        hasher.update(&buf);
        let checksum = hasher.finalize();
        options
            .serialize_into(&mut buf, &checksum.to_le_bytes())
            .unwrap();
        self.file.write_all(&buf)?;
        Ok(id)
    }

    pub fn checkpoint(&mut self, id: u64) -> Result<u64> {
        self.push(&id.to_le_bytes())?;
        self.file.sync_data()?;
        Ok(id)
    }
}
