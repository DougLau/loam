// error.rs     Errors
//
// Copyright (c) 2021  Douglas P Lau
//

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
