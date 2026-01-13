//! Error types for Synarion Engine.

use thiserror::Error;

/// The main error type for Synarion Engine.
#[derive(Error, Debug)]
pub enum Error {
    /// An I/O error occurred.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// A resource was not found.
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// An invalid handle was used.
    #[error("Invalid handle")]
    InvalidHandle,

    /// An invalid state was encountered.
    #[error("Invalid state: {0}")]
    InvalidState(String),

    /// A custom error message.
    #[error("{0}")]
    Custom(String),
}

/// A Result type alias using the engine's Error type.
pub type Result<T> = std::result::Result<T, Error>;
