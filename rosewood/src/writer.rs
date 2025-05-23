// writer.rs
//
// Copyright (c) 2021-2025  Douglas P Lau
//
use crate::gis::Gis;
use crate::node::{Entry, M_NODE, Node, Root};
use loam::{Id, Reader, Result, Writer};
use pointy::Float;
use serde::{Serialize, de::DeserializeOwned};
use std::io::ErrorKind;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

/// Axis for sorting
#[derive(Copy, Clone, Debug, PartialEq)]
enum Axis {
    X,
    Y,
}

impl Axis {
    fn with_height(self, height: usize) -> Self {
        if height % 2 != 0 {
            self
        } else {
            match self {
                Axis::X => Axis::Y,
                Axis::Y => Axis::X,
            }
        }
    }
}

/// Node file element
enum NodeElem<F>
where
    F: Float,
{
    /// Leaf node
    Leaf(Node<F>),

    /// Non-leaf node with back-ref indices into nodes vector
    Node(Vec<usize>),
}

impl<F> NodeElem<F>
where
    F: Float,
{
    fn lookup(&self, node_entries: &[Entry<F>]) -> Node<F> {
        match self {
            NodeElem::Leaf(leaf) => leaf.clone(),
            NodeElem::Node(children) => {
                let mut n = Node::new();
                for child in children {
                    let entry = &node_entries[*child];
                    n.push(entry.id(), entry.bbox());
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
/// 1. All `Gis` values, grouped by leaf node in order to reduce page faults
///    when reading.
/// 2. All `Node` values, in depth-first order, with the root appearing last.
///
/// [OMT]: http://ceur-ws.org/Vol-74/files/FORUM_18.pdf
pub struct BulkWriter<D, F, G>
where
    F: Float + Serialize + DeserializeOwned,
    G: Gis<F, Data = D> + Serialize + DeserializeOwned,
{
    /// Path to file
    path: PathBuf,

    /// Writer to temporary file
    writer: Writer,

    /// Reader for temporary file
    reader: Reader,

    /// Gis entries
    elems: Vec<Entry<F>>,

    /// Node entries
    ///
    /// This is built during the first step (while writing `Gis` entries), and
    /// used during the second step to write out `Node` data
    nodes: Vec<NodeElem<F>>,

    /// Axis for odd height values
    odd_axis: Axis,

    _data: PhantomData<D>,
    _float: PhantomData<F>,
    _geom: PhantomData<G>,
}

/// Make a loam writer, overwriting file if it exists
fn make_writer(path: &Path) -> Result<Writer> {
    match Writer::new(path) {
        Err(loam::Error::Io(e)) if e.kind() == ErrorKind::AlreadyExists => {
            std::fs::remove_file(path)?;
            Writer::new(path)
        }
        w => w,
    }
}

impl<D, F, G> BulkWriter<D, F, G>
where
    F: Float + Serialize + DeserializeOwned,
    G: Gis<F, Data = D> + Serialize + DeserializeOwned,
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
        let writer = make_writer(&tmp)?;
        let reader = Reader::new_empty()?;
        Ok(Self {
            path,
            writer,
            reader,
            elems: Vec::new(),
            nodes: Vec::new(),
            odd_axis: Axis::X,
            _data: PhantomData,
            _float: PhantomData,
            _geom: PhantomData,
        })
    }

    /// Push geometry
    pub fn push(&mut self, geom: &G) -> Result<()> {
        let id = self.writer.push(geom)?;
        let bbox = geom.bbox();
        self.elems.push(Entry::new(id, bbox));
        Ok(())
    }

    /// Build RTree from pushed items
    pub fn finish(mut self) -> Result<()> {
        let mut elems = std::mem::take(&mut self.elems);
        if elems.is_empty() {
            // no items were pushed
            self.cancel()?;
            return Err(loam::Error::InvalidCheckpoint);
        }
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
        self.build_tree(&mut elems)?;
        let id = self.write_nodes(elems.len())?;
        self.writer.checkpoint(id)?;
        let path = self.path;
        drop(self.writer);
        remove_tmp_file(&path)?;
        rename_tree(&path)?;
        Ok(())
    }

    /// Cancel building RTree
    pub fn cancel(self) -> Result<()> {
        let path = self.path;
        drop(self.writer);
        remove_tmp_file(&path)
    }

    /// Build the tree recursively
    fn build_tree(&mut self, elems: &mut [Entry<F>]) -> Result<usize> {
        let n_elems = elems.len();
        log::debug!("n_elems: {}", n_elems);
        let height = Node::<F>::height(n_elems);
        log::debug!("height: {}", height);
        self.odd_axis = Axis::Y.with_height(height);
        if height > 1 {
            elems.sort_unstable_by(Entry::compare_x);
            let groups = Node::<F>::root_groups(n_elems);
            assert!(groups > 0);
            let n_group = (n_elems as f32 / groups as f32).ceil() as usize;
            let v_group = M_NODE / groups;
            log::debug!(
                "groups: {}, n_group: {}, v_group: {}",
                groups,
                n_group,
                v_group
            );
            let mut children = Vec::with_capacity(M_NODE);
            for v_chunk in elems.chunks_mut(n_group) {
                v_chunk.sort_unstable_by(Entry::compare_y);
                let n_chunk =
                    (v_chunk.len() as f32 / v_group as f32).ceil() as usize;
                log::debug!("n_chunk: {}", n_chunk);
                for h_chunk in v_chunk.chunks_mut(n_chunk) {
                    let child = self.build_subtree(height - 1, h_chunk)?;
                    children.push(child);
                }
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

    /// Build a sub-tree recursively
    fn build_subtree(
        &mut self,
        height: usize,
        elems: &mut [Entry<F>],
    ) -> Result<usize> {
        if height > 1 {
            match self.odd_axis.with_height(height) {
                Axis::X => elems.sort_unstable_by(Entry::compare_x),
                Axis::Y => elems.sort_unstable_by(Entry::compare_y),
            }
            let mut children = Vec::with_capacity(M_NODE);
            let n_group = Node::<F>::partition_sz(height);
            for chunk in elems.chunks_mut(n_group) {
                let child = self.build_subtree(height - 1, chunk)?;
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
    fn build_leaf(&mut self, elems: &[Entry<F>]) -> Result<usize> {
        let mut leaf = Node::<F>::new();
        for entry in elems {
            let geom: G = self.reader.lookup(entry.id())?;
            let wid = self.writer.push(&geom)?;
            leaf.push(wid, entry.bbox());
        }
        Ok(self.push_node(NodeElem::Leaf(leaf)))
    }

    /// Write out all nodes
    fn write_nodes(&mut self, n_elems: usize) -> Result<Id> {
        assert!(n_elems > 0);
        let n_nodes = self.nodes.len();
        let mut node_entries = Vec::with_capacity(n_nodes);
        for ne in &self.nodes[..n_nodes - 1] {
            let node = ne.lookup(&node_entries);
            let id = self.writer.push(&node)?;
            let bbox = node.bbox();
            node_entries.push(Entry::new(id, bbox));
        }
        let ne = &self.nodes[n_nodes - 1];
        let node = ne.lookup(&node_entries);
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn axis() {
        let axis = Axis::Y.with_height(4);
        assert_eq!(Axis::X, axis);
        assert_eq!(Axis::X, axis.with_height(3));
        assert_eq!(Axis::Y, axis.with_height(2));
        assert_eq!(Axis::X, axis.with_height(1));
        let axis = Axis::Y.with_height(3);
        assert_eq!(Axis::Y, axis);
        assert_eq!(Axis::X, axis.with_height(2));
        assert_eq!(Axis::Y, axis.with_height(1));
    }
}
