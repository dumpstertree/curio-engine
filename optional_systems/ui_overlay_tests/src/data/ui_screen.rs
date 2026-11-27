#[cfg(test)]
mod tests {
    use camera::camera_snapshot::CameraSnapshot;
    use curio_core::{GraphicsMapping, Vector2, Vector3};
    use ui_overlay::{UiRect, UiScreen};

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 0.0001
    }
    fn full() -> GraphicsMapping {
        GraphicsMapping::new(Vector2::zero(), Vector2::one())
    }

    #[test]
    fn ui_screen_pixels_follow_the_mapping_rect() {
        let screen = UiScreen::new(Vector2::new(1280.0, 720.0), &GraphicsMapping::new(Vector2::new(0.5, 0.0), Vector2::new(1.0, 1.0)), Vector2::new(1921.0, 1080.0));
        // same rounding as the 3D pass's Viewport::new
        assert_eq!(screen.pixels(), Vector2::new(961.0, 1080.0));
        assert!(close(screen.aspect(), 961.0 / 1080.0));
        let whole = UiScreen::new(Vector2::new(10.0, 10.0), &GraphicsMapping::default(), Vector2::new(800.0, 600.0));
        assert_eq!(whole.pixels(), Vector2::new(800.0, 600.0));
    }

    #[test]
    fn ui_screen_maps_corners_to_ndc() {
        let screen = UiScreen::new(Vector2::new(1280.0, 720.0), &full(), Vector2::new(1920.0, 1080.0));
        let top_left = screen.to_ndc(0.0, 0.0);
        let bottom_right = screen.to_ndc(1280.0, 720.0);
        assert!(close(top_left[0], -1.0) && close(top_left[1], 1.0));
        assert!(close(bottom_right[0], 1.0) && close(bottom_right[1], -1.0));
        let half = UiScreen::new(Vector2::new(640.0, 720.0), &GraphicsMapping::new(Vector2::new(0.5, 0.0), Vector2::one()), Vector2::new(1280.0, 720.0));
        assert!(close(half.to_ndc(0.0, 0.0)[0], 0.0));
        assert!(close(half.to_ndc(640.0, 0.0)[0], 1.0));
    }

    #[test]
    fn ui_screen_parks_the_whole_viewport_on_the_whole_frustum() {
        let screen = UiScreen::new(Vector2::new(1280.0, 720.0), &full(), Vector2::new(1280.0, 720.0));
        let camera = CameraSnapshot::new(Vector3::new(1.0, 2.0, 3.0));
        let Some(placed) = screen.place_in_world(UiRect::new(0.0, 0.0, 1280.0, 720.0), &camera, 2.0) else {
            panic!("a valid screen must place");
        };
        let full_h = 2.0 * 2.0 * (30.0f32).to_radians().tan();
        assert!(close(placed.scale.y, full_h));
        assert!(close(placed.scale.x, full_h * 1280.0 / 720.0));
        assert!(close(placed.position.x, 1.0) && close(placed.position.y, 2.0) && close(placed.position.z, 5.0));
    }

    #[test]
    fn ui_screen_frustum_aspect_comes_from_the_capture_not_the_viewport() {
        // an authored 1280x720 design space stretched onto a 2.4:1 window
        let screen = UiScreen::new(Vector2::new(1280.0, 720.0), &full(), Vector2::new(1920.0, 800.0));
        let camera = CameraSnapshot::new(Vector3::zero());
        let Some(placed) = screen.place_in_world(UiRect::new(0.0, 0.0, 640.0, 720.0), &camera, 1.0) else {
            panic!("a valid screen must place");
        };
        let full_h = 2.0 * (30.0f32).to_radians().tan();
        let full_w = full_h * 2.4;
        assert!(close(placed.scale.x, full_w * 0.5));
        assert!(close(placed.scale.y, full_h));
        // curio's screen-right is -X, so the left half sits towards +X
        assert!(close(placed.position.x, full_w * 0.25));
        assert!(close(placed.position.y, 0.0));
    }

    #[test]
    fn ui_screen_refuses_unusable_input() {
        let camera = CameraSnapshot::new(Vector3::zero());
        let empty = UiScreen::new(Vector2::zero(), &full(), Vector2::new(1280.0, 720.0));
        assert!(
            empty
                .place_in_world(UiRect::new(0.0, 0.0, 1.0, 1.0), &camera, 1.0)
                .is_none()
        );
        let screen = UiScreen::new(Vector2::new(1280.0, 720.0), &full(), Vector2::new(1280.0, 720.0));
        assert!(
            screen
                .place_in_world(UiRect::new(0.0, 0.0, 1.0, 1.0), &camera, 0.0)
                .is_none()
        );
    }
}
