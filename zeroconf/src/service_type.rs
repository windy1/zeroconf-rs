//! Data type for constructing a service type

use crate::Result;
use std::str::FromStr;

/// Data type for representing a service subtype to register as part of an mDNS service.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct SubType {
    // The string containing the subtype data
    subtype: String,
    // The service type
    kind: String,
}

impl SubType {
    fn new(subtype: &str, kind: &str) -> Result<Self> {
        let subtype = check_valid_characters(subtype)?;
        Ok(Self {
            subtype: subtype.to_owned(),
            kind: kind.to_owned(),
        })
    }
}

impl ToString for SubType {
    fn to_string(&self) -> String {
        format!(
            "{}{}._sub.{}",
            if self.subtype.starts_with('_') {
                ""
            } else {
                "_"
            },
            self.subtype,
            self.kind
        )
    }
}

impl FromStr for SubType {
    type Err = crate::error::Error;

    fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split("._sub.").collect();
        if parts.len() != 2 {
            return Err(format!("Could not parse SubType from {s}").into());
        }

        let subtype_part = parts[0];
        let service_kind_part = parts[1];

        if !ServiceType::from_str(service_kind_part).is_ok() {
            return Err(format!(
                "Could not parse SubType component for service kind from {service_kind_part}"
            )
            .into());
        }

        Ok(Self {
            subtype: subtype_part.to_owned(),
            kind: service_kind_part.to_owned(),
        })
    }
}

/// Data type for constructing a service type to register as an mDNS service.
#[derive(Default, Debug, Getters, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ServiceType {
    name: String,
    protocol: String,
    sub_types: Vec<SubType>,
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
        let mut service_type = Self {
            name: name.to_string(),
            protocol: protocol.to_string(),
            sub_types: vec![],
        };
        service_type.sub_types = sub_types
            .into_iter()
            .map(|subtype| SubType::new(subtype, &service_type.to_string()))
            .collect::<Result<Vec<_>>>()?;
        Ok(service_type)
    }
}

impl ToString for ServiceType {
    fn to_string(&self) -> String {
        format!("_{}._{}", self.name, self.protocol)
    }
}

impl FromStr for ServiceType {
    type Err = crate::error::Error;

    fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 2 {
            return Err("invalid name and protocol".into());
        }

        let name = lstrip_underscore(check_valid_characters(parts[0])?);
        let protocol = lstrip_underscore(check_valid_characters(parts[1])?);

        ServiceType::new(name, protocol)
    }
}

fn lstrip_underscore(s: &str) -> &str {
    if let Some(stripped) = s.strip_prefix('_') {
        stripped
    } else {
        s
    }
}

fn check_valid_characters(part: &str) -> Result<&str> {
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
    fn must_have_name_and_protocol() {
        ServiceType::from_str("_http").expect_err("invalid name and protocol");
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
        let service_type =
            ServiceType::with_sub_types("http", "tcp", vec!["api-v1", "api-v2"]).unwrap();
        assert_eq!(service_type.to_string(), "_http._tcp");

        assert_eq!(service_type.sub_types()[0].kind, "_http._tcp");
        assert_eq!(service_type.sub_types()[0].subtype, "api-v1");
        assert_eq!(
            service_type.sub_types()[0].to_string(),
            "_api-v1._sub._http._tcp"
        );

        assert_eq!(service_type.sub_types()[1].kind, "_http._tcp");
        assert_eq!(service_type.sub_types()[1].subtype, "api-v2");
        assert_eq!(
            service_type.sub_types()[1].to_string(),
            "_api-v2._sub._http._tcp"
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
    fn from_str_with_sub_types_err() {
        ServiceType::from_str("_http._tcp,api-v1,api-v2").expect_err("Subtype format invalid");
    }

    #[test]
    fn from_str_subtype_success() {
        SubType::from_str("_printer._sub._http._tcp").unwrap();
    }
}
