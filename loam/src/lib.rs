// lib.rs      loam crate.
//
// Copyright (c) 2021-2022  Douglas P Lau
//
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

mod common;
mod reader;
mod writer;

pub use common::{Error, Id, Result};
pub use reader::Reader;
pub use writer::Writer;
