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
    /// Path for file
    path: PathBuf,

    _float: PhantomData<F>,
    _geom: PhantomData<G>,
}

/// Query iterator for RTree
struct RTreeQuery<D, F, G>
where
    F: Float + DeserializeOwned,
    G: Gis<F, Data = D> + DeserializeOwned,
{
    /// RTree reader
    reader: Option<Reader>,

    /// Query bounding box
    bbox: BBox<F>,

    /// Work list of Id / height tuples in bounding box
    work: Vec<(Id, usize)>,

    /// Error, if any
    error: Option<Error>,

    _data: PhantomData<D>,
    _geom: PhantomData<G>,
}

impl<D, F, G> Iterator for RTreeQuery<D, F, G>
where
    F: Float + DeserializeOwned,
    G: Gis<F, Data = D> + DeserializeOwned,
{
    type Item = Result<G>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(err) = self.error.take() {
            return Some(Err(err));
        }
        let reader = self.reader.as_ref()?;
        while let Some((id, height)) = self.work.pop() {
            if height > 1 {
                match reader.lookup::<Node<F>>(id) {
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
                match reader.lookup::<G>(id) {
                    Ok(geom) => return Some(Ok(geom)),
                    Err(e) => return Some(Err(e)),
                }
            }
        }
        None
    }
}

impl<D, F, G> RTreeQuery<D, F, G>
where
    F: Float + DeserializeOwned,
    G: Gis<F, Data = D> + DeserializeOwned,
{
    /// Create a new RTree query
    fn new(tree: &RTree<F, G>, bbox: BBox<F>) -> Self {
        match Self::build(tree.path.as_path(), bbox) {
            Ok(query) => query,
            Err(e) => Self {
                reader: None,
                bbox,
                work: Vec::new(),
                error: Some(e),
                _data: PhantomData,
                _geom: PhantomData,
            },
        }
    }

    /// Build query
    fn build(path: &Path, bbox: BBox<F>) -> Result<Self> {
        let mut work = Vec::new();
        let reader = Reader::new(path)?;
        let id = reader.root()?;
        let root = reader.lookup::<Root<F>>(id)?;
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
        Ok(Self {
            reader: Some(reader),
            bbox,
            work,
            error: None,
            _data: PhantomData,
            _geom: PhantomData,
        })
    }
}

impl<D, F, G> RTree<F, G>
where
    F: Float + DeserializeOwned,
    G: Gis<F, Data = D> + DeserializeOwned,
{
    /// Open an RTree `.loam` file for reading
    pub fn new<P>(path: P) -> Self
    where
        P: AsRef<Path>,
    {
        let mut tmp = PathBuf::new();
        tmp.push(path);
        let path = tmp;
        Self {
            path,
            _float: PhantomData,
            _geom: PhantomData,
        }
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
