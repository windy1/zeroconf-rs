//! Data type for constructing a service type

use std::str::FromStr;

use crate::{error::Error, Result};

/// Data type for constructing a service type to register as an mDNS service.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Default, Debug, Getters, Clone, PartialEq, Eq)]
pub struct ServiceType {
    name: String,
    protocol: String,
    sub_types: Vec<String>,
}

impl ServiceType {
    /// Creates a new `ServiceType` with the specified name (e.g. `http`) and protocol (e.g. `tcp`)
    pub fn new(name: &str, protocol: &str) -> Result<Self> {
        Ok(Self {
            name: check_valid_characters(name)?.to_string(),
            protocol: check_valid_characters(protocol)?.to_string(),
            sub_types: vec![],
        })
    }

    /// Creates a new `ServiceType` with the specified name (e.g. `http`) and protocol (e.g. `tcp`)
    /// and sub-types.
    pub fn with_sub_types(name: &str, protocol: &str, sub_types: Vec<&str>) -> Result<Self> {
        Ok(Self {
            name: check_valid_characters(name)?.to_string(),
            protocol: check_valid_characters(protocol)?.to_string(),
            sub_types: sub_types
                .into_iter()
                .map(|s| check_valid_characters(s).map(|valid| valid.to_string()))
                .collect::<Result<Vec<_>>>()?,
        })
    }
}

impl FromStr for ServiceType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let parts = s.split('.').collect::<Vec<_>>();

        if parts.len() != 2 {
            let msg = "invalid name and protocol";
            return Err(Error::InvalidServiceType(msg.into()));
        }

        let name = lstrip_underscore(check_valid_characters(parts[0])?);
        let protocol = lstrip_underscore(check_valid_characters(parts[1])?);

        Self::new(name, protocol)
    }
}

pub fn check_valid_characters(part: &str) -> Result<&str> {
    if part.contains('.') {
        let msg = "invalid character: .";
        Err(Error::InvalidServiceType(msg.into()))
    } else if part.contains(',') {
        Err(Error::InvalidServiceType("invalid character: ,".into()))
    } else if part.is_empty() {
        Err(Error::InvalidServiceType("cannot be empty".into()))
    } else {
        Ok(part)
    }
}

pub fn lstrip_underscore(s: &str) -> &str {
    if let Some(stripped) = s.strip_prefix('_') {
        stripped
    } else {
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_invalid() {
        ServiceType::new(".http", "tcp").expect_err("invalid character: .");
        ServiceType::new("http", ".tcp").expect_err("invalid character: .");
        ServiceType::new(",http", "tcp").expect_err("invalid character: ,");
        ServiceType::new("http", ",tcp").expect_err("invalid character: ,");
        ServiceType::new("", "tcp").expect_err("cannot be empty");
        ServiceType::new("http", "").expect_err("cannot be empty");
    }

    #[test]
    fn from_str_requires_two_parts() {
        ServiceType::from_str("_http").expect_err("invalid name and protocol");
        ServiceType::from_str("_http._tcp._foo").expect_err("invalid name and protocol");
    }

    #[test]
    fn from_str_success() {
        assert_eq!(
            ServiceType::from_str("_http._tcp").unwrap(),
            ServiceType::new("http", "tcp").unwrap()
        );
    }

    #[test]
    fn check_valid_characters_returns_error_if_dot() {
        check_valid_characters("foo.bar").expect_err("invalid character: .");
    }

    #[test]
    fn check_valid_characters_returns_error_if_comma() {
        check_valid_characters("foo,bar").expect_err("invalid character: ,");
    }

    #[test]
    fn check_valid_characters_returns_error_if_empty() {
        check_valid_characters("").expect_err("cannot be empty");
    }

    #[test]
    fn check_valid_characters_success() {
        assert_eq!(check_valid_characters("foo").unwrap(), "foo");
    }

    #[test]
    fn lstrip_underscore_returns_stripped() {
        assert_eq!(lstrip_underscore("_foo"), "foo");
    }

    #[test]
    fn lstrip_underscore_returns_original() {
        assert_eq!(lstrip_underscore("foo"), "foo");
    }
}
