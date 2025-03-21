// reader.rs
//
// Copyright (c) 2021-2025  Douglas P Lau
//
use crate::gis::Gis;
use crate::node::{M_NODE, Node, Root};
use loam::{Error, Id, Reader, Result};
use pointy::{BBox, Bounded, Float};
use serde::de::DeserializeOwned;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

/// RTree reader
///
/// Reads a `.loam` file containing [Gis] data.
///
/// [Gis]: gis/trait.Gis.html
pub struct RTree<F, G>
where
    F: Float + DeserializeOwned,
    G: Gis<F> + DeserializeOwned,
{
    /// Reader for file
    reader: Reader,

    _float: PhantomData<F>,
    _geom: PhantomData<G>,
}

/// Query iterator for RTree
struct RTreeQuery<'a, D, F, G>
where
    F: Float + DeserializeOwned,
    G: Gis<F, Data = D> + DeserializeOwned,
{
    /// RTree
    tree: &'a RTree<F, G>,

    /// Query bounding box
    bbox: BBox<F>,

    /// Work list of Id / height tuples in bounding box
    work: Vec<(Id, usize)>,

    /// Error, if any
    error: Option<Error>,

    _data: PhantomData<D>,
}

impl<D, F, G> Iterator for RTreeQuery<'_, D, F, G>
where
    F: Float + DeserializeOwned,
    G: Gis<F, Data = D> + DeserializeOwned,
{
    type Item = Result<G>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(err) = self.error.take() {
            return Some(Err(err));
        }
        while let Some((id, height)) = self.work.pop() {
            if height > 1 {
                match self.tree.reader.lookup::<Node<F>>(id) {
                    Ok(node) => {
                        let children = node.into_entries();
                        for child in children {
                            log::trace!("{height}: {:?}", child.bbox());
                            if child.bounded_by(self.bbox) {
                                self.work.push((child.id(), height - 1));
                            }
                        }
                    }
                    Err(e) => return Some(Err(e)),
                }
            } else {
                match self.tree.reader.lookup::<G>(id) {
                    Ok(geom) => return Some(Ok(geom)),
                    Err(e) => return Some(Err(e)),
                }
            }
        }
        None
    }
}

impl<'a, D, F, G> RTreeQuery<'a, D, F, G>
where
    F: Float + DeserializeOwned,
    G: Gis<F, Data = D> + DeserializeOwned,
{
    /// Create a new RTree query
    fn new(tree: &'a RTree<F, G>, bbox: BBox<F>) -> Self {
        let mut work = Vec::new();
        let mut error = None;
        match tree.reader.root() {
            Ok(id) => {
                match tree.reader.lookup::<Root<F>>(id) {
                    Ok(root) => {
                        let height = Node::<F>::height(root.n_elem());
                        log::trace!("root: {height}");
                        let node = root.into_node();
                        let children = node.into_entries();
                        work.reserve(height * M_NODE);
                        for child in children {
                            log::trace!("query: {bbox:?}");
                            if child.bounded_by(bbox) {
                                log::trace!("child: {:?}", child.bbox());
                                work.push((child.id(), height));
                            }
                        }
                    }
                    Err(e) => error = Some(e),
                };
            }
            Err(e) => error = Some(e),
        }
        Self {
            tree,
            bbox,
            work,
            error,
            _data: PhantomData,
        }
    }
}

impl<D, F, G> RTree<F, G>
where
    F: Float + DeserializeOwned,
    G: Gis<F, Data = D> + DeserializeOwned,
{
    /// Open an RTree `.loam` file for reading
    pub fn new<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let mut tmp = PathBuf::new();
        tmp.push(path);
        let path = tmp;
        let reader = Reader::new(path)?;
        Ok(Self {
            reader,
            _float: PhantomData,
            _geom: PhantomData,
        })
    }

    /// Query a bounding box
    ///
    /// Returns an iterator of all [Gis] items within the bounds.
    ///
    /// [Gis]: gis/trait.Gis.html
    pub fn query<'a>(
        &'a self,
        bbox: BBox<F>,
    ) -> impl Iterator<Item = Result<G>> + 'a
    where
        D: 'a,
    {
        RTreeQuery::new(self, bbox)
    }
}
