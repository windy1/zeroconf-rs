macro_rules! assert_not_null {
    ($ptr:expr) => {
        assert!(!$ptr.is_null(), "expected non-null value");
    };
}

macro_rules! c_string {
    (alloc($len:expr)) => {
        ::std::ffi::CString::from_vec_unchecked(vec![0; $len])
    };
    ($x:expr) => {
        ::std::ffi::CString::new($x).unwrap()
    };
}

#[cfg(test)]
mod tests {
    use libc::c_char;
    use std::ffi::CString;
    use std::ptr;

    #[test]
    fn assert_not_null_non_null_success() {
        assert_not_null!(c_string!("foo").as_ptr());
    }

    #[test]
    #[should_panic]
    fn assert_not_null_null_panics() {
        assert_not_null!(ptr::null() as *const c_char);
    }

    #[test]
    fn c_string_success() {
        assert_eq!(c_string!("foo"), CString::new("foo").unwrap());
    }
}
