//! Utilities related to Bonjour

use std::{ffi::CString, str::FromStr};

use super::constants;
use crate::{check_valid_characters, lstrip_underscore, NetworkInterface, Result, ServiceType};
use bonjour_sys::DNSServiceErrorType;

/// Normalizes the specified domain `&str` to conform to a standard enforced by this crate.
///
/// Bonjour suffixes domains with a final `'.'` character in some contexts but is not required by
/// the standard. This function removes the final dot if present.
pub fn normalize_domain(domain: &str) -> String {
    let end = domain
        .chars()
        .nth(domain.len() - 1)
        .expect("could not index domain string");

    if end == '.' {
        String::from(&domain[..domain.len() - 1])
    } else {
        String::from(domain)
    }
}

/// Converts the specified [`NetworkInterface`] to the Bonjour expected value.
///
/// [`NetworkInterface`]: ../../enum.NetworkInterface.html
pub fn interface_index(interface: NetworkInterface) -> u32 {
    match interface {
        NetworkInterface::Unspec => constants::BONJOUR_IF_UNSPEC,
        NetworkInterface::AtIndex(i) => i,
    }
}

/// Converts the specified Bonjour interface index to a [`NetworkInterface`].
pub fn interface_from_index(index: u32) -> NetworkInterface {
    match index {
        constants::BONJOUR_IF_UNSPEC => NetworkInterface::Unspec,
        _ => NetworkInterface::AtIndex(index),
    }
}

/// Executes the specified closure and returns a formatted `Result`
pub fn sys_exec<F: FnOnce() -> DNSServiceErrorType>(func: F, message: &str) -> Result<()> {
    let err = func();

    if err < 0 {
        Err(format!("{} (code: {})", message, err).into())
    } else {
        Ok(())
    }
}

/// Formats the specified `ServiceType` as a `CString` for use with Bonjour
pub fn format_regtype(service_type: &ServiceType) -> CString {
    let mut regtype = vec![format!(
        "_{}._{}",
        service_type.name(),
        service_type.protocol()
    )];

    regtype.extend(
        service_type
            .sub_types()
            .iter()
            .map(|sub_type| format!("_{sub_type}")),
    );

    c_string!(regtype.join(","))
}

/// Parses the specified `&str` into a `ServiceType`
pub fn parse_regtype(regtype: &str) -> Result<ServiceType> {
    let types = regtype.split(',').collect::<Vec<_>>();
    let service_type = ServiceType::from_str(types[0])?;

    let sub_types = types[1..]
        .iter()
        .map(|s| check_valid_characters(lstrip_underscore(s)))
        .collect::<Result<Vec<_>>>()?;

    ServiceType::with_sub_types(service_type.name(), service_type.protocol(), sub_types)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ServiceType;

    #[test]
    fn parse_regtype_success() {
        assert_eq!(
            parse_regtype("_http._tcp,_printer1,_printer2").unwrap(),
            ServiceType::with_sub_types("http", "tcp", vec!["printer1", "printer2"]).unwrap()
        );
    }

    #[test]
    fn parse_regtype_success_no_subtypes() {
        assert_eq!(
            parse_regtype("_http._tcp").unwrap(),
            ServiceType::new("http", "tcp").unwrap()
        );
    }

    #[test]
    fn parse_regtype_failure_invalid_regtype() {
        assert_eq!(
            parse_regtype("foobar"),
            Err("invalid name and protocol".into())
        );
    }

    #[test]
    fn format_regtype_success() {
        assert_eq!(
            format_regtype(
                &ServiceType::with_sub_types("http", "tcp", vec!["printer1", "printer2"]).unwrap()
            ),
            c_string!("_http._tcp,_printer1,_printer2")
        );
    }

    #[test]
    fn format_regtype_success_no_subtypes() {
        assert_eq!(
            format_regtype(&ServiceType::new("http", "tcp").unwrap()),
            c_string!("_http._tcp")
        );
    }

    #[test]
    fn sys_exec_returns_error() {
        assert_eq!(
            sys_exec(|| -42, "uh oh spaghetti-o"),
            Err("uh oh spaghetti-o (code: -42)".into())
        );
    }

    #[test]
    fn sys_exec_returns_ok() {
        assert_eq!(sys_exec(|| 0, "success"), Ok(()));
    }

    #[test]
    fn network_interface_unspec_maps_to_bonjour_if_unspec() {
        assert_eq!(interface_index(NetworkInterface::Unspec), 0);
    }

    #[test]
    fn network_interface_at_index_maps_to_index() {
        assert_eq!(interface_index(NetworkInterface::AtIndex(42)), 42);
    }

    #[test]
    fn normalize_domain_removes_trailing_dot() {
        assert_eq!(
            normalize_domain("foo.bar.baz."),
            String::from("foo.bar.baz")
        );
    }

    #[test]
    fn normalize_domain_does_not_remove_trailing_dot_if_not_present() {
        assert_eq!(normalize_domain("foo.bar.baz"), String::from("foo.bar.baz"));
    }
}
