unsafe fn array_finalize<T>(array: *mut [T]) {
    drop(Box::from_raw(array));
}
