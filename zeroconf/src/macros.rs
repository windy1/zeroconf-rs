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

#[cfg(target_os = "linux")]
macro_rules! avahi {
    ($call:expr, $msg:expr) => {{
        #[allow(unused_unsafe)]
        let err = unsafe { $call };
        if err < 0 {
            crate::Result::Err(
                format!(
                    "{}",
                    format!("{}: `{}`", $msg, crate::linux::avahi_util::get_error(err))
                )
                .into(),
            )
        } else {
            crate::Result::Ok(())
        }
    }};
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
