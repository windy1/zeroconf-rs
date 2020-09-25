macro_rules! assert_not_null {
    ($ptr:expr) => {
        assert!(!$ptr.is_null(), "expected non-null value");
    };
}
