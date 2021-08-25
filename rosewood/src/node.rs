// node.rs
//
// Copyright (c) 2021  Douglas P Lau
//
use crate::common::{Error, Result};
use loam::Id;
use pointy::{BBox, Float};
use serde::{Deserialize, Serialize};

/// Number of elements per node
const ELEM_PER_NODE: usize = 6;

/// Node of RTree
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Node<F>
where
    F: Float,
{
    /// Child Ids, with bounding boxes
    children: [(Id, BBox<F>); ELEM_PER_NODE],
}

/// Root node
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Root<F>
where
    F: Float,
{
    node: Node<F>,
    size: usize,
}

impl<F> Node<F>
where
    F: Float,
{
    /// Get the height of a tree
    pub fn height(n_elem: usize) -> usize {
        // integer logarithm (avoiding domain errors)
        let mut total = ELEM_PER_NODE;
        for height in 1.. {
            match total.checked_mul(height) {
                Some(t) => {
                    total = t;
                    if total >= n_elem {
                        return height;
                    }
                }
                None => break,
            }
        }
        panic!("Incalculable height!")
    }

    /// Create a new node
    pub fn new() -> Self {
        let children = [(Id::new(0), BBox::default()); ELEM_PER_NODE];
        Node { children }
    }

    /// Push a child node
    pub fn push(&mut self, id: Id, bbox: BBox<F>) -> Result<()> {
        for i in 0..ELEM_PER_NODE {
            if !self.children[i].0.is_valid() {
                self.children[i] = (id, bbox);
                return Ok(());
            }
        }
        Err(Error::InvalidTree)
    }
}
