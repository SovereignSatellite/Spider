fn rust_box_new<T>(value: T) -> *mut T {
    Box::into_raw(Box::new(value))
}
