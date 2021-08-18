// lib.rs      loam crate.
//
// Copyright (c) 2021  Douglas P Lau
//
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![warn(missing_doc_code_examples)]

mod error;
mod reader;
mod writer;

use serde::{Deserialize, Serialize};
use std::convert::TryInto;
use std::num::NonZeroU64;

/// Identifier for data chunks
#[derive(Debug, Deserialize, Serialize)]
pub struct Id(NonZeroU64);

impl Id {
    pub(crate) fn new(id: u64) -> Option<Self> {
        NonZeroU64::new(id).map(Id)
    }
    pub(crate) fn from_le_slice(buf: &[u8]) -> Option<Self> {
        let bytes = buf.try_into().ok()?;
        let id = u64::from_le_bytes(bytes);
        Self::new(id)
    }
    pub(crate) fn to_le_bytes(self) -> [u8; 8] {
        self.0.get().to_le_bytes()
    }
    pub(crate) fn as_usize(self) -> usize {
        self.0.get() as usize
    }
}

pub use error::{Error, Result};
pub use reader::Reader;
pub use writer::Writer;
