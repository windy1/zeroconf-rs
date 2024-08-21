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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_service_type_display() {
        let error = Error::InvalidServiceType("invalid name and protocol".into());
        assert_eq!(
            error.to_string(),
            "Invalid ServiceType format: invalid name and protocol"
        );
    }

    #[test]
    fn test_mdns_system_error_display() {
        let error = Error::MdnsSystemError {
            code: -42,
            message: "uh oh spaghetti-o".into(),
        };
        assert_eq!(error.to_string(), "uh oh spaghetti-o (code: -42)");
    }

    #[test]
    fn test_system_error_display() {
        let error = Error::SystemError {
            code: -42,
            message: "uh oh spaghetti-o".into(),
        };
        assert_eq!(error.to_string(), "uh oh spaghetti-o (code: -42)");
    }

    #[test]
    fn test_browser_error_display() {
        let error = Error::BrowserError("uh oh spaghetti-o".into());
        assert_eq!(error.to_string(), "uh oh spaghetti-o");
    }

    #[test]
    fn test_service_error_display() {
        let error = Error::ServiceError("uh oh spaghetti-o".into());
        assert_eq!(error.to_string(), "uh oh spaghetti-o");
    }
}
