/// Error enum
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

    /// Missing checkpoint
    #[error("Missing Checkpoint")]
    MissingCheckpoint,

    /// Invalid ID
    #[error("Invalid ID")]
    InvalidId,
}

pub type Result<T> = std::result::Result<T, Error>;
