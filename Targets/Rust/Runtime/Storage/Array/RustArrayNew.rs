fn rust_array_new<T: Copy>(length: u32, value: T) -> *mut [T] {
    let mut elements = Vec::new();

    if elements.try_reserve_exact(length as usize).is_err() {
        return core::ptr::slice_from_raw_parts_mut(core::ptr::null_mut(), 0);
    }

    elements.resize(length as usize, value);
    Box::into_raw(elements.into_boxed_slice())
}
