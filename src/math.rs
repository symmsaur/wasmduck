pub fn pow(x: f64, exp: u32) -> f64 {
    let mut result = 1.0;
    for _i in 0..exp {
        result *= x;
    }
    return result;
}

pub fn length(x: f64, y: f64) -> f64 {
    return f64::sqrt(pow(x, 2) + pow(y, 2));
}