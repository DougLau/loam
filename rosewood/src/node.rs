// node.rs
//
// Copyright (c) 2021  Douglas P Lau
//
use loam::Id;
use pointy::BBox;
use serde::{Deserialize, Serialize};

/// Branch node
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Node {
    bbox: BBox<f32>,
    children: [Id; 6],
}

/// Root node
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RTree {
    root: Node,
    size: usize,
}
