// common.rs
//
// Copyright (c) 2021  Douglas P Lau
//

/// Errors for reading or writing rosewood files
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// I/O error
    #[error("I/O {0}")]
    Io(#[from] std::io::Error),

    /// Loam error
    #[error("Loam {0}")]
    Loam(#[from] loam::Error),
}

/// Result for reading or writing rosewood files
pub type Result<T> = std::result::Result<T, Error>;
