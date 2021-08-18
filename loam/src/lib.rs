// lib.rs      loam crate.
//
// Copyright (c) 2021  Douglas P Lau
//
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![warn(missing_doc_code_examples)]

mod common;
mod reader;
mod writer;

pub use common::{Error, Id, Result};
pub use reader::Reader;
pub use writer::Writer;
