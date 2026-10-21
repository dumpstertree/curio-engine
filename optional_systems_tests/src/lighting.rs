#[cfg(test)]
mod tests {
    use curio_core::{Color, FieldState, Vector3};
    use gameplay::traits::field_override::FieldOverride;
    use lighting::{DrawCallLight, Light, LightType, SysRecordLights, SysRecordSun};

    fn state<'a>(fields: &'a [FieldState], name: &str) -> &'a serde_json::Value {
        fields
            .iter()
            .find(|f| f.field_name == name)
            .map(|f| &f.data)
            .unwrap_or_else(|| panic!("field '{}' missing from get_state", name))
    }

    // ---------- builder ----------

    #[test]
    fn builder_defaults() {
        let light = Light::builder().build();
        assert_eq!(light.light_type, LightType::Directional);
        assert_eq!(light.direction, Vector3::down());
        assert_eq!(light.color, Color::white());
        assert_eq!(light.radius, 5.0);
        assert_eq!(light.intensity, 1.0);
    }

    #[test]
    fn builder_sets_every_field() {
        let light = Light::builder()
            .light_type(LightType::Point)
            .direction(Vector3::new(0.0, 1.0, 0.0))
            .radius(12.0)
            .intensity(0.5)
            .build();
        assert_eq!(light.light_type, LightType::Point);
        assert_eq!(light.direction, Vector3::new(0.0, 1.0, 0.0));
        assert_eq!(light.radius, 12.0);
        assert_eq!(light.intensity, 0.5);
    }

    // ---------- apply / get_state ----------

    #[test]
    fn apply_type() {
        let mut light = Light::builder().build();
        light.apply("type", "point");
        assert_eq!(light.light_type, LightType::Point);
        light.apply("type", "DIRECTIONAL");
        assert_eq!(light.light_type, LightType::Directional);
    }

    #[test]
    fn apply_type_bad_keeps_current() {
        let mut light = Light::builder().light_type(LightType::Point).build();
        light.apply("type", "spotlight");
        assert_eq!(light.light_type, LightType::Point);
    }

    #[test]
    fn apply_direction() {
        let mut light = Light::builder().build();
        light.apply("direction", "(0, 1, 0)");
        assert_eq!(light.direction, Vector3::new(0.0, 1.0, 0.0));
    }

    #[test]
    fn apply_direction_bad_keeps_current() {
        let mut light = Light::builder().build();
        let before = light.direction;
        light.apply("direction", "not a vector");
        assert_eq!(light.direction, before);
    }

    #[test]
    fn apply_radius_and_intensity() {
        let mut light = Light::builder().build();
        light.apply("radius", "3.5");
        light.apply("intensity", "0.25");
        assert_eq!(light.radius, 3.5);
        assert_eq!(light.intensity, 0.25);
    }

    #[test]
    fn apply_radius_bad_keeps_current() {
        // a radius of 0 breaks point light falloff
        let mut light = Light::builder().radius(5.0).build();
        light.apply("radius", "zero");
        assert_eq!(light.radius, 5.0);
    }

    #[test]
    fn apply_color_hex() {
        let mut light = Light::builder().build();
        light.apply("color", "#ff0000");
        assert_eq!(light.color, Color::new_hex("ff0000"));
    }

    #[test]
    fn get_state_exposes_all_fields() {
        let light = Light::builder().light_type(LightType::Point).radius(7.0).intensity(2.0).build();
        let fields = light.get_state();
        assert_eq!(state(&fields, "type").as_str(), Some("point"));
        assert_eq!(state(&fields, "radius").as_f64(), Some(7.0));
        assert_eq!(state(&fields, "intensity").as_f64(), Some(2.0));
    }

    #[test]
    fn get_state_apply_round_trip() {
        // what get_state writes must be re-appliable
        let mut light = Light::builder().build();
        for field in light.get_state() {
            light.apply(&field.field_name.clone(), &field.data.to_string());
        }
        assert!(light.radius > 0.0);
    }

    // ---------- sun ----------

    #[test]
    fn sun_draw_call_maps_fields() {
        let mut sun = SysRecordSun::default();
        sun.direction = Vector3::new(-0.25, -1.0, -0.15);
        sun.color = Color::white();
        let draw = sun.get_draw_call();
        assert_eq!(draw.light_type, LightType::Directional);
        assert_eq!(draw.direction, [-0.25, -1.0, -0.15]);
        assert_eq!(draw.position, [0.0, 0.0, 0.0]);
        assert_eq!(draw.intensity, 1.0);
    }

    #[test]
    fn sun_cast_shadows_flag() {
        let mut sun = SysRecordSun::default();
        assert!(!sun.cast_shadows);
        sun.cast_shadows = true;
        assert!(sun.cast_shadows);
    }

    // ---------- lights record ----------

    #[test]
    fn lights_record_starts_empty() {
        let record = SysRecordLights::default();
        assert!(record.all_lights.is_empty());
    }

    #[test]
    fn draw_call_light_defaults() {
        let light = DrawCallLight::default();
        assert_eq!(light.light_type, LightType::Point);
        assert_eq!(light.position, [0.0, 0.0, 0.0]);
    }
}
