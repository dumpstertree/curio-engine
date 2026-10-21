#[cfg(test)]
mod tests {
    use curio_core::{Color, FieldState};
    use gameplay::traits::field_override::FieldOverride;
    use rendering_dynamic::facet::renderer_dynamic::RendererDynamic;

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

    // ---------- defaults ----------

    #[test]
    fn defaults() {
        let renderer = RendererDynamic::default();
        assert!(renderer.asset.is_none());
        assert!(renderer.mesh.is_empty());
        assert!(renderer.enabled);
        assert_eq!(renderer.tint, Color::white());
    }

    // ---------- builder ----------

    #[test]
    fn builder_sets_every_field() {
        let renderer = RendererDynamic::builder()
            .animation("walk", true)
            .enabled(false)
            .tint(Color::new_hex("ff0000"))
            .build();
        assert!(!renderer.enabled);
        assert_eq!(renderer.tint, Color::new_hex("ff0000"));
        // animation + looping are private, verify through get_state
        let fields = renderer.get_state();
        assert_eq!(state(&fields, "animation").as_str(), Some("walk"));
        assert_eq!(state(&fields, "looping").as_bool(), Some(true));
    }

    #[test]
    fn builder_defaults_match_default() {
        let built = RendererDynamic::builder().build();
        let default = RendererDynamic::default();
        assert_eq!(built.enabled, default.enabled);
        assert_eq!(built.tint, default.tint);
        assert_eq!(state(&built.get_state(), "animation").as_str(), Some(""));
    }

    // ---------- apply ----------

    #[test]
    fn apply_enabled_and_tint() {
        let mut renderer = RendererDynamic::default();
        renderer.apply("enabled", "false");
        assert!(!renderer.enabled);
        renderer.apply("tint", "#00ff00");
        assert_eq!(renderer.tint, Color::new_hex("00ff00"));
    }

    #[test]
    fn apply_tint_bad_keeps_current() {
        let mut renderer = RendererDynamic::default();
        renderer.apply("tint", "wat");
        assert_eq!(renderer.tint, Color::white());
    }

    #[test]
    fn apply_animation_and_looping() {
        let mut renderer = RendererDynamic::default();
        renderer.apply("animation", "run");
        renderer.apply("looping", "true");
        let fields = renderer.get_state();
        assert_eq!(state(&fields, "animation").as_str(), Some("run"));
        assert_eq!(state(&fields, "looping").as_bool(), Some(true));
    }

    #[test]
    fn apply_unknown_field_ignored() {
        let mut renderer = RendererDynamic::default();
        renderer.apply("bogus", "x");
        assert!(renderer.enabled);
    }

    // ---------- get_state ----------

    #[test]
    fn get_state_exposes_all_fields_but_asset() {
        let fields = RendererDynamic::default().get_state();
        assert!(has_field(&fields, "enabled"));
        assert!(has_field(&fields, "tint"));
        assert!(has_field(&fields, "animation"));
        assert!(has_field(&fields, "looping"));
        assert!(!has_field(&fields, "asset"));
    }

    // ---------- update_mesh ----------

    #[test]
    fn update_mesh_without_asset_is_a_noop() {
        let mut renderer = RendererDynamic::default();
        renderer.update_mesh(0.5);
        assert!(renderer.mesh.is_empty());
    }
}
