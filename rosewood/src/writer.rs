// writer.rs
//
// Copyright (c) 2021  Douglas P Lau
//
use crate::Result;
use loam::{Id, Writer};
use serde::Serialize;
use std::path::{Path, PathBuf};

/// Bulk writer for RTree
pub struct BulkWriter {
    /// Path to file
    path: PathBuf,

    /// Writer to temporary file
    writer: Writer,

    /// IDs of data in temporary file
    ids: Vec<Id>,
}

impl BulkWriter {
    /// Create a new bulk writer
    pub fn new<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let mut tmp = PathBuf::new();
        tmp.push(path);
        let path = tmp.clone();
        tmp.set_extension("tmp");
        let writer = Writer::new(&tmp)?;
        Ok(Self {
            path,
            writer,
            ids: vec![],
        })
    }

    /// Push data
    pub fn push<D>(&mut self, data: &D) -> Result<()>
    where
        D: Serialize,
    {
        let id = self.writer.push(data)?;
        self.ids.push(id);
        Ok(())
    }

    /// Build RTree from pushed items
    pub fn finish(&mut self) -> Result<()> {
        todo!();
    }
}
