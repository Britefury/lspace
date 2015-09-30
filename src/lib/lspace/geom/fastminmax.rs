/// Computes the maximum of two numbers
///
/// A faster alternative to f64::max
pub fn fast_max<T: PartialOrd>(a: T, b: T) -> T {
    if a > b {
        return a;
    }
    else {
        return b;
    }
}

/// Computes the maximum of two optional numbers
///
/// If both numbers are present, the maximum is returned.
/// If one number is present, it is returned as is.
/// If both are absent (`None`), then `None` is returned.
pub fn fast_maxopt<T: PartialOrd>(a: Option<T>, b: Option<T>) -> Option<T> {
    match (a, b) {
        (Some(x), Some(y)) => Some(fast_max(x, y)),
        (None, None) => None,
        (x, None) => x,
        (None, y) => y,
    }
}

/// Computes the minimum of two numbers
///
/// A faster alternative to f64::min
pub fn fast_min<T: PartialOrd>(a: T, b: T) -> T {
    if a < b {
        return a;
    }
    else {
        return b;
    }
}

/// Computes the minimum of two optional numbers
///
/// If both numbers are present, the minimum is returned.
/// If one number is present, it is returned as is.
/// If both are absent (`None`), then `None` is returned.
pub fn fast_minopt<T: PartialOrd>(a: Option<T>, b: Option<T>) -> Option<T> {
    match (a, b) {
        (Some(x), Some(y)) => Some(fast_min(x, y)),
        (None, None) => None,
        (x, None) => x,
        (None, y) => y,
    }
}
