//! Utilities related to Avahi

use super::constants;
use crate::NetworkInterface;
use avahi_sys::{avahi_address_snprint, avahi_strerror, AvahiAddress};
use libc::c_char;
use std::ffi::{CStr, CString};
use std::mem;

/// Converts the specified `*const AvahiAddress` to a `String`.
///
/// The new `String` is constructed through allocating a new `CString`, passing it to
/// `avahi_address_snprint` and then converting it to a Rust-type `String`.
///
/// # Safety
/// This function is unsafe because of internal Avahi calls and raw pointer dereference.
pub unsafe fn avahi_address_to_string(addr: *const AvahiAddress) -> String {
    assert_not_null!(addr);

    let addr_str = CString::from_vec_unchecked(vec![0; constants::AVAHI_ADDRESS_STR_MAX]);

    avahi_address_snprint(
        addr_str.as_ptr() as *mut c_char,
        mem::size_of_val(&addr_str),
        addr,
    );

    String::from(addr_str.to_str().unwrap())
        .trim_matches(char::from(0))
        .to_string()
}

/// Returns the `&str` message associated with the specified error code.
pub fn get_error<'a>(code: i32) -> &'a str {
    unsafe {
        CStr::from_ptr(avahi_strerror(code))
            .to_str()
            .expect("could not fetch Avahi error string")
    }
}

/// Converts the specified [`NetworkInterface`] to the Avahi expected value.
///
/// [`NetworkInterface`]: ../../enum.NetworkInterface.html
pub fn interface_index(interface: NetworkInterface) -> i32 {
    match interface {
        NetworkInterface::Unspec => constants::AVAHI_IF_UNSPEC,
        NetworkInterface::AtIndex(i) => i as i32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_error_returns_valid_error_string() {
        assert_eq!(get_error(avahi_sys::AVAHI_ERR_FAILURE), "Operation failed");
    }
}
