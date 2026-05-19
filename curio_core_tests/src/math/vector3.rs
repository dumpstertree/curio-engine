#[cfg(test)]
mod tests {
    use curio_core::Vector3;

    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::panic;
    use std::str::FromStr;

    const EPSILON: f32 = 0.0001;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < EPSILON
    }

    fn assert_vec3_eq(a: Vector3, b: Vector3) {
        assert!(approx_eq(a.x, b.x), "x mismatch: {} != {}", a.x, b.x);
        assert!(approx_eq(a.y, b.y), "y mismatch: {} != {}", a.y, b.y);
        assert!(approx_eq(a.z, b.z), "z mismatch: {} != {}", a.z, b.z);
    }

    // =========================================================
    // Constructors
    // =========================================================

    #[test]
    fn new_creates_correct_vector() {
        let v = Vector3::new(1.0, 2.0, 3.0);

        assert_eq!(v.x, 1.0);
        assert_eq!(v.y, 2.0);
        assert_eq!(v.z, 3.0);
    }

    #[test]
    fn zero_returns_zero_vector() {
        assert_eq!(Vector3::zero(), Vector3::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn one_returns_one_vector() {
        assert_eq!(Vector3::one(), Vector3::new(1.0, 1.0, 1.0));
    }

    #[test]
    fn direction_vectors_are_correct() {
        assert_eq!(Vector3::forward(), Vector3::new(0.0, 0.0, 1.0));
        assert_eq!(Vector3::back(), Vector3::new(0.0, 0.0, -1.0));
        assert_eq!(Vector3::left(), Vector3::new(1.0, 0.0, 0.0));
        assert_eq!(Vector3::right(), Vector3::new(-1.0, 0.0, 0.0));
        assert_eq!(Vector3::up(), Vector3::new(0.0, 1.0, 0.0));
        assert_eq!(Vector3::down(), Vector3::new(0.0, -1.0, 0.0));
    }

    // =========================================================
    // Parsing
    // =========================================================

    #[test]
    fn from_str_parses_valid_vector() {
        let v = Vector3::from_str("(1.0, 2.5, -3.0)").unwrap();

        assert_vec3_eq(v, Vector3::new(1.0, 2.5, -3.0));
    }

    #[test]
    fn from_str_parses_with_whitespace() {
        let v = Vector3::from_str(" ( 1 , 2 , 3 ) ").unwrap();

        assert_vec3_eq(v, Vector3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn from_str_rejects_missing_parentheses() {
        assert!(Vector3::from_str("1,2,3").is_err());
    }

    #[test]
    fn from_str_rejects_extra_values() {
        assert!(Vector3::from_str("(1,2,3,4)").is_err());
    }

    #[test]
    fn from_str_rejects_invalid_numbers() {
        assert!(Vector3::from_str("(a,b,c)").is_err());
    }

    // =========================================================
    // Magnitude / Dot / Cross
    // =========================================================

    #[test]
    fn magnitude_returns_correct_length() {
        let v = Vector3::new(3.0, 4.0, 12.0);

        assert!(approx_eq(v.magnitude(), 13.0));
    }

    #[test]
    fn dot_product_is_correct() {
        let a = Vector3::new(1.0, 2.0, 3.0);
        let b = Vector3::new(4.0, 5.0, 6.0);

        assert!(approx_eq(Vector3::dot(a, b), 32.0));
    }

    #[test]
    fn cross_product_is_correct() {
        let a = Vector3::new(1.0, 0.0, 0.0);
        let b = Vector3::new(0.0, 1.0, 0.0);

        let result = Vector3::cross(a, b);

        assert_vec3_eq(result, Vector3::new(0.0, 0.0, 1.0));
    }

    // =========================================================
    // Reflection
    // =========================================================

    #[test]
    fn reflect_returns_correct_vector() {
        let direction = Vector3::new(1.0, -1.0, 0.0);
        let normal = Vector3::up();

        let reflected = Vector3::reflect(direction, normal);

        assert_vec3_eq(reflected, Vector3::new(1.0, 1.0, 0.0));
    }

    // =========================================================
    // Normalization
    // =========================================================

    #[test]
    fn normalize_modifies_vector() {
        let mut v = Vector3::new(3.0, 0.0, 4.0);

        v.normalize();

        assert!(approx_eq(v.magnitude(), 1.0));
        assert!(approx_eq(v.x, 0.6));
        assert!(approx_eq(v.z, 0.8));
    }

    #[test]
    fn normalize_and_copy_returns_normalized_vector() {
        let v = Vector3::new(0.0, 3.0, 4.0);

        let normalized = v.normalize_and_copy();

        assert!(approx_eq(normalized.magnitude(), 1.0));
        assert!(approx_eq(normalized.y, 0.6));
        assert!(approx_eq(normalized.z, 0.8));

        // original should remain unchanged
        assert_eq!(v, Vector3::new(0.0, 3.0, 4.0));
    }

    #[test]
    fn normalize_zero_vector_panics() {
        let result = panic::catch_unwind(|| {
            let mut v = Vector3::zero();
            v.normalize();
        });

        assert!(result.is_err());
    }

    #[test]
    fn normalize_and_copy_zero_vector_panics() {
        let result = panic::catch_unwind(|| {
            let v = Vector3::zero();
            v.normalize_and_copy();
        });

        assert!(result.is_err());
    }

    // =========================================================
    // Lerp
    // =========================================================

    #[test]
    fn lerp_returns_interpolated_value() {
        let a = Vector3::zero();
        let b = Vector3::new(10.0, 10.0, 10.0);

        let result = Vector3::lerp(a, b, 0.5);

        assert_vec3_eq(result, Vector3::new(5.0, 5.0, 5.0));
    }

    // =========================================================
    // Clamp
    // =========================================================

    #[test]
    fn clamp_x_modifies_x_only() {
        let mut v = Vector3::new(10.0, 2.0, 3.0);

        v.clamp_x(0.0, 5.0);

        assert_vec3_eq(v, Vector3::new(5.0, 2.0, 3.0));
    }

    #[test]
    fn clamp_y_modifies_y_only() {
        let mut v = Vector3::new(1.0, -10.0, 3.0);

        v.clamp_y(0.0, 5.0);

        assert_vec3_eq(v, Vector3::new(1.0, 0.0, 3.0));
    }

    #[test]
    fn clamp_z_modifies_z_only() {
        let mut v = Vector3::new(1.0, 2.0, 10.0);

        v.clamp_z(0.0, 5.0);

        assert_vec3_eq(v, Vector3::new(1.0, 2.0, 5.0));
    }

    #[test]
    fn clamp_and_copy_returns_new_vector() {
        let v = Vector3::new(10.0, -10.0, 3.0);

        let result = v.clamp_and_copy(Vector3::new(0.0, 0.0, 0.0), Vector3::new(5.0, 5.0, 5.0));

        assert_vec3_eq(result, Vector3::new(5.0, 0.0, 3.0));

        // original unchanged
        assert_vec3_eq(v, Vector3::new(10.0, -10.0, 3.0));
    }

    #[test]
    fn clamped_modifies_all_components() {
        let mut v = Vector3::new(10.0, -10.0, 3.0);

        v.clamped(Vector3::new(0.0, 0.0, 0.0), Vector3::new(5.0, 5.0, 5.0));

        assert_vec3_eq(v, Vector3::new(5.0, 0.0, 3.0));
    }

    // =========================================================
    // Operators
    // =========================================================

    #[test]
    fn scalar_multiplication_works() {
        let v = Vector3::new(1.0, 2.0, 3.0);

        assert_vec3_eq(v * 2.0, Vector3::new(2.0, 4.0, 6.0));
    }

    #[test]
    fn scalar_division_works() {
        let v = Vector3::new(2.0, 4.0, 6.0);

        assert_vec3_eq(v / 2.0, Vector3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn vector_addition_works() {
        let a = Vector3::new(1.0, 2.0, 3.0);
        let b = Vector3::new(4.0, 5.0, 6.0);

        assert_vec3_eq(a + b, Vector3::new(5.0, 7.0, 9.0));
    }

    #[test]
    fn vector_subtraction_works() {
        let a = Vector3::new(5.0, 7.0, 9.0);
        let b = Vector3::new(1.0, 2.0, 3.0);

        assert_vec3_eq(a - b, Vector3::new(4.0, 5.0, 6.0));
    }

    // =========================================================
    // Display
    // =========================================================

    #[test]
    fn display_formats_correctly() {
        let v = Vector3::new(1.0, 2.0, 3.0);

        assert_eq!(format!("{}", v), "Vector3(1, 2, 3)");
    }

    // =========================================================
    // Hash / Eq
    // =========================================================

    #[test]
    fn equal_vectors_have_same_hash() {
        let a = Vector3::new(1.0, 2.0, 3.0);
        let b = Vector3::new(1.0, 2.0, 3.0);

        let mut hasher_a = DefaultHasher::new();
        let mut hasher_b = DefaultHasher::new();

        a.hash(&mut hasher_a);
        b.hash(&mut hasher_b);

        assert_eq!(hasher_a.finish(), hasher_b.finish());
    }
}
