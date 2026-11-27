#[cfg(test)]
mod tests {
    use curio_core::Vector2;
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
}
