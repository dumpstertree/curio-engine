#[cfg(test)]
mod tests {
    use curio_core::{Color, Vector2};
    use std::f32::consts::FRAC_PI_2;
    use ui_overlay::{UiDrawCall, UiKind, UiRect};

    #[test]
    fn a_new_draw_call_is_upright_and_turned_sets_both_fields() {
        let call = UiDrawCall::new(UiRect::new(0.0, 0.0, 10.0, 10.0), Color::white(), UiKind::Block, 2.5, String::new(), 12.0, 7, 0);
        assert_eq!((call.rotation, call.scale, call.depth), (0.0, Vector2::one(), 2.5));
        let call = call.turned(0.5, Vector2::new(2.0, 3.0));
        assert_eq!((call.rotation, call.scale), (0.5, Vector2::new(2.0, 3.0)));
    }

    #[test]
    fn corners_and_contains_use_the_turned_box() {
        let call = UiDrawCall::new(UiRect::new(0.0, 45.0, 100.0, 10.0), Color::white(), UiKind::Button, 0.0, String::new(), 12.0, 1, 0).turned(FRAC_PI_2, Vector2::one());
        let corners = call.corners();
        // the top-left corner of the upright bar lands bottom-left once it stands up
        assert!((corners[0].x - 45.0).abs() < 0.001 && (corners[0].y - 100.0).abs() < 0.001, "{:?}", corners[0]);
        assert!(call.contains(50.0, 95.0));
        assert!(!call.contains(95.0, 50.0));
    }
}
