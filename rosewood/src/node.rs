// node.rs
//
// Copyright (c) 2021  Douglas P Lau
//
use loam::Id;
use pointy::{BBox, Float};
use serde::{Deserialize, Serialize};

/// Number of elements per node
const ELEM_PER_NODE: usize = 6;

/// Branch node
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Node<F>
where
    F: Float,
{
    bbox: BBox<F>,
    children: [Id; ELEM_PER_NODE],
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
    /// Get the depth of a tree
    pub fn depth(n_elem: usize) -> usize {
        // integer logarithm (avoiding domain errors)
        let mut total = ELEM_PER_NODE;
        for depth in 1.. {
            match total.checked_mul(depth) {
                Some(t) => {
                    total = t;
                    if total >= n_elem {
                        return depth;
                    }
                }
                None => break,
            }
        }
        panic!("Incalculable depth!")
    }

    /// Create a new leaf node
    pub fn new_leaf(bbox: BBox<F>, ids: &[Id]) -> Self {
        assert!(ids.len() <= ELEM_PER_NODE);
        let mut children = [Id::new(0); ELEM_PER_NODE];
        for i in 0..ids.len() {
            children[i] = ids[i];
        }
        Node { bbox, children }
    }
}
