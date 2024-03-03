/// Calculates angle in rads between side `a` and `b`, where `a` straight to the right and `b` goes up.
pub fn sss_triangle(a: u16, b: u16, c: u16) -> f32 {
    let a = if a == 0 { 1_f32 } else { a as f32 };
    let b = if b == 0 { 1_f32 } else { b as f32 };
    let c = if c == 0 { 1_f32 } else { c as f32 };
    ((c * c + a * a - b * b) / (2_f32 * c * a)).acos()
}
