// writer.rs
//
// Copyright (c) 2021  Douglas P Lau
//
use crate::node::Node;
use crate::{Geometry, Result};
use loam::{Id, Reader, Writer};
use pointy::{BBox, Float};
use serde::{de::DeserializeOwned, Serialize};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

/// RTree bulk writer
pub struct BulkWriter<D, F, G>
where
    F: Float + Serialize + DeserializeOwned,
    G: Geometry<F, Data = D> + Serialize,
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
    G: Geometry<F, Data = D> + Serialize,
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
        let reader = Reader::new(&tmp)?; // we don't need this yet...
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
        let depth = Node::<F>::depth(self.ids.len());
        let ids = std::mem::take(&mut self.ids);
        self.build_tree(depth, ids)?;
        self.remove_tmp_file()?;
        self.rename_tree()?;
        Ok(())
    }

    /// Build an RTree of the given IDs
    fn build_tree(&mut self, depth: usize, ids: Vec<Id>) -> Result<()> {
        debug_assert!(ids.len() > 0);
        if depth == 1 {
            let mut bbox = BBox::<F>::default();
            let mut wids = Vec::with_capacity(ids.len());
            for id in ids {
                if id.is_valid() {
                    let geom = self.reader.lookup(id)?;
                    bbox.extend(&[geom]);
                    let wid = self.writer.push(&geom)?;
                    wids.push(wid);
                }
            }
            let leaf = Node::new_leaf(bbox, &wids);
            todo!();
        }
        todo!();
    }

    /// Remove the temporary file
    fn remove_tmp_file(&self) -> Result<()> {
        let mut tmp = PathBuf::new();
        tmp.push(&self.path);
        tmp.set_extension("tmp");
        std::fs::remove_file(tmp)?;
        Ok(())
    }

    /// Rename tree file
    fn rename_tree(&self) -> Result<()> {
        let mut tmp2 = PathBuf::new();
        tmp2.push(&self.path);
        tmp2.set_extension("tmp2");
        std::fs::rename(tmp2, &self.path)?;
        Ok(())
    }
}
