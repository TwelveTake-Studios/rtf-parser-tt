#[macro_export]
macro_rules! include_test_file {
    ($filename:expr) => {
        include_str!(concat!("../resources/tests/", $filename))
    };
}
