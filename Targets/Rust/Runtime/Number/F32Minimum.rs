fn f32_minimum(left: f32, right: f32) -> f32 {
    if left < right {
        left
    } else if right < left {
        right
    } else if left == right {
        f32::from_bits(left.to_bits() | right.to_bits())
    } else {
        left + right
    }
}
