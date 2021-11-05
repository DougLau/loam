// lib.rs
//
// Copyright (c) 2021  Douglas P Lau
//
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![warn(rustdoc::missing_doc_code_examples)]
#![forbid(unsafe_code)]

mod geometry;
mod node;
mod reader;
mod writer;

pub use geometry::{Geom, Geometry, Linestring, Point, Polygon};
pub use reader::RTree;
pub use writer::BulkWriter;
