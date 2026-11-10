// needs Imports
unsafe fn import<T>(namespace: &[u8], identifier: &[u8]) -> T {
    core::mem::transmute_copy(&IMPORTS.lock().unwrap()[namespace][identifier])
}
