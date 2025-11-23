#![allow(useless_ptr_null_checks)]

macro_rules! assert_not_null {
    ($ptr:expr) => {
        assert!(!$ptr.is_null(), "expected non-null value");
    };
}

macro_rules! c_string {
    (alloc($len:expr)) => {
        unsafe { ::std::ffi::CString::from_vec_unchecked(vec![0; $len]) }
    };
    ($x:expr) => {
        ::std::ffi::CString::new($x).expect("could not create new CString")
    };
}

#[cfg(test)]
mod tests {
    use libc::c_char;
    use std::ffi::CString;
    use std::ptr;

    #[test]
    fn assert_not_null_non_null_success() {
        let c_str = c_string!("foo");
        assert_not_null!(c_str.as_ptr());
    }

    #[test]
    #[should_panic]
    fn assert_not_null_null_panics() {
        assert_not_null!(ptr::null() as *const c_char);
    }

    #[test]
    fn c_string_success() {
        assert_eq!(
            c_string!("foo"),
            CString::new("foo").expect("could not create new CString")
        );
    }
}
