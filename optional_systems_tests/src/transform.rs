#[cfg(test)]
mod tests {
    use curio_core::{Matrix4x4, Vector3};
    use gameplay::{
        built_in::facet::transform::{transform2d::Transform2D, transform3d::Transform3D},
        traits::transform_common::TransformCommon,
    };

    const EPSILON: f32 = 0.0001;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < EPSILON
    }

    fn assert_vec3_eq(a: Vector3, b: Vector3) {
        assert!(approx_eq(a.x, b.x) && approx_eq(a.y, b.y) && approx_eq(a.z, b.z), "{:?} != {:?}", a, b);
    }

    #[test]
    fn position_round_trips_through_the_trait_on_3d() {
        let mut transform = Transform3D::default();
        transform.set_position(Vector3::new(1.0, 2.0, 3.0));
        assert_vec3_eq(transform.position(), Vector3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn position_promotes_and_drops_z_on_2d() {
        let mut transform = Transform2D::default();
        // reading a 2D position promotes to Vector3 with z = 0
        assert_vec3_eq(transform.position(), Vector3::zero());
        // writing a Vector3 drops z (the plane is z = 0)
        transform.set_position(Vector3::new(4.0, 5.0, 99.0));
        assert_vec3_eq(transform.position(), Vector3::new(4.0, 5.0, 0.0));
        assert_vec3_eq(Vector3::new(transform.position.x, transform.position.y, 0.0), Vector3::new(4.0, 5.0, 0.0));
    }

    #[test]
    fn scale_round_trips_through_the_trait() {
        let mut transform_3d = Transform3D::default();
        transform_3d.set_scale(Vector3::new(2.0, 0.5, 1.0));
        assert_vec3_eq(transform_3d.scale(), Vector3::new(2.0, 0.5, 1.0));

        let mut transform_2d = Transform2D::default();
        transform_2d.set_scale(Vector3::new(2.0, 0.5, 1.0));
        assert_vec3_eq(transform_2d.scale(), Vector3::new(2.0, 0.5, 1.0));
    }

    #[test]
    fn euler_helper_matches_explicit_quaternion() {
        use curio_core::Quaternion;

        let mut transform = Transform3D::default();
        transform.set_rotation_euler(Vector3::new(0.0, 90.0, 0.0));
        let expected = Quaternion::from_euler(Vector3::new(0.0, 90.0, 0.0));
        let actual = transform.rotation();
        assert!(approx_eq(actual.x, expected.x) && approx_eq(actual.y, expected.y) && approx_eq(actual.z, expected.z) && approx_eq(actual.w, expected.w));
    }

    #[test]
    fn local_matrix_translation_follows_position() {
        let mut transform = Transform3D::default();
        transform.set_position(Vector3::new(7.0, -2.0, 5.0));
        let matrix = transform.local_matrix();
        // Matrix4x4 stores translation in the fourth row
        for i in 0..3 {
            assert!(approx_eq(matrix.model[3][i], [7.0, -2.0, 5.0][i]), "row: {:?}", matrix.model[3]);
        }
    }

    #[test]
    fn trait_objects_share_one_interface() {
        // the whole point of the abstraction: one closure body over both transforms
        fn nudge(transform: &mut dyn TransformCommon, dz: f32) {
            let mut position = transform.position();
            position.z += dz;
            transform.set_position(position);
        }

        let mut transform_3d = Transform3D::default();
        nudge(&mut transform_3d, 2.0);
        assert_vec3_eq(transform_3d.position(), Vector3::new(0.0, 0.0, 2.0));

        let mut transform_2d = Transform2D::default();
        nudge(&mut transform_2d, 2.0);
        // z is dropped on the way back into the 2D transform
        assert_vec3_eq(transform_2d.position(), Vector3::zero());
    }
}
