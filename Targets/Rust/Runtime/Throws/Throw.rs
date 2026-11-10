fn throw<P, T>(payload: P) -> T {
    std::panic::resume_unwind(Box::new(Box::into_raw(Box::new(payload)) as usize))
}
