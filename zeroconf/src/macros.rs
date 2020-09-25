macro_rules! assert_not_null {
    ($ptr:expr) => {
        assert!(!$ptr.is_null(), "expected non-null value");
    };
}

macro_rules! c_string {
    ($x:expr) => {
        ::std::ffi::CString::new($x).unwrap()
    };
}
