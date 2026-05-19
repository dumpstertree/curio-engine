#[cfg(test)]
mod tests {
    use curio_core::{Vector2, Vector2Int, Vector3, Vector3Int, Vector4, Vector4Int};

    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::panic;
    use std::str::FromStr;

    const EPSILON: f32 = 0.0001;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < EPSILON
    }

    fn assert_vec2_eq(a: Vector2, b: Vector2) {
        assert!(approx_eq(a.x, b.x), "x mismatch: {} != {}", a.x, b.x);
        assert!(approx_eq(a.y, b.y), "y mismatch: {} != {}", a.y, b.y);
    }

    // =========================================================
    // Constructors
    // =========================================================

    #[test]
    fn new_creates_correct_vector() {
        let v = Vector2::new(1.0, 2.0);

        assert_eq!(v.x, 1.0);
        assert_eq!(v.y, 2.0);
    }

    #[test]
    fn zero_returns_zero_vector() {
        assert_eq!(Vector2::zero(), Vector2::new(0.0, 0.0));
    }

    #[test]
    fn one_returns_one_vector() {
        assert_eq!(Vector2::one(), Vector2::new(1.0, 1.0));
    }

    // =========================================================
    // Parsing
    // =========================================================

    #[test]
    fn from_str_parses_valid_vector() {
        let v = Vector2::from_str("(1.5, -2.5)").unwrap();

        assert_vec2_eq(v, Vector2::new(1.5, -2.5));
    }

    #[test]
    fn from_str_parses_whitespace() {
        let v = Vector2::from_str(" ( 1 , 2 ) ").unwrap();

        assert_vec2_eq(v, Vector2::new(1.0, 2.0));
    }

    #[test]
    fn from_str_rejects_missing_parentheses() {
        assert!(Vector2::from_str("1,2").is_err());
    }

    #[test]
    fn from_str_rejects_extra_values() {
        assert!(Vector2::from_str("(1,2,3)").is_err());
    }

    #[test]
    fn from_str_rejects_invalid_numbers() {
        assert!(Vector2::from_str("(a,b)").is_err());
    }

    // =========================================================
    // Magnitude
    // =========================================================

    #[test]
    fn magnitude_returns_correct_length() {
        let v = Vector2::new(3.0, 4.0);

        assert!(approx_eq(v.magnitude(), 5.0));
    }

    // =========================================================
    // Normalization
    // =========================================================

    #[test]
    fn normalize_modifies_vector() {
        let mut v = Vector2::new(3.0, 4.0);

        v.normalize();

        assert!(approx_eq(v.magnitude(), 1.0));
        assert!(approx_eq(v.x, 0.6));
        assert!(approx_eq(v.y, 0.8));
    }

    #[test]
    fn normalize_and_copy_returns_normalized_vector() {
        let v = Vector2::new(3.0, 4.0);

        let normalized = v.normalize_and_copy();

        assert!(approx_eq(normalized.magnitude(), 1.0));
        assert!(approx_eq(normalized.x, 0.6));
        assert!(approx_eq(normalized.y, 0.8));

        // original unchanged
        assert_vec2_eq(v, Vector2::new(3.0, 4.0));
    }

    #[test]
    fn normalize_zero_vector_panics() {
        let result = panic::catch_unwind(|| {
            let mut v = Vector2::zero();
            v.normalize();
        });

        assert!(result.is_err());
    }

    #[test]
    fn normalize_and_copy_zero_vector_panics() {
        let result = panic::catch_unwind(|| {
            let v = Vector2::zero();
            v.normalize_and_copy();
        });

        assert!(result.is_err());
    }

    // =========================================================
    // Lerp
    // =========================================================

    #[test]
    fn lerp_returns_interpolated_vector() {
        let a = Vector2::zero();
        let b = Vector2::new(10.0, 20.0);

        let result = Vector2::lerp(a, b, 0.5);

        assert_vec2_eq(result, Vector2::new(5.0, 10.0));
    }

    // =========================================================
    // Clamp
    // =========================================================

    #[test]
    fn clamp_x_modifies_x_only() {
        let mut v = Vector2::new(10.0, 2.0);

        v.clamp_x(0.0, 5.0);

        assert_vec2_eq(v, Vector2::new(5.0, 2.0));
    }

    #[test]
    fn clamp_y_modifies_y_only() {
        let mut v = Vector2::new(1.0, -10.0);

        v.clamp_y(0.0, 5.0);

        assert_vec2_eq(v, Vector2::new(1.0, 0.0));
    }

    #[test]
    fn clamp_x_and_copy_returns_new_vector() {
        let v = Vector2::new(10.0, 2.0);

        let result = v.clamp_x_and_copy(0.0, 5.0);

        assert_vec2_eq(result, Vector2::new(5.0, 2.0));

        // original unchanged
        assert_vec2_eq(v, Vector2::new(10.0, 2.0));
    }

    #[test]
    fn clamp_y_and_copy_returns_new_vector() {
        let v = Vector2::new(1.0, -10.0);

        let result = v.clamp_y_and_copy(0.0, 5.0);

        assert_vec2_eq(result, Vector2::new(1.0, 0.0));

        // original unchanged
        assert_vec2_eq(v, Vector2::new(1.0, -10.0));
    }

    #[test]
    fn clamped_modifies_all_components() {
        let mut v = Vector2::new(10.0, -10.0);

        v.clamped(Vector2::new(0.0, 0.0), Vector2::new(5.0, 5.0));

        assert_vec2_eq(v, Vector2::new(5.0, 0.0));
    }

    #[test]
    fn clamp_and_copy_returns_new_clamped_vector() {
        let v = Vector2::new(10.0, -10.0);

        let result = v.clamp_and_copy(Vector2::new(0.0, 0.0), Vector2::new(5.0, 5.0));

        assert_vec2_eq(result, Vector2::new(5.0, 0.0));

        // original unchanged
        assert_vec2_eq(v, Vector2::new(10.0, -10.0));
    }

    // =========================================================
    // Conversions
    // =========================================================

    #[test]
    fn to_vector3_converts_correctly() {
        let v = Vector2::new(1.0, 2.0);

        let result = v.to_vector3(3.0);

        assert_eq!(result, Vector3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn to_vector4_converts_correctly() {
        let v = Vector2::new(1.0, 2.0);

        let result = v.to_vector4(3.0, 4.0);

        assert_eq!(result, Vector4::new(1.0, 2.0, 3.0, 4.0));
    }

    #[test]
    fn to_vector2_int_rounds_correctly() {
        let v = Vector2::new(1.4, 2.6);

        let result = v.to_vector2_int();

        assert_eq!(result, Vector2Int::new(1, 3));
    }

    #[test]
    fn to_vector3_int_converts_correctly() {
        let v = Vector2::new(1.4, 2.6);

        let result = v.to_vector3_int(10);

        assert_eq!(result, Vector3Int::new(1, 3, 10));
    }

    #[test]
    fn to_vector4_int_converts_correctly() {
        let v = Vector2::new(1.4, 2.6);

        let result = v.to_vector4_int(10, 20);

        assert_eq!(result, Vector4Int::new(1, 3, 10, 20));
    }

    // =========================================================
    // Operators
    // =========================================================

    #[test]
    fn scalar_multiplication_works() {
        let v = Vector2::new(1.0, 2.0);

        assert_vec2_eq(v * 2.0, Vector2::new(2.0, 4.0));
    }

    #[test]
    fn scalar_division_works() {
        let v = Vector2::new(2.0, 4.0);

        assert_vec2_eq(v / 2.0, Vector2::new(1.0, 2.0));
    }

    #[test]
    fn vector_addition_works() {
        let a = Vector2::new(1.0, 2.0);
        let b = Vector2::new(3.0, 4.0);

        assert_vec2_eq(a + b, Vector2::new(4.0, 6.0));
    }

    #[test]
    fn vector_subtraction_works() {
        let a = Vector2::new(5.0, 7.0);
        let b = Vector2::new(1.0, 2.0);

        assert_vec2_eq(a - b, Vector2::new(4.0, 5.0));
    }

    // =========================================================
    // Display
    // =========================================================

    #[test]
    fn display_formats_correctly() {
        let v = Vector2::new(1.0, 2.0);

        assert_eq!(format!("{}", v), "Vector2(1, 2)");
    }

    // =========================================================
    // Hash
    // =========================================================

    #[test]
    fn equal_vectors_have_same_hash() {
        let a = Vector2::new(1.0, 2.0);
        let b = Vector2::new(1.0, 2.0);

        let mut hasher_a = DefaultHasher::new();
        let mut hasher_b = DefaultHasher::new();

        a.hash(&mut hasher_a);
        b.hash(&mut hasher_b);

        assert_eq!(hasher_a.finish(), hasher_b.finish());
    }
}
