#[cfg(test)]
mod tests {
    use curio_core::{Color, FieldState};
    use gameplay::traits::field_override::FieldOverride;
    use rendering_static::RendererStatic;

    fn state<'a>(fields: &'a [FieldState], name: &str) -> &'a serde_json::Value {
        fields
            .iter()
            .find(|f| f.field_name == name)
            .map(|f| &f.data)
            .unwrap_or_else(|| panic!("field '{}' missing from get_state", name))
    }

    #[test]
    fn defaults() {
        let renderer = RendererStatic::default();
        assert!(renderer.asset.is_none());
        assert_eq!(renderer.tint, Color::white());
    }

    #[test]
    fn builder_sets_tint() {
        let renderer = RendererStatic::builder().tint(Color::new_hex("4488ff")).build();
        assert_eq!(renderer.tint, Color::new_hex("4488ff"));
        assert!(renderer.asset.is_none());
    }

    #[test]
    fn builder_defaults_match_default() {
        let built = RendererStatic::builder().build();
        assert_eq!(built.tint, RendererStatic::default().tint);
    }

    #[test]
    fn apply_tint_valid() {
        let mut renderer = RendererStatic::default();
        renderer.apply("tint", "#010203");
        assert_eq!(renderer.tint, Color::new_hex("010203"));
    }

    #[test]
    fn apply_tint_bad_keeps_current() {
        let mut renderer = RendererStatic::builder().tint(Color::new_hex("111111")).build();
        renderer.apply("tint", "oops");
        assert_eq!(renderer.tint, Color::new_hex("111111"));
    }

    #[test]
    fn apply_unknown_field_ignored() {
        let mut renderer = RendererStatic::default();
        renderer.apply("bogus", "x");
        assert_eq!(renderer.tint, Color::white());
    }

    #[test]
    fn get_state_exposes_tint_only() {
        let fields = RendererStatic::default().get_state();
        assert_eq!(fields.len(), 1);
        assert!(state(&fields, "tint").is_object() || state(&fields, "tint").is_string());
    }
}
