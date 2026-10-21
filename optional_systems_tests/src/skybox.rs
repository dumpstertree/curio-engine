#[cfg(test)]
mod tests {
    use curio_core::{Color, FieldState, RecordOverride};
    use skybox::{SkyboxTypes, SysRecordSkybox};

    fn state<'a>(fields: &'a [FieldState], name: &str) -> &'a serde_json::Value {
        fields
            .iter()
            .find(|f| f.field_name == name)
            .map(|f| &f.data)
            .unwrap_or_else(|| panic!("field '{}' missing from get_state", name))
    }

    #[test]
    fn default_skybox_is_the_default_variant() {
        assert!(matches!(SkyboxTypes::default(), SkyboxTypes::Defualt));
        let record = SysRecordSkybox::default();
        assert!(matches!(record.skybox, SkyboxTypes::Defualt));
    }

    #[test]
    fn default_state_reports_default_type() {
        let record = SysRecordSkybox::default();
        let fields = record.get_state();
        assert_eq!(state(&fields, "type").as_str(), Some("default"));
    }

    #[test]
    fn color_variant_reports_color() {
        let mut record = SysRecordSkybox::default();
        record.skybox = SkyboxTypes::Color(Color::new_hex("102030"));
        let fields = record.get_state();
        assert_eq!(state(&fields, "type").as_str(), Some("default"));
        assert!(fields.iter().any(|f| f.field_name == "color"));
    }

    #[test]
    fn color_variant_equality() {
        let a = SkyboxTypes::Color(Color::new_hex("ffffff"));
        let b = SkyboxTypes::Color(Color::new_hex("ffffff"));
        let c = SkyboxTypes::Color(Color::new_hex("000000"));
        assert!(a == b);
        assert!(a != c);
        assert!(a != SkyboxTypes::Defualt);
        assert!(matches!(SkyboxTypes::default(), SkyboxTypes::Defualt));
    }
}
