// node.rs
//
// Copyright (c) 2021  Douglas P Lau
//
use loam::Id;
use pointy::BBox;
use serde::{Deserialize, Serialize};

/// Number of elements per node
const ELEM_PER_NODE: usize = 6;

/// Branch node
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Node {
    bbox: BBox<f32>,
    children: [Id; ELEM_PER_NODE],
    size: usize,
}

impl Node {
    pub fn depth(n_elem: usize) -> usize {
        (n_elem as f32).log(ELEM_PER_NODE as f32).ceil() as usize
    }
}
