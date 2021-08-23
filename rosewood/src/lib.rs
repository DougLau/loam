// lib.rs
//
// Copyright (c) 2021  Douglas P Lau
//
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![warn(missing_doc_code_examples)]

mod common;
mod geometry;
mod node;
//mod reader;
mod writer;

pub use common::{Error, Result};
pub use geometry::{
    GeomType, Geometry, LineString, MultiLineString, MultiPoint, MultiPolygon,
    Polygon,
};
pub use writer::BulkWriter;
