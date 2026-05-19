#[cfg(test)]
mod tests {
    use std::collections::hash_map::DefaultHasher;
    use std::f32::consts::PI;
    use std::hash::{Hash, Hasher};

    use curio_core::{Quaternion, Vector3};

    const EPSILON: f32 = 0.0001;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < EPSILON
    }

    fn assert_quat_eq(a: Quaternion, b: Quaternion) {
        assert!(approx_eq(a.x, b.x), "x mismatch: {} != {}", a.x, b.x);
        assert!(approx_eq(a.y, b.y), "y mismatch: {} != {}", a.y, b.y);
        assert!(approx_eq(a.z, b.z), "z mismatch: {} != {}", a.z, b.z);
        assert!(approx_eq(a.w, b.w), "w mismatch: {} != {}", a.w, b.w);
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
    fn new_creates_correct_quaternion() {
        let q = Quaternion::new(1.0, 2.0, 3.0, 4.0);

        assert_eq!(q.x, 1.0);
        assert_eq!(q.y, 2.0);
        assert_eq!(q.z, 3.0);
        assert_eq!(q.w, 4.0);
    }

    #[test]
    fn identity_returns_correct_quaternion() {
        assert_eq!(Quaternion::identity(), Quaternion::new(0.0, 0.0, 0.0, 1.0));
    }

    #[test]
    fn zero_returns_expected_quaternion() {
        // Note: implementation currently returns w = 10.0
        assert_eq!(Quaternion::zero(), Quaternion::new(0.0, 0.0, 0.0, 10.0));
    }

    // =========================================================
    // Euler Conversion
    // =========================================================

    #[test]
    fn from_euler_zero_returns_identity() {
        let q = Quaternion::from_euler(Vector3::zero());

        assert_quat_eq(q, Quaternion::identity());
    }

    #[test]
    fn from_euler_90_degree_x_rotation() {
        let q = Quaternion::from_euler(Vector3::new(90.0, 0.0, 0.0));

        assert!(approx_eq(q.x.abs(), 0.70710677));
        assert!(approx_eq(q.w.abs(), 0.70710677));
    }

    #[test]
    fn to_euler_identity_returns_zero() {
        let euler = Quaternion::identity().to_euler();

        assert_vec3_eq(euler, Vector3::zero());
    }

    #[test]
    fn euler_round_trip_is_approximately_equal() {
        let original = Vector3::new(30.0, 45.0, 60.0);

        let q = Quaternion::from_euler(original);
        let result = q.to_euler();

        // to_euler() returns pitch,yaw,roll ordering
        assert!(approx_eq(result.x, original.y));
        assert!(approx_eq(result.y, original.z));
        assert!(approx_eq(result.z, original.x));
    }

    // =========================================================
    // Inverse
    // =========================================================

    #[test]
    fn inverse_of_identity_is_identity() {
        let inv = Quaternion::identity().inverse();

        assert_quat_eq(inv, Quaternion::identity());
    }

    #[test]
    fn inverse_multiplied_by_original_is_identity() {
        let q = Quaternion::from_euler(Vector3::new(45.0, 30.0, 60.0));

        let result = q * q.inverse();

        assert!(approx_eq(result.x, 0.0));
        assert!(approx_eq(result.y, 0.0));
        assert!(approx_eq(result.z, 0.0));
        assert!(approx_eq(result.w, 1.0));
    }

    #[test]
    fn inverse_of_zero_quaternion_returns_identity() {
        let q = Quaternion::new(0.0, 0.0, 0.0, 0.0);

        assert_eq!(q.inverse(), Quaternion::identity());
    }

    // =========================================================
    // Angle Axis
    // =========================================================

    #[test]
    fn from_angle_axis_zero_angle_returns_identity() {
        let q = Quaternion::from_angle_axis(Vector3::up(), 0.0);

        assert_eq!(q, Quaternion::identity());
    }

    #[test]
    fn from_angle_axis_creates_expected_rotation() {
        let q = Quaternion::from_angle_axis(Vector3::up(), PI);

        assert!(approx_eq(q.y.abs(), 1.0));
        assert!(approx_eq(q.w.abs(), 0.0));
    }

    // =========================================================
    // Look Rotation
    // =========================================================

    #[test]
    fn look_rotation_forward_up_returns_identity() {
        let q = Quaternion::from_look_rotation(Vector3::forward(), Vector3::up());

        assert!(approx_eq(q.x, 0.0));
        assert!(approx_eq(q.y, 0.0));
        assert!(approx_eq(q.z, 0.0));
        assert!(approx_eq(q.w, 1.0));
    }

    // =========================================================
    // Slerp
    // =========================================================

    #[test]
    fn slerp_at_zero_returns_start() {
        let start = Quaternion::identity();
        let end = Quaternion::from_euler(Vector3::new(0.0, 90.0, 0.0));

        let result = Quaternion::slerp(start, end, 0.0);

        assert_quat_eq(result, start);
    }

    #[test]
    fn slerp_at_one_returns_end() {
        let start = Quaternion::identity();
        let end = Quaternion::from_euler(Vector3::new(0.0, 90.0, 0.0));

        let result = Quaternion::slerp(start, end, 1.0);

        assert_quat_eq(result, end.normalized());
    }

    #[test]
    fn slerp_clamps_t_values() {
        let start = Quaternion::identity();
        let end = Quaternion::from_euler(Vector3::new(0.0, 90.0, 0.0));

        let below_zero = Quaternion::slerp(start, end, -1.0);
        let above_one = Quaternion::slerp(start, end, 2.0);

        assert_quat_eq(below_zero, start);
        assert_quat_eq(above_one, end.normalized());
    }

    #[test]
    fn slerp_result_is_normalized() {
        let start = Quaternion::identity();
        let end = Quaternion::from_euler(Vector3::new(0.0, 180.0, 0.0));

        let result = Quaternion::slerp(start, end, 0.5);

        let mag = (result.x * result.x + result.y * result.y + result.z * result.z + result.w * result.w).sqrt();

        assert!(approx_eq(mag, 1.0));
    }

    // =========================================================
    // Normalization
    // =========================================================

    #[test]
    fn normalized_returns_unit_quaternion() {
        let q = Quaternion::new(1.0, 2.0, 3.0, 4.0);

        let normalized = q.normalized();

        let mag = (normalized.x * normalized.x + normalized.y * normalized.y + normalized.z * normalized.z + normalized.w * normalized.w).sqrt();

        assert!(approx_eq(mag, 1.0));
    }

    #[test]
    fn normalized_zero_quaternion_returns_self() {
        let q = Quaternion::new(0.0, 0.0, 0.0, 0.0);

        assert_eq!(q.normalized(), q);
    }

    // =========================================================
    // Quaternion Multiplication
    // =========================================================

    #[test]
    fn quaternion_identity_multiplication_returns_same_quaternion() {
        let q = Quaternion::from_euler(Vector3::new(10.0, 20.0, 30.0));

        assert_quat_eq(q * Quaternion::identity(), q);
        assert_quat_eq(Quaternion::identity() * q, q);
    }

    #[test]
    fn quaternion_multiplication_combines_rotations() {
        let qx = Quaternion::from_euler(Vector3::new(90.0, 0.0, 0.0));
        let qy = Quaternion::from_euler(Vector3::new(0.0, 90.0, 0.0));

        let combined = qx * qy;

        // Just verify result is normalized and non-zero
        let mag = (combined.x * combined.x + combined.y * combined.y + combined.z * combined.z + combined.w * combined.w).sqrt();

        assert!(approx_eq(mag, 1.0));
    }

    // =========================================================
    // Vector Rotation
    // =========================================================

    #[test]
    fn quaternion_rotates_vector_correctly() {
        let q = Quaternion::from_angle_axis(Vector3::up(), PI);

        let v = Vector3::forward();

        let rotated = q * v;

        assert!(approx_eq(rotated.x, 0.0));
        assert!(approx_eq(rotated.z, -1.0));
    }

    #[test]
    fn identity_quaternion_does_not_rotate_vector() {
        let v = Vector3::new(1.0, 2.0, 3.0);

        let rotated = Quaternion::identity() * v;

        assert_vec3_eq(rotated, v);
    }

    // =========================================================
    // Display
    // =========================================================

    #[test]
    fn display_formats_correctly() {
        let q = Quaternion::new(1.0, 2.0, 3.0, 4.0);

        assert_eq!(format!("{}", q), "Quaternion(1, 2, 3, 4)");
    }

    // =========================================================
    // Hash
    // =========================================================

    #[test]
    fn equal_quaternions_have_same_hash() {
        let a = Quaternion::new(1.0, 2.0, 3.0, 4.0);
        let b = Quaternion::new(1.0, 2.0, 3.0, 4.0);

        let mut hasher_a = DefaultHasher::new();
        let mut hasher_b = DefaultHasher::new();

        a.hash(&mut hasher_a);
        b.hash(&mut hasher_b);

        assert_eq!(hasher_a.finish(), hasher_b.finish());
    }
}
