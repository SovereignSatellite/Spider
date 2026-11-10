unsafe fn rust_caught<P>(outcome: Result<impl Sized, *mut u8>) -> P {
    *Box::from_raw(outcome.unwrap_err_unchecked() as *mut P)
}
