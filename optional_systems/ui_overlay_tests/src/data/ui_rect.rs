#[cfg(test)]
mod tests {
    use curio_core::Vector2;
    use std::f32::consts::FRAC_PI_2;
    use ui_overlay::UiRect;

    fn near(a: Vector2, b: Vector2) -> bool {
        (a.x - b.x).abs() < 0.001 && (a.y - b.y).abs() < 0.001
    }

    #[test]
    fn upright_unscaled_box_keeps_its_corners() {
        let rect = UiRect::new(10.0, 20.0, 40.0, 20.0);
        let corners = rect.turned_corners(0.0, Vector2::one());
        assert!(near(corners[0], Vector2::new(10.0, 20.0)));
        assert!(near(corners[2], Vector2::new(50.0, 40.0)));
    }

    #[test]
    fn quarter_turn_is_counter_clockwise_about_the_centre() {
        // centre (30, 30); the right edge's midpoint swings up on a y-down screen
        let rect = UiRect::new(10.0, 20.0, 40.0, 20.0);
        let turned = rect.turn_point(50.0, 30.0, FRAC_PI_2, Vector2::one());
        assert!(near(turned, Vector2::new(30.0, 10.0)), "{:?}", turned);
        assert!(near(rect.turn_point(30.0, 30.0, 1.0, Vector2::new(3.0, 2.0)), rect.center()));
    }

    #[test]
    fn scale_grows_the_box_about_its_centre() {
        let rect = UiRect::new(0.0, 0.0, 100.0, 50.0);
        let corners = rect.turned_corners(0.0, Vector2::new(2.0, 2.0));
        assert!(near(corners[0], Vector2::new(-50.0, -25.0)));
        assert!(near(corners[2], Vector2::new(150.0, 75.0)));
    }

    #[test]
    fn hit_test_follows_the_turned_box() {
        // a long thin bar turned upright: hits move from its old ends to above and below it
        let rect = UiRect::new(0.0, 45.0, 100.0, 10.0);
        assert!(rect.contains_turned(95.0, 50.0, 0.0, Vector2::one()));
        assert!(!rect.contains_turned(95.0, 50.0, FRAC_PI_2, Vector2::one()));
        assert!(rect.contains_turned(50.0, 5.0, FRAC_PI_2, Vector2::one()));
        assert!(rect.contains_turned(140.0, 50.0, 0.0, Vector2::new(2.0, 1.0)));
        assert!(!rect.contains_turned(50.0, 50.0, 0.0, Vector2::new(0.0, 1.0)));
    }
}
