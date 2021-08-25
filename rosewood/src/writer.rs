// writer.rs
//
// Copyright (c) 2021  Douglas P Lau
//
use crate::node::Node;
use crate::{Geometry, Result};
use loam::{Id, Reader, Writer};
use pointy::Float;
use serde::{de::DeserializeOwned, Serialize};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

/// RTree bulk writer
pub struct BulkWriter<D, F, G>
where
    F: Float + Serialize + DeserializeOwned,
    G: Geometry<F, Data = D> + Serialize + DeserializeOwned,
{
    /// Path to file
    path: PathBuf,

    /// Writer to temporary file
    writer: Writer,

    /// Reader for temporary file
    reader: Reader,

    /// IDs of data in temporary file
    ids: Vec<Id>,

    _data: PhantomData<D>,
    _float: PhantomData<F>,
    _geom: PhantomData<G>,
}

impl<D, F, G> BulkWriter<D, F, G>
where
    F: Float + Serialize + DeserializeOwned,
    G: Geometry<F, Data = D> + Serialize + DeserializeOwned,
{
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
        let reader = Reader::new_empty()?;
        Ok(Self {
            path,
            writer,
            reader,
            ids: vec![],
            _data: PhantomData,
            _float: PhantomData,
            _geom: PhantomData,
        })
    }

    /// Push geometry
    pub fn push(&mut self, geom: &G) -> Result<()> {
        let id = self.writer.push(geom)?;
        self.ids.push(id);
        Ok(())
    }

    /// Build RTree from pushed items
    pub fn finish(mut self) -> Result<()> {
        // finish writing to the temp file
        self.writer.checkpoint(Id::new(0))?;
        // open another file for the real tree
        let mut tmp = PathBuf::new();
        tmp.push(&self.path);
        tmp.set_extension("tmp2");
        self.writer = Writer::new(&tmp)?;
        // reopen the temp file for reading
        tmp.set_extension("tmp");
        self.reader = Reader::new(&tmp)?;
        let height = Node::<F>::height(self.ids.len());
        let ids = std::mem::take(&mut self.ids);
        let id = self.build_tree(height, ids)?;
        self.writer.checkpoint(id)?;
        let path = self.path;
        drop(self.writer);
        remove_tmp_file(&path)?;
        rename_tree(&path)?;
        Ok(())
    }

    /// Build an RTree of the given IDs
    fn build_tree(&mut self, height: usize, ids: Vec<Id>) -> Result<Id> {
        debug_assert!(ids.len() > 0);
        if height == 1 {
            self.build_leaf(&ids)
        } else {
            todo!();
        }
    }

    /// Build a leaf node
    fn build_leaf(&mut self, ids: &[Id]) -> Result<Id> {
        let mut leaf = Node::<F>::new();
        for id in ids {
            if id.is_valid() {
                let geom: G = self.reader.lookup(*id)?;
                let wid = self.writer.push(&geom)?;
                leaf.push(wid, geom.bbox())?;
            }
        }
        Ok(self.writer.push(&leaf)?)
    }
}

/// Remove the temporary file
fn remove_tmp_file(path: &Path) -> Result<()> {
    let mut tmp = PathBuf::new();
    tmp.push(path);
    tmp.set_extension("tmp");
    std::fs::remove_file(tmp)?;
    Ok(())
}

/// Rename tree file
fn rename_tree(path: &Path) -> Result<()> {
    let mut tmp2 = PathBuf::new();
    tmp2.push(path);
    tmp2.set_extension("tmp2");
    std::fs::rename(tmp2, path)?;
    Ok(())
}
