//! Utilities related to c-string handling

use libc::c_char;
use std::ffi::{CStr, CString};

/// Helper trait to map to `Option<*const c_char>`.
pub trait AsCChars {
    /// Maps the type to a `Option<*const c_char>`.
    fn as_c_chars(&self) -> Option<*const c_char>;
}

impl AsCChars for Option<&CString> {
    fn as_c_chars(&self) -> Option<*const c_char> {
        self.map(|s| s.as_ptr() as *const c_char)
    }
}

/// Returns the specified `*const c_char` as a `&'a str`. Ownership is not taken.
///
/// # Safety
/// This function is unsafe due to a call to the unsafe function [`CStr::from_ptr()`].
///
/// [`CStr::from_ptr()`]: https://doc.rust-lang.org/std/ffi/struct.CStr.html#method.from_ptr
pub unsafe fn raw_to_str<'a>(s: *const c_char) -> &'a str {
    assert_not_null!(s);
    CStr::from_ptr(s).to_str().unwrap()
}

/// Copies the specified `*const c_char` into a `String`.
///
/// # Safety
/// This function is unsafe due to a call to the unsafe function [`raw_to_str()`].
///
/// [`raw_to_str()`]: fn.raw_to_str.html
pub unsafe fn copy_raw(s: *const c_char) -> String {
    assert_not_null!(s);
    String::from(raw_to_str(s))
}

#[cfg(test)]
mod tests {
    use super::*;
    use libc::c_char;
    use std::ptr;

    #[test]
    fn raw_to_str_success() {
        let c_string = c_string!("foo");
        unsafe { assert_eq!(raw_to_str(c_string.as_ptr() as *const c_char), "foo") };
    }

    #[test]
    #[should_panic]
    fn raw_to_str_expects_non_null() {
        unsafe { raw_to_str(ptr::null() as *const c_char) };
    }

    #[test]
    fn copy_raw_success() {
        let c_string = c_string!("foo");
        let c_str = c_string.as_ptr() as *const c_char;
        unsafe { assert_eq!(copy_raw(c_str), "foo".to_string()) };
    }

    #[test]
    #[should_panic]
    fn copy_raw_expects_non_null() {
        unsafe { copy_raw(ptr::null() as *const c_char) };
    }
}
