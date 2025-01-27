pub(crate) fn is_dev() -> bool {
    std::env::var("TESTONLY_ENABLE_DEV").is_ok_and(|v| v == "true")
}
