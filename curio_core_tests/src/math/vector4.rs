#[cfg(test)]
mod tests {
    use std::panic;

    use curio_core::{Vector2, Vector3, Vector4, Vector4Int};

    const EPSILON: f32 = 0.0001;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < EPSILON
    }

    fn assert_vec4_eq(a: Vector4, b: Vector4) {
        assert!(approx_eq(a.x, b.x), "x mismatch {} != {}", a.x, b.x);
        assert!(approx_eq(a.y, b.y), "y mismatch {} != {}", a.y, b.y);
        assert!(approx_eq(a.z, b.z), "z mismatch {} != {}", a.z, b.z);
        assert!(approx_eq(a.w, b.w), "w mismatch {} != {}", a.w, b.w);
    }

    // =========================================================
    // Constructors
    // =========================================================

    #[test]
    fn new_creates_vector4() {
        let v = Vector4::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(v.x, 1.0);
        assert_eq!(v.y, 2.0);
        assert_eq!(v.z, 3.0);
        assert_eq!(v.w, 4.0);
    }

    #[test]
    fn zero_returns_zero_vector() {
        assert_eq!(Vector4::zero(), Vector4::new(0.0, 0.0, 0.0, 0.0));
    }

    #[test]
    fn one_returns_expected_vector() {
        assert_eq!(Vector4::one(), Vector4::new(1.0, 1.0, 1.0, 0.0));
    }

    // =========================================================
    // Magnitude
    // =========================================================

    #[test]
    fn magnitude_is_correct() {
        let v = Vector4::new(1.0, 2.0, 2.0, 1.0);
        assert!(approx_eq(v.magnitude(), 3.0));
    }

    // =========================================================
    // Normalize
    // =========================================================

    #[test]
    fn normalize_modifies_vector() {
        let mut v = Vector4::new(1.0, 0.0, 0.0, 0.0);
        v.normalize();

        assert!(approx_eq(v.magnitude(), 1.0));
        assert!(approx_eq(v.x, 1.0));
    }

    #[test]
    fn normalize_and_copy_returns_new_vector() {
        let v = Vector4::new(0.0, 3.0, 4.0, 0.0);
        let n = v.normalize_and_copy();

        assert!(approx_eq(n.magnitude(), 1.0));
        assert_eq!(v, Vector4::new(0.0, 3.0, 4.0, 0.0)); // unchanged
    }

    #[test]
    fn normalize_zero_panics() {
        let result = panic::catch_unwind(|| {
            let mut v = Vector4::zero();
            v.normalize();
        });
        assert!(result.is_err());
    }

    // =========================================================
    // Clamp
    // =========================================================

    #[test]
    fn clamp_x_works() {
        let mut v = Vector4::new(10.0, 2.0, 3.0, 4.0);
        v.clamp_x(0.0, 5.0);
        assert_vec4_eq(v, Vector4::new(5.0, 2.0, 3.0, 4.0));
    }

    #[test]
    fn clamp_w_works() {
        let mut v = Vector4::new(1.0, 2.0, 3.0, 10.0);
        v.clamp_w(0.0, 5.0);
        assert_vec4_eq(v, Vector4::new(1.0, 2.0, 3.0, 5.0));
    }

    #[test]
    fn clamp_and_copy_works() {
        let v = Vector4::new(10.0, -10.0, 3.0, 7.0);

        let result = v.clamp_and_copy(Vector4::new(0.0, 0.0, 0.0, 0.0), Vector4::new(5.0, 5.0, 5.0, 5.0));

        assert_vec4_eq(result, Vector4::new(5.0, 0.0, 3.0, 5.0));
    }

    // =========================================================
    // Math operations
    // =========================================================

    #[test]
    fn scalar_mul_works() {
        let v = Vector4::new(1.0, 2.0, 3.0, 4.0);
        assert_vec4_eq(v * 2.0, Vector4::new(2.0, 4.0, 6.0, 8.0));
    }

    #[test]
    fn scalar_div_works() {
        let v = Vector4::new(2.0, 4.0, 6.0, 8.0);
        assert_vec4_eq(v / 2.0, Vector4::new(1.0, 2.0, 3.0, 4.0));
    }

    #[test]
    fn add_works() {
        let a = Vector4::new(1.0, 2.0, 3.0, 4.0);
        let b = Vector4::new(1.0, 1.0, 1.0, 1.0);
        assert_vec4_eq(a + b, Vector4::new(2.0, 3.0, 4.0, 5.0));
    }

    #[test]
    fn sub_works() {
        let a = Vector4::new(5.0, 6.0, 7.0, 8.0);
        let b = Vector4::new(1.0, 2.0, 3.0, 4.0);

        assert_vec4_eq(a - b, Vector4::new(4.0, 4.0, 4.0, 4.0));
    }

    // =========================================================
    // Conversions
    // =========================================================

    #[test]
    fn to_vector2_discards_correct_components() {
        let v = Vector4::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(v.to_vector2(), Vector2::new(1.0, 2.0));
    }

    #[test]
    fn to_vector3_discards_w() {
        let v = Vector4::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(v.to_vector3(), Vector3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn to_vector4_int_converts() {
        let v = Vector4::new(1.7, 2.2, 3.9, 4.1);
        assert_eq!(v.to_vector4_int(), Vector4Int::new(1, 2, 3, 4));
    }

    // =========================================================
    // Display
    // =========================================================

    #[test]
    fn display_formats_correctly() {
        let v = Vector4::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(format!("{}", v), "Vector4(1, 2, 3, 4)");
    }
}
