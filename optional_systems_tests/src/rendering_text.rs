#[cfg(test)]
mod tests {
    use curio_core::{Color, FieldState, Vector2};
    use gameplay::traits::field_override::FieldOverride;
    use rendering_text::{AligmentHorizontal, AligmentVertical, RendererText};

    fn state<'a>(fields: &'a [FieldState], name: &str) -> &'a serde_json::Value {
        fields
            .iter()
            .find(|f| f.field_name == name)
            .map(|f| &f.data)
            .unwrap_or_else(|| panic!("field '{}' missing from get_state", name))
    }

    // most RendererText fields are private, so everything is asserted through get_state

    // ---------- defaults ----------

    #[test]
    fn defaults() {
        let text = RendererText::default();
        let fields = text.get_state();
        assert_eq!(state(&fields, "contents").as_str(), Some("Lorum ipsum..."));
        // f32 -> JSON -> f64 widens 0.05, so compare with tolerance
        let font_size = state(&fields, "font_size").as_f64().unwrap() as f32;
        assert!((font_size - 0.05).abs() < 0.0001, "font_size {}", font_size);
        assert_eq!(state(&fields, "enabled").as_bool(), Some(true));
        assert_eq!(state(&fields, "align_horizontal").as_str(), Some("center"));
        assert_eq!(state(&fields, "align_vertical").as_str(), Some("center"));
    }

    // ---------- builder ----------

    #[test]
    fn builder_sets_every_field() {
        let text = RendererText::builder()
            .contents("hello")
            .font_size(1.5)
            .bounds(Vector2::new(2.0, 3.0))
            .enabled(false)
            .tint(Color::new_hex("ff0000"))
            .horizontal_alignment(AligmentHorizontal::Left)
            .vertical_alignment(AligmentVertical::Top)
            .build();
        let fields = text.get_state();
        assert_eq!(state(&fields, "contents").as_str(), Some("hello"));
        assert_eq!(state(&fields, "font_size").as_f64(), Some(1.5));
        assert_eq!(state(&fields, "bounds")["x"].as_f64(), Some(2.0));
        assert_eq!(state(&fields, "bounds")["y"].as_f64(), Some(3.0));
        assert_eq!(state(&fields, "enabled").as_bool(), Some(false));
        assert_eq!(state(&fields, "align_horizontal").as_str(), Some("left"));
        assert_eq!(state(&fields, "align_vertical").as_str(), Some("top"));
        assert_eq!(text.tint, Color::new_hex("ff0000"));
    }

    // ---------- apply ----------

    #[test]
    fn apply_contents() {
        let mut text = RendererText::default();
        text.apply("contents", "42");
        assert_eq!(state(&text.get_state(), "contents").as_str(), Some("42"));
        text.apply("contents", "42");
        assert_eq!(state(&text.get_state(), "contents").as_str(), Some("42"));
    }

    #[test]
    fn apply_font_size_valid_and_bad() {
        let mut text = RendererText::builder().font_size(2.0).build();
        text.apply("font_size", "0.75");
        assert_eq!(state(&text.get_state(), "font_size").as_f64(), Some(0.75));
        // a bad parse must not zero the font size
        text.apply("font_size", "big");
        assert_eq!(state(&text.get_state(), "font_size").as_f64(), Some(0.75));
    }

    #[test]
    fn apply_bounds_valid_and_bad() {
        let mut text = RendererText::default();
        text.apply("bounds", "(2, 4)");
        assert_eq!(state(&text.get_state(), "bounds")["x"].as_f64(), Some(2.0));
        // a bad parse must not zero the bounds
        text.apply("bounds", "(2, 4, 0)");
        text.apply("bounds", "garbage");
        let fields = text.get_state();
        assert_eq!(state(&fields, "bounds")["x"].as_f64(), Some(2.0));
        assert_eq!(state(&fields, "bounds")["y"].as_f64(), Some(4.0));
    }

    #[test]
    fn apply_tint_valid_and_bad() {
        let mut text = RendererText::default();
        text.apply("tint", "#00ff00");
        assert_eq!(text.tint, Color::new_hex("00ff00"));
        text.apply("tint", "not a color");
        assert_eq!(text.tint, Color::new_hex("00ff00"));
    }

    #[test]
    fn apply_alignments() {
        let mut text = RendererText::default();
        text.apply("align_horizontal", "left");
        text.apply("align_vertical", "top");
        let fields = text.get_state();
        assert_eq!(state(&fields, "align_horizontal").as_str(), Some("left"));
        assert_eq!(state(&fields, "align_vertical").as_str(), Some("top"));

        text.apply("align_horizontal", "RIGHT");
        text.apply("align_vertical", "bottom");
        let fields = text.get_state();
        assert_eq!(state(&fields, "align_horizontal").as_str(), Some("right"));
        assert_eq!(state(&fields, "align_vertical").as_str(), Some("bottom"));

        // anything else falls back to center
        text.apply("align_horizontal", "middle-ish");
        text.apply("align_vertical", "middle-ish");
        let fields = text.get_state();
        assert_eq!(state(&fields, "align_horizontal").as_str(), Some("center"));
        assert_eq!(state(&fields, "align_vertical").as_str(), Some("center"));
    }

    #[test]
    fn apply_enabled() {
        let mut text = RendererText::default();
        text.apply("enabled", "false");
        assert!(!text.enabled);
    }

    // ---------- get_state round trips ----------

    #[test]
    fn get_state_values_reapply() {
        let text = RendererText::builder()
            .contents("round")
            .font_size(3.0)
            .horizontal_alignment(AligmentHorizontal::Right)
            .vertical_alignment(AligmentVertical::Bottom)
            .build();

        let mut other = RendererText::default();
        for field in text.get_state() {
            other.apply(&field.field_name.clone(), &field.data.to_string().trim_matches('"').to_string());
        }
        let fields = other.get_state();
        assert_eq!(state(&fields, "contents").as_str(), Some("round"));
        assert_eq!(state(&fields, "font_size").as_f64(), Some(3.0));
        assert_eq!(state(&fields, "align_horizontal").as_str(), Some("right"));
        assert_eq!(state(&fields, "align_vertical").as_str(), Some("bottom"));
    }
}
