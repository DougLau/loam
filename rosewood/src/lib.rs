// lib.rs
//
// Copyright (c) 2021  Douglas P Lau
//
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![warn(missing_doc_code_examples)]

mod geometry;
mod node;
mod reader;
mod writer;

pub use geometry::{GeomType, Geometry, Linestring, Point, Polygon};
pub use reader::RTree;
pub use writer::BulkWriter;
