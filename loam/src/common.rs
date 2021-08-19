// common.rs    Common stuff
//
// Copyright (c) 2021  Douglas P Lau
//
use serde::{Deserialize, Serialize};
use std::fmt;

/// Errors for reading or writing loam files
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// I/O error
    #[error("I/O {0}")]
    Io(#[from] std::io::Error),

    /// Bincode error
    #[error("Bincode {0}")]
    Bincode(#[from] Box<bincode::ErrorKind>),

    /// Invalid Header
    #[error("Invalid Header")]
    InvalidHeader,

    /// Invalid CRC
    #[error("Invalid CRC")]
    InvalidCrc(Id),

    /// Invalid Checkpoint
    #[error("Invalid Checkpoint")]
    InvalidCheckpoint,

    /// Invalid ID
    #[error("Invalid ID")]
    InvalidId(Id),
}

/// Result for reading or writing loam files
pub type Result<T> = std::result::Result<T, Error>;

/// File header
pub const HEADER: &[u8; 8] = b"loam0000";

/// Chunk Identifier
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Id(u64);

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Id: {:?}", self.0)
    }
}

impl Id {
    /// Create a new Id
    pub fn new(id: u64) -> Self {
        Id(id)
    }

    /// Check if Id is valid
    pub fn is_valid(self) -> bool {
        self.0 > 0
    }

    pub(crate) fn from_le_bytes(bytes: [u8; 8]) -> Self {
        Id(u64::from_le_bytes(bytes))
    }

    pub(crate) fn to_le_bytes(self) -> [u8; 8] {
        self.0.to_le_bytes()
    }

    pub(crate) fn from_usize(id: usize) -> Self {
        Id(id as u64)
    }

    pub(crate) fn to_usize(self) -> usize {
        self.0 as usize
    }
}

#[cfg(feature = "crc")]
pub const CRC_SZ: usize = 4;

#[cfg(feature = "crc")]
pub fn checksum(buf: &[u8]) -> Option<u32> {
    let mut hasher = crc32fast::Hasher::new();
    hasher.update(&buf);
    Some(hasher.finalize())
}

#[cfg(not(feature = "crc"))]
pub const CRC_SZ: usize = 0;

#[cfg(not(feature = "crc"))]
pub fn checksum(_buf: &[u8]) -> Option<u32> {
    None
}
