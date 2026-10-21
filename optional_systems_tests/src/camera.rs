#[cfg(test)]
mod tests {
    use camera::{Camera, SysRecordCamera};
    use curio_core::{FieldState, RecordCommon, Vector3};
    use gameplay::traits::field_override::FieldOverride;

    const EPSILON: f32 = 0.0001;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < EPSILON
    }

    fn state<'a>(fields: &'a [FieldState], name: &str) -> &'a serde_json::Value {
        fields
            .iter()
            .find(|f| f.field_name == name)
            .map(|f| &f.data)
            .unwrap_or_else(|| panic!("field '{}' missing from get_state", name))
    }

    // ---------- facet ----------

    #[test]
    fn default_has_reasonable_fov() {
        assert_eq!(Camera::default().fov, 60.0);
    }

    #[test]
    fn builder_sets_fov() {
        let camera = Camera::builder().fov(90.0).build();
        assert_eq!(camera.fov, 90.0);
    }

    #[test]
    fn builder_defaults_match_default() {
        assert_eq!(Camera::builder().build().fov, Camera::default().fov);
    }

    #[test]
    fn apply_fov_valid() {
        let mut camera = Camera::default();
        camera.apply("fov", "45.5");
        assert!(approx_eq(camera.fov, 45.5));
    }

    #[test]
    fn apply_fov_bad_parse_keeps_current() {
        let mut camera = Camera::builder().fov(70.0).build();
        camera.apply("fov", "not_a_number");
        assert_eq!(camera.fov, 70.0);
    }

    #[test]
    fn apply_unknown_field_ignored() {
        let mut camera = Camera::default();
        camera.apply("bogus", "1");
        assert_eq!(camera.fov, 60.0);
    }

    #[test]
    fn get_state_exposes_fov() {
        let camera = Camera::builder().fov(42.0).build();
        let fields = camera.get_state();
        assert_eq!(fields.len(), 1);
        assert_eq!(state(&fields, "fov").as_f64(), Some(42.0));
    }

    // ---------- snapshot ----------

    #[test]
    fn snapshot_defaults() {
        let snapshot = camera::camera_snapshot::CameraSnapshot::default();
        assert!(approx_eq(snapshot.fovy, 60.0));
        assert!(approx_eq(snapshot.znear, 0.1));
        assert!(approx_eq(snapshot.zfar, 100000.0));
    }

    #[test]
    fn snapshot_new_uses_short_zfar() {
        let snapshot = camera::camera_snapshot::CameraSnapshot::new(Vector3::new(1.0, 2.0, 3.0));
        assert_eq!(snapshot.position, Vector3::new(1.0, 2.0, 3.0));
        assert!(approx_eq(snapshot.zfar, 512.0));
    }

    #[test]
    fn calc_matrix_at_origin_has_no_translation() {
        let snapshot = camera::camera_snapshot::CameraSnapshot::new(Vector3::zero());
        let matrix = snapshot.calc_matrix();
        // view matrix translation column
        for i in 0..3 {
            assert!(approx_eq(matrix[3][i], 0.0), "expected no translation, got {:?}", matrix[3]);
        }
    }

    #[test]
    fn calc_matrix_translation_follows_eye() {
        let snapshot = camera::camera_snapshot::CameraSnapshot::new(Vector3::new(1.0, 2.0, 3.0));
        let matrix = snapshot.calc_matrix();
        // identity rotation with forward = -z: view translation = -[s.e, u.e, f.e] for the eye
        assert!(approx_eq(matrix[3][0], 1.0), "x: {}", matrix[3][0]);
        assert!(approx_eq(matrix[3][1], -2.0), "y: {}", matrix[3][1]);
        assert!(approx_eq(matrix[3][2], 3.0), "z: {}", matrix[3][2]);
    }

    #[test]
    fn projection_y_scale_matches_fovy() {
        let snapshot = camera::camera_snapshot::CameraSnapshot::new(Vector3::zero());
        let projection = snapshot.get_projection(800, 600);
        let matrix = projection.calc_matrix();
        // m.y.y = 1 / tan(fovy/2) = 1 / tan(30deg)
        assert!((matrix[1][1] - 1.7320508).abs() < 0.001, "y scale {}", matrix[1][1]);
    }

    // ---------- record ----------

    #[test]
    fn record_new_starts_empty() {
        let record = SysRecordCamera::new();
        assert_eq!(record.resolution_width, 0);
        assert_eq!(record.resolution_height, 0);
    }

    #[test]
    fn record_has_unique_id() {
        assert_ne!(SysRecordCamera::id(), collision::SysRecordCollider::id());
    }
}
