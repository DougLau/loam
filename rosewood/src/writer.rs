// writer.rs
//
// Copyright (c) 2021  Douglas P Lau
//
use crate::node::{Node, Root};
use crate::{Geometry, Result};
use loam::{Id, Reader, Writer};
use pointy::{BBox, Float};
use serde::{de::DeserializeOwned, Serialize};
use std::cmp::Ordering;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

/// File element
struct Elem<F>
where
    F: Float + Serialize + DeserializeOwned,
{
    id: Id,
    bbox: BBox<F>,
}

impl<F> Elem<F>
where
    F: Float + Serialize + DeserializeOwned,
{
    fn compare_x(&self, rhs: &Self) -> Ordering {
        self.bbox
            .x_mid()
            .partial_cmp(&rhs.bbox.x_mid())
            .unwrap_or(Ordering::Equal)
    }
    fn compare_y(&self, rhs: &Self) -> Ordering {
        self.bbox
            .y_mid()
            .partial_cmp(&rhs.bbox.y_mid())
            .unwrap_or(Ordering::Equal)
    }
}

/// Node file element
enum NodeElem<F>
where
    F: Float + Serialize + DeserializeOwned,
{
    /// Leaf node
    Leaf(Node<F>),

    /// Non-leaf node with back-ref indices into nodes vector
    Node(Vec<usize>),
}

impl<F> NodeElem<F>
where
    F: Float + Serialize + DeserializeOwned,
{
    fn lookup(&self, elems: &[Elem<F>]) -> Node<F> {
        match self {
            NodeElem::Leaf(leaf) => leaf.clone(),
            NodeElem::Node(children) => {
                let mut n = Node::new();
                for child in children {
                    let Elem { id, bbox } = elems[*child];
                    n.push(id, bbox);
                }
                n
            }
        }
    }
}

/// RTree bulk writer
///
/// This writes a 2-dimensional RTree into a `loam` file, using the [OMT] bulk
/// loading algorithm.
///
/// The file is written in two steps:
///
/// 1. All `Geometry` values, grouped by leaf node in order to reduce page
///    faults when reading.
/// 2. All `Node` values, in depth-first order, with the root appearing last.
///
/// [OMT]: http://ceur-ws.org/Vol-74/files/FORUM_18.pdf
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

    /// Geometry elements
    elems: Vec<Elem<F>>,

    /// Node elements
    ///
    /// This is built during the first step (while writing `Geometry`), and
    /// used during the second step to write out `Node` data
    nodes: Vec<NodeElem<F>>,

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
            elems: vec![],
            nodes: vec![],
            _data: PhantomData,
            _float: PhantomData,
            _geom: PhantomData,
        })
    }

    /// Push geometry
    pub fn push(&mut self, geom: &G) -> Result<()> {
        let id = self.writer.push(geom)?;
        let bbox = geom.bbox();
        self.elems.push(Elem { id, bbox });
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
        let mut elems = std::mem::take(&mut self.elems);
        self.build_root(&mut elems)?;
        let id = self.write_nodes(elems.len())?;
        self.writer.checkpoint(id)?;
        let path = self.path;
        drop(self.writer);
        remove_tmp_file(&path)?;
        rename_tree(&path)?;
        Ok(())
    }

    /// Build the root node
    fn build_root(&mut self, elems: &mut [Elem<F>]) -> Result<usize> {
        let n_elems = elems.len();
        let height = Node::<F>::height(n_elems);
        if height > 1 {
            elems.sort_unstable_by(Elem::compare_x);
            let groups = Node::<F>::root_groups(n_elems);
            assert!(groups > 0);
            let n_group = (n_elems as f32 / groups as f32).ceil() as usize;
            let mut children = vec![];
            for v_chunk in elems.chunks_mut(n_group) {
                v_chunk.sort_unstable_by(Elem::compare_y);
                let n_chunk =
                    (v_chunk.len() as f32 / groups as f32).ceil() as usize;
                for h_chunk in v_chunk.chunks_mut(n_chunk) {
                    let child = self.build_tree_x(height - 1, h_chunk)?;
                    children.push(child);
                }
            }
            Ok(self.push_node(NodeElem::Node(children)))
        } else {
            self.build_leaf(elems)
        }
    }

    /// Build a tree (or sub-tree) by partitioning in X dimension
    fn build_tree_x(
        &mut self,
        height: usize,
        elems: &mut [Elem<F>],
    ) -> Result<usize> {
        if height > 1 {
            elems.sort_unstable_by(Elem::compare_x);
            let mut children = vec![];
            let n_group = Node::<F>::partition_sz(height);
            for chunk in elems.chunks_mut(n_group) {
                let child = self.build_tree_y(height - 1, chunk)?;
                children.push(child);
            }
            Ok(self.push_node(NodeElem::Node(children)))
        } else {
            self.build_leaf(elems)
        }
    }

    /// Push a node to the node list
    fn push_node(&mut self, ne: NodeElem<F>) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(ne);
        idx
    }

    /// Build a tree (or sub-tree) by partitioning in Y dimension
    fn build_tree_y(
        &mut self,
        height: usize,
        elems: &mut [Elem<F>],
    ) -> Result<usize> {
        if height > 1 {
            elems.sort_unstable_by(Elem::compare_y);
            let mut children = vec![];
            let n_group = Node::<F>::partition_sz(height);
            for chunk in elems.chunks_mut(n_group) {
                let child = self.build_tree_x(height - 1, chunk)?;
                children.push(child);
            }
            Ok(self.push_node(NodeElem::Node(children)))
        } else {
            self.build_leaf(elems)
        }
    }

    /// Build a leaf node
    ///
    /// Returns index in nodes vector
    fn build_leaf(&mut self, elems: &[Elem<F>]) -> Result<usize> {
        let mut leaf = Node::<F>::new();
        for Elem { id, bbox } in elems {
            let geom: G = self.reader.lookup(*id)?;
            let wid = self.writer.push(&geom)?;
            leaf.push(wid, *bbox);
        }
        Ok(self.push_node(NodeElem::Leaf(leaf)))
    }

    /// Write out all nodes
    fn write_nodes(&mut self, n_elems: usize) -> Result<Id> {
        assert!(n_elems > 0);
        let mut elems = vec![];
        let n_nodes = self.nodes.len();
        for ne in &self.nodes[..n_nodes - 1] {
            let node = ne.lookup(&elems);
            let id = self.writer.push(&node)?;
            let bbox = node.bbox();
            elems.push(Elem { id, bbox });
        }
        let ne = &self.nodes[n_nodes - 1];
        let node = ne.lookup(&elems);
        let root = Root::new(node, n_elems);
        let id = self.writer.push(&root)?;
        Ok(id)
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
