pub static IMPORTS: std::sync::Mutex<
    std::collections::BTreeMap<Vec<u8>, std::collections::BTreeMap<Vec<u8>, usize>>,
> = std::sync::Mutex::new(std::collections::BTreeMap::new());
