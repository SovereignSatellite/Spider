// needs Exports
fn export<T>(identifier: &[u8], box_: *mut T) {
    EXPORTS.lock().unwrap().insert(identifier.to_vec(), box_ as usize);
}
