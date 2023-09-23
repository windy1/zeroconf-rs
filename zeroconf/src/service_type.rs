//! Data type for constructing a service type

use std::str::FromStr;

use crate::{error::Error, Result};

/// Data type for constructing a service type to register as an mDNS service.
#[derive(Default, Debug, Getters, Serialize, Deserialize, Clone, PartialEq, Eq)]
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
            return Err("invalid name and protocol".into());
        }

        let name = lstrip_underscore(check_valid_characters(parts[0])?);
        let protocol = lstrip_underscore(check_valid_characters(parts[1])?);

        Self::new(name, protocol)
    }
}

pub fn check_valid_characters(part: &str) -> Result<&str> {
    if part.contains('.') {
        Err("invalid character: .".into())
    } else if part.contains(',') {
        Err("invalid character: ,".into())
    } else if part.is_empty() {
        Err("cannot be empty".into())
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
}
