//! Data type for constructing a service type

use crate::Result;
use std::str::FromStr;

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
            name: Self::check_part(name)?.to_string(),
            protocol: Self::check_part(protocol)?.to_string(),
            sub_types: vec![],
        })
    }

    /// Creates a new `ServiceType` with the specified name (e.g. `http`) and protocol (e.g. `tcp`)
    /// and sub-types.
    pub fn with_sub_types(name: &str, protocol: &str, sub_types: Vec<&str>) -> Result<Self> {
        for sub_type in &sub_types {
            Self::check_part(sub_type)?;
        }

        Ok(Self {
            name: name.to_string(),
            protocol: protocol.to_string(),
            sub_types: sub_types.iter().map(|s| s.to_string()).collect(),
        })
    }

    fn check_part(part: &str) -> Result<&str> {
        if part.contains(".") {
            Err("invalid character: .".into())
        } else if part.contains(",") {
            Err("invalid character: ,".into())
        } else {
            Ok(part)
        }
    }
}

impl ToString for ServiceType {
    fn to_string(&self) -> String {
        format!("_{}._{}{}", self.name, self.protocol, {
            if !self.sub_types.is_empty() {
                format!(",_{}", self.sub_types.join(",_"))
            } else {
                "".to_string()
            }
        })
    }
}

impl FromStr for ServiceType {
    type Err = crate::error::Error;
    fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split(",").collect();
        if parts.is_empty() {
            return Err("could not parse ServiceType from string".into());
        }

        let head: Vec<&str> = parts[0].split(".").collect();
        let mut name = head[0];
        if name.starts_with("_") {
            name = &name[1..];
        }
        let mut protocol = head[1];
        if protocol.starts_with("_") {
            protocol = &protocol[1..];
        }

        let mut sub_types: Vec<&str> = vec![];
        if parts.len() > 1 {
            for i in 1..parts.len() {
                let mut sub_type = parts[i];
                if sub_type.starts_with("_") {
                    sub_type = &sub_type[1..];
                }
                sub_types.push(sub_type);
            }
        }

        Ok(ServiceType::with_sub_types(name, protocol, sub_types)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_invalid() {
        ServiceType::new(".http", "tcp").expect_err("invalid character: .".into());
        ServiceType::new("http", ".tcp").expect_err("invalid character: .".into());
        ServiceType::new(",http", "tcp").expect_err("invalid character: ,".into());
        ServiceType::new("http", ",tcp").expect_err("invalid character: ,".into());
    }

    #[test]
    fn to_string_success() {
        assert_eq!(
            ServiceType::new("http", "tcp").unwrap().to_string(),
            "_http._tcp"
        );
    }

    #[test]
    fn to_string_with_sub_types_success() {
        assert_eq!(
            ServiceType::with_sub_types("http", "tcp", vec!["api-v1", "api-v2"])
                .unwrap()
                .to_string(),
            "_http._tcp,_api-v1,_api-v2"
        );
    }

    #[test]
    fn from_str_success() {
        assert_eq!(
            ServiceType::from_str("_http._tcp").unwrap(),
            ServiceType::new("http", "tcp").unwrap()
        );
    }

    #[test]
    fn from_str_with_sub_types_success() {
        assert_eq!(
            ServiceType::from_str("_http._tcp,api-v1,api-v2").unwrap(),
            ServiceType::with_sub_types("http", "tcp", vec!["api-v1", "api-v2"]).unwrap()
        );
    }
}
