fn rust_try<R>(callee: impl FnOnce() -> R) -> Result<R, *mut u8> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(callee)).map_err(|thrown| match thrown.downcast::<usize>() {
        Ok(payload) => *payload as *mut u8,
        Err(foreign) => std::panic::resume_unwind(foreign),
    })
}
