// node.rs
//
// Copyright (c) 2021-2023  Douglas P Lau
//
use loam::Id;
use pointy::{BBox, Bounded, Float, Pt};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// Number of elements per node
pub const M_NODE: usize = 6;

/// Entry in a file (geometry or node)
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Entry<F>
where
    F: Float,
{
    /// Id (file offset)
    id: Id,

    /// Bounding box
    bbox: BBox<F>,
}

/// Node of RTree
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Node<F>
where
    F: Float,
{
    /// Child entries
    children: [Entry<F>; M_NODE],
}

/// Root node
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Root<F>
where
    F: Float,
{
    /// Node containing children
    node: Node<F>,

    /// Number of elements in tree
    n_elem: usize,
}

impl<F> Default for Entry<F>
where
    F: Float,
{
    fn default() -> Self {
        let id = Id::new(0);
        let pt = Pt::new(F::zero(), F::zero());
        let bbox = BBox::new([pt, pt]);
        Self { id, bbox }
    }
}

impl<F> Entry<F>
where
    F: Float,
{
    /// Create a new entry
    pub fn new(id: Id, bbox: BBox<F>) -> Self {
        Self { id, bbox }
    }

    /// Get the entry Id
    pub fn id(&self) -> Id {
        self.id
    }

    /// Get the entry bounding box
    pub fn bbox(&self) -> BBox<F> {
        self.bbox
    }

    /// Compare entries by X coordinate
    pub fn compare_x(&self, rhs: &Self) -> Ordering {
        self.bbox
            .x_mid()
            .partial_cmp(&rhs.bbox.x_mid())
            .unwrap_or(Ordering::Equal)
    }

    /// Compare entries by Y coordinate
    pub fn compare_y(&self, rhs: &Self) -> Ordering {
        self.bbox
            .y_mid()
            .partial_cmp(&rhs.bbox.y_mid())
            .unwrap_or(Ordering::Equal)
    }
}

impl<F> Bounded<F> for &Entry<F>
where
    F: Float,
{
    fn bounded_by(self, bbox: BBox<F>) -> bool {
        self.id.is_valid() && self.bbox.bounded_by(bbox)
    }
}

impl<F> Node<F>
where
    F: Float,
{
    /// Get the height of a tree
    pub fn height(n_elem: usize) -> usize {
        // integer logarithm (avoiding domain errors)
        let mut capacity = M_NODE;
        for height in 1.. {
            if capacity >= n_elem {
                return height;
            }
            match capacity.checked_mul(M_NODE) {
                Some(c) => capacity = c,
                None => break,
            }
        }
        panic!("Incalculable height!")
    }

    /// Calculate the number of groups to partition on each axis
    pub fn root_groups(n_elem: usize) -> usize {
        let height = Node::<F>::height(n_elem);
        let n_subtree = M_NODE.pow(height as u32 - 1);
        let n_groups = (n_elem as f32 / n_subtree as f32).ceil();
        n_groups.sqrt().ceil() as usize
    }

    /// Get the partition size for a subtree
    pub fn partition_sz(height: usize) -> usize {
        M_NODE.pow(height as u32 - 1)
    }

    /// Create a new node
    pub fn new() -> Self {
        let children = [Entry::default(); M_NODE];
        Node { children }
    }

    /// Push a child node
    pub fn push(&mut self, id: Id, bbox: BBox<F>) {
        for i in 0..M_NODE {
            if !self.children[i].id.is_valid() {
                self.children[i] = Entry::new(id, bbox);
                return;
            }
        }
        panic!("Too many children: {id:?}");
    }

    /// Get the bounding box
    pub fn bbox(&self) -> BBox<F> {
        let mut bbox = BBox::default();
        for child in &self.children {
            if child.id.is_valid() {
                bbox.extend(child.bbox);
            }
        }
        bbox
    }

    pub fn into_entries(self) -> [Entry<F>; M_NODE] {
        self.children
    }
}

impl<F> Root<F>
where
    F: Float,
{
    /// Create a new root node
    pub fn new(node: Node<F>, n_elem: usize) -> Self {
        Self { node, n_elem }
    }

    /// Get the number of elements
    pub fn n_elem(&self) -> usize {
        self.n_elem
    }

    /// Get inner node
    pub fn into_node(self) -> Node<F> {
        self.node
    }
}
