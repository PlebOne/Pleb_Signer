//! Error types for Pleb Signer

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SignerError {
    #[error("Key not found: {0}")]
    KeyNotFound(String),

    #[error("Invalid key format: {0}")]
    InvalidKeyFormat(String),

    #[error("Encryption error: {0}")]
    EncryptionError(String),

    #[error("Decryption error: {0}")]
    DecryptionError(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Invalid password")]
    InvalidPassword,

    #[error("No keys configured")]
    NoKeysConfigured,

    #[error("Key already exists: {0}")]
    KeyAlreadyExists(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Nostr error: {0}")]
    NostrError(String),

    #[error("D-Bus error: {0}")]
    DbusError(String),

    #[error("User rejected the request")]
    UserRejected,

    #[error("Request timeout")]
    Timeout,

    #[error("Application not authorized: {0}")]
    NotAuthorized(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),
}

impl From<nostr::key::Error> for SignerError {
    fn from(e: nostr::key::Error) -> Self {
        SignerError::NostrError(e.to_string())
    }
}

impl From<nostr::event::Error> for SignerError {
    fn from(e: nostr::event::Error) -> Self {
        SignerError::NostrError(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, SignerError>;
