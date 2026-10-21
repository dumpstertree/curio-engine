#[cfg(test)]
mod tests {
    use curio_core::{Color, FieldState};
    use gameplay::traits::field_override::FieldOverride;
    use rendering_image::RendererImage;

    fn state<'a>(fields: &'a [FieldState], name: &str) -> &'a serde_json::Value {
        fields
            .iter()
            .find(|f| f.field_name == name)
            .map(|f| &f.data)
            .unwrap_or_else(|| panic!("field '{}' missing from get_state", name))
    }

    fn has_field<'a>(fields: &'a [FieldState], name: &str) -> bool {
        fields.iter().any(|f| f.field_name == name)
    }

    #[test]
    fn defaults() {
        let image = RendererImage::default();
        assert!(image.asset.is_none());
        assert!(image.enabled);
        assert_eq!(image.tint, Color::white());
    }

    #[test]
    fn builder_without_texture_needs_no_gpu() {
        let image = RendererImage::builder().enabled(false).tint(Color::new_hex("123456")).build();
        assert!(!image.enabled);
        assert_eq!(image.tint, Color::new_hex("123456"));
        assert!(image.asset.is_none());
    }

    #[test]
    fn apply_enabled_and_tint() {
        let mut image = RendererImage::default();
        image.apply("enabled", "false");
        assert!(!image.enabled);
        image.apply("tint", "#ff0000");
        assert_eq!(image.tint, Color::new_hex("ff0000"));
    }

    #[test]
    fn apply_tint_bad_keeps_current() {
        let mut image = RendererImage::builder().tint(Color::new_hex("abcdef")).build();
        image.apply("tint", "garbage");
        assert_eq!(image.tint, Color::new_hex("abcdef"));
    }

    #[test]
    fn get_state_exposes_enabled_and_tint_only() {
        let fields = RendererImage::default().get_state();
        assert!(has_field(&fields, "enabled"));
        assert!(has_field(&fields, "tint"));
        assert!(!has_field(&fields, "asset"));
        assert_eq!(state(&fields, "enabled").as_bool(), Some(true));
    }
}
