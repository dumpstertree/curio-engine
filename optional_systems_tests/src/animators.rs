#[cfg(test)]
mod tests {
    use animators::{AnimatorPositionSin, AnimatorRotationSin, AnimatorScaleSin};
    use curio_core::{FieldState, Vector3};
    use gameplay::traits::field_override::FieldOverride;

    fn state<'a>(fields: &'a [FieldState], name: &str) -> &'a serde_json::Value {
        fields
            .iter()
            .find(|f| f.field_name == name)
            .map(|f| &f.data)
            .unwrap_or_else(|| panic!("field '{}' missing from get_state", name))
    }

    // every sin animator shares the same field set, so each check runs over all three

    #[test]
    fn defaults_match_across_animators() {
        let defaults = [
            AnimatorPositionSin::default().get_state(),
            AnimatorRotationSin::default().get_state(),
            AnimatorScaleSin::default().get_state(),
        ];
        for fields in defaults {
            assert_eq!(state(&fields, "enabled").as_bool(), Some(false));
            assert_eq!(state(&fields, "speed").as_f64(), Some(1.0));
            assert_eq!(state(&fields, "min")["x"].as_f64(), Some(0.5));
            assert_eq!(state(&fields, "max")["x"].as_f64(), Some(1.5));
        }
    }

    #[test]
    fn builders_set_every_field() {
        let position = AnimatorPositionSin::builder()
            .enabled(true)
            .min(Vector3::new(0.0, 1.0, 2.0))
            .max(Vector3::new(3.0, 4.0, 5.0))
            .speed(2.5)
            .build();
        assert!(position.enabled);
        assert_eq!(position.min, Vector3::new(0.0, 1.0, 2.0));
        assert_eq!(position.max, Vector3::new(3.0, 4.0, 5.0));
        assert_eq!(position.speed, 2.5);

        let rotation = AnimatorRotationSin::builder().enabled(true).speed(0.25).build();
        assert!(rotation.enabled);
        assert_eq!(rotation.speed, 0.25);

        let scale = AnimatorScaleSin::builder().min(Vector3::new(0.1, 0.1, 0.1)).build();
        assert_eq!(scale.min, Vector3::new(0.1, 0.1, 0.1));
    }

    #[test]
    fn builder_defaults_match_default() {
        assert_eq!(AnimatorPositionSin::builder().build().speed, AnimatorPositionSin::default().speed);
        assert_eq!(AnimatorRotationSin::builder().build().min, AnimatorRotationSin::default().min);
        assert_eq!(AnimatorScaleSin::builder().build().max, AnimatorScaleSin::default().max);
    }

    #[test]
    fn apply_valid_values() {
        let mut animator = AnimatorPositionSin::default();
        animator.apply("min", "(0, 0, 0)");
        animator.apply("max", "(10, 0, 0)");
        animator.apply("speed", "3.5");
        animator.apply("enabled", "true");
        assert_eq!(animator.min, Vector3::zero());
        assert_eq!(animator.max, Vector3::new(10.0, 0.0, 0.0));
        assert_eq!(animator.speed, 3.5);
        assert!(animator.enabled);
    }

    #[test]
    fn apply_bad_values_keep_current() {
        let mut animator = AnimatorScaleSin::builder()
            .min(Vector3::new(1.0, 1.0, 1.0))
            .max(Vector3::new(2.0, 2.0, 2.0))
            .speed(4.0)
            .build();
        animator.apply("min", "not a vector");
        animator.apply("max", "(1,2)");
        animator.apply("speed", "fast");
        assert_eq!(animator.min, Vector3::new(1.0, 1.0, 1.0));
        assert_eq!(animator.max, Vector3::new(2.0, 2.0, 2.0));
        assert_eq!(animator.speed, 4.0);
    }

    #[test]
    fn apply_unknown_field_ignored() {
        let mut animator = AnimatorRotationSin::default();
        animator.apply("bogus", "x");
        assert!(!animator.enabled);
    }

    #[test]
    fn get_state_exposes_all_fields() {
        let animator = AnimatorPositionSin::builder().enabled(true).speed(9.0).build();
        let fields = animator.get_state();
        assert_eq!(fields.len(), 4);
        assert_eq!(state(&fields, "enabled").as_bool(), Some(true));
        assert_eq!(state(&fields, "speed").as_f64(), Some(9.0));
    }
}
