fn f64_maximum(left: f64, right: f64) -> f64 {
    if left > right {
        left
    } else if right > left {
        right
    } else if left == right {
        f64::from_bits(left.to_bits() & right.to_bits())
    } else {
        left + right
    }
}
