// common.rs    Common stuff
//
// Copyright (c) 2021  Douglas P Lau
//
use serde::{Deserialize, Serialize};
use std::convert::TryInto;
use std::num::NonZeroU64;

/// Errors for reading or writing loam files
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// I/O error
    #[error("I/O {0}")]
    Io(#[from] std::io::Error),

    /// Bincode error
    #[error("Bincode {0}")]
    Bincode(#[from] Box<bincode::ErrorKind>),

    /// Invalid header
    #[error("Invalid Header")]
    InvalidHeader,

    /// Invalid checkpoint
    #[error("Invalid Checkpoint")]
    InvalidCheckpoint,

    /// Invalid ID
    #[error("Invalid ID")]
    InvalidId,
}

/// Result for reading or writing loam files
pub type Result<T> = std::result::Result<T, Error>;

/// Identifier for data chunks
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
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
    pub(crate) fn to_usize(self) -> usize {
        self.0.get() as usize
    }
}
