/// Calculates angle in rads between side `a` and `b`, where `a` straight to the right and `b` goes up.
pub fn sss_triangle(a: u16, b: u16, c: u16) -> f32 {
    ((c * c + a * a - b * b) as f32 / (2 * c * a) as f32).acos()
}
