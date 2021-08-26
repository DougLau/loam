// node.rs
//
// Copyright (c) 2021  Douglas P Lau
//
use loam::Id;
use pointy::{BBox, Float};
use serde::{Deserialize, Serialize};

/// Number of elements per node
const M_NODE: usize = 6;

/// Node of RTree
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Node<F>
where
    F: Float,
{
    /// Child Ids, with bounding boxes
    children: [(Id, BBox<F>); M_NODE],
}

/// Root node
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Root<F>
where
    F: Float,
{
    node: Node<F>,
    n_elem: usize,
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
        let children = [(Id::new(0), BBox::default()); M_NODE];
        Node { children }
    }

    /// Push a child node
    pub fn push(&mut self, id: Id, bbox: BBox<F>) {
        for i in 0..M_NODE {
            if !self.children[i].0.is_valid() {
                self.children[i] = (id, bbox);
                return;
            }
        }
        panic!("Too many children!");
    }

    /// Get the bounding box
    pub fn bbox(&self) -> BBox<F> {
        let mut bbox = BBox::default();
        for (_, bb) in &self.children {
            bbox.extend(*bb);
        }
        bbox
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
}
