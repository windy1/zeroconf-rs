use std::fmt;

/// For when something goes wrong when interfacing with mDNS implementations
#[derive(Debug, new)]
pub struct Error {
    description: String,
    #[new(default)]
    source: Option<Box<dyn std::error::Error>>,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description)
    }
}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Error::from(s.to_string())
    }
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::new(s)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
