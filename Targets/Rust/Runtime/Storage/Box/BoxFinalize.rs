unsafe fn box_finalize<T>(box_: *mut T) {
    drop(Box::from_raw(box_));
}
