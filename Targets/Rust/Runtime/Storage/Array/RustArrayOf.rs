unsafe fn rust_array_of<T>(bytes: &[u8]) -> *mut [T] {
    let mut elements = Box::<[T]>::new_uninit_slice(bytes.len() / size_of::<T>());
    core::ptr::copy_nonoverlapping(bytes.as_ptr(), elements.as_mut_ptr() as *mut u8, bytes.len());
    Box::into_raw(elements.assume_init())
}
