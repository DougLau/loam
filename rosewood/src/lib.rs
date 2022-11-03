// lib.rs
//
// Copyright (c) 2021-2022  Douglas P Lau
//
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![forbid(unsafe_code)]

pub mod gis;
mod node;
mod reader;
mod writer;

pub use reader::RTree;
pub use writer::BulkWriter;
