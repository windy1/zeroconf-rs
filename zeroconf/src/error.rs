//! Utilities regarding error handling

use thiserror::Error;

/// Error type for the zeroconf crate
#[derive(Error, Debug, PartialEq)]
pub enum Error {
    /// An instance of `crate::ServiceType` could not be created due to an invalid format
    #[error("Invalid ServiceType format: {0}")]
    InvalidServiceType(String),
    /// An error occurred in the underlying mDNS system (Avahi/Bonjour)
    #[error("{message} (code: {code})")]
    MdnsSystemError { code: i32, message: String },
    /// An error occurred in the underlying system (ABI)
    #[error("{message} (code: {code})")]
    SystemError { code: i32, message: String },
    /// An error occurred in an instance of an `crate::MdnsBrowser`
    #[error("{0}")]
    BrowserError(String),
    /// An error occurred in an instance of an `crate::MdnsService`
    #[error("{0}")]
    ServiceError(String),
}
