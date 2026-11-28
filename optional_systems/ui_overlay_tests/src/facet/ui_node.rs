#[cfg(test)]
mod tests {
    use curio_core::Vector2;
    use gameplay::traits::field_override::FieldOverride;
    use std::f32::consts::FRAC_PI_2;
    use ui_overlay::{UiKind, UiNode, UiRect};

    #[test]
    fn a_built_node_starts_upright() {
        let node = UiNode::builder()
            .kind(UiKind::Block)
            .size(10.0, 10.0)
            .build();
        assert_eq!((node.rotation, node.scale, node.depth), (0.0, Vector2::one(), 0.0));
    }

    #[test]
    fn hit_test_uses_the_resolved_turn_and_scale() {
        let mut node = UiNode::builder().kind(UiKind::Button).build();
        node.rect = UiRect::new(0.0, 45.0, 100.0, 10.0);
        assert!(node.hit_test(95.0, 50.0));
        node.rotation = FRAC_PI_2;
        assert!(!node.hit_test(95.0, 50.0));
        assert!(node.hit_test(50.0, 95.0));
        node.rotation = 0.0;
        node.scale = Vector2::new(0.5, 1.0);
        assert!(!node.hit_test(95.0, 50.0));
    }

    #[test]
    fn the_image_field_takes_a_uid_and_round_trips() {
        let mut node = UiNode::builder().kind(UiKind::Image).build();
        assert_eq!(node.image, None);
        node.apply("image", " 42 ");
        assert_eq!(node.image, Some(42));
        let state = node.get_state();
        let image = state
            .iter()
            .find(|field| field.field_name == "image")
            .expect("image is serialized");
        assert_eq!(image.data, serde_json::Value::String("42".to_string()));
        node.apply("image", "");
        assert_eq!(node.image, None);
    }
}
