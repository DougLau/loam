// writer.rs
//
// Copyright (c) 2021  Douglas P Lau
//
use crate::node::Node;
use crate::{Geometry, Result};
use loam::{Id, Writer};
use pointy::Float;
use serde::Serialize;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

/// RTree bulk writer
pub struct BulkWriter<D, F, G>
where
    F: Float,
    G: Geometry<F, Data = D> + Serialize,
{
    /// Path to file
    path: PathBuf,

    /// Writer to temporary file
    tmp_file: Writer,

    /// Writer to RTree file
    tree_file: Writer,

    /// IDs of data in temporary file
    ids: Vec<Id>,

    _data: PhantomData<D>,
    _float: PhantomData<F>,
    _geom: PhantomData<G>,
}

impl<D, F, G> BulkWriter<D, F, G>
where
    F: Float,
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
        let tmp_file = Writer::new(&tmp)?;
        tmp.set_extension("tmp2");
        let tree_file = Writer::new(&tmp)?;
        Ok(Self {
            path,
            tmp_file,
            tree_file,
            ids: vec![],
            _data: PhantomData,
            _float: PhantomData,
            _geom: PhantomData,
        })
    }

    /// Push geometry
    pub fn push(&mut self, geom: &G) -> Result<()> {
        let id = self.tmp_file.push(geom)?;
        self.ids.push(id);
        Ok(())
    }

    /// Build RTree from pushed items
    pub fn finish(mut self) -> Result<()> {
        let depth = Node::depth(self.ids.len());
        let ids = std::mem::take(&mut self.ids);
        self.build_tree(depth, ids)?;
        self.remove_tmp_file()?;
        self.rename_tree()?;
        Ok(())
    }

    /// Build an RTree of the given IDs
    fn build_tree(&mut self, depth: usize, ids: Vec<Id>) -> Result<()> {
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
