#[cfg(test)]
mod tests {
    use curio_core::{AssetCommon, FieldState, RecordOverride};
    use ext_rendering::data::material::ShaderDesc;
    use ext_rendering::SysRecordRendering;

    fn state<'a>(fields: &'a [FieldState], name: &str) -> &'a serde_json::Value {
        fields
            .iter()
            .find(|f| f.field_name == name)
            .map(|f| &f.data)
            .unwrap_or_else(|| panic!("field '{}' missing from get_state", name))
    }

    // ---------- shader descriptors ----------

    #[test]
    fn shader_desc_from_bits_parses_json() {
        let json = br#"{"shader_module_path":"shader_module_lit","textures":[{"label":"diffuse"}],"colors":[{"label":"base"}]}"#;
        let desc = ShaderDesc::from_bits(&json.to_vec());
        assert_eq!(desc.shader_module_path, "shader_module_lit");
        assert_eq!(desc.textures.len(), 1);
        assert_eq!(desc.colors.len(), 1);
    }

    #[test]
    fn shader_desc_from_bits_empty_lists() {
        let json = br#"{"shader_module_path":"shader_module_unlit","textures":[],"colors":[]}"#;
        let desc = ShaderDesc::from_bits(&json.to_vec());
        assert_eq!(desc.shader_module_path, "shader_module_unlit");
        assert!(desc.textures.is_empty());
        assert!(desc.colors.is_empty());
    }

    #[test]
    fn shader_desc_from_bits_rejects_garbage() {
        let garbage = b"this is not json".to_vec();
        let result = std::panic::catch_unwind(|| ShaderDesc::from_bits(&garbage));
        assert!(result.is_err());
    }

    // ---------- record ----------

    #[test]
    fn rendering_record_starts_empty() {
        let record = SysRecordRendering::default();
        assert!(record.draw_calls.is_empty());
    }

    #[test]
    fn rendering_record_state_shows_count() {
        let record = SysRecordRendering::default();
        let fields = record.get_state();
        assert_eq!(state(&fields, "num_calls").as_u64(), Some(0));
    }
}
