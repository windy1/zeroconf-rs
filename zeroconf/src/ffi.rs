//! Utilities related to FFI bindings

use libc::c_void;

/// Helper trait to convert a raw `*mut c_void` to it's rust type
pub trait FromRaw<T> {
    /// Converts the specified `*mut c_void` to a `&'a mut T`.
    ///
    /// # Safety
    /// This function is unsafe due to the dereference of the specified raw pointer.
    unsafe fn from_raw<'a>(raw: *mut c_void) -> &'a mut T {
        assert_not_null!(raw);
        &mut *(raw as *mut T)
    }
}

/// Helper trait to convert and clone a raw `*mut c_void` to it's rust type
pub trait CloneRaw<T: FromRaw<T> + Clone> {
    /// Converts and clones the specified `*mut c_void` to a `Box<T>`.
    ///
    /// # Safety
    /// This function is unsafe due to a call to the unsafe function [`FromRaw::from_raw()`].
    ///
    /// [`FromRaw::from_raw()`]: trait.FromRaw.html#method.from_raw
    unsafe fn clone_raw(raw: *mut c_void) -> Box<T> {
        assert_not_null!(raw);
        Box::new(T::from_raw(raw).clone())
    }
}

/// Helper trait to convert self to a raw `*mut c_void`
pub trait AsRaw {
    /// Converts self to a raw `*mut c_void` by cast.
    fn as_raw(&mut self) -> *mut c_void {
        self as *mut _ as *mut c_void
    }
}

pub mod cstr {
    //! FFI utilities related to c-strings

    use libc::c_char;
    use std::ffi::CStr;

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
        use std::ffi::CString;
        use std::ptr;

        #[test]
        fn raw_to_str_success() {
            let c_string = CString::new("foo").unwrap();
            unsafe { assert_eq!(raw_to_str(c_string.as_ptr() as *const c_char), "foo") };
        }

        #[test]
        #[should_panic]
        fn raw_to_str_expects_non_null() {
            unsafe { raw_to_str(ptr::null() as *const c_char) };
        }

        #[test]
        fn copy_raw_success() {
            let c_string = CString::new("foo").unwrap();
            let c_str = c_string.as_ptr() as *const c_char;
            unsafe { assert_eq!(copy_raw(c_str), "foo".to_string()) };
        }

        #[test]
        #[should_panic]
        fn copy_raw_expects_non_null() {
            unsafe { copy_raw(ptr::null() as *const c_char) };
        }
    }
}
