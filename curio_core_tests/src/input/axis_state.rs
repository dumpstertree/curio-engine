#[cfg(test)]
mod tests {
    use curio_core::AxisState;
    use curio_core::Vector2;

    // =========================================================
    // Defaults
    // =========================================================

    #[test]
    fn default_is_zero_position_and_delta() {
        let state = AxisState::default();
        assert_eq!(state.position, Vector2::zero());
        assert_eq!(state.delta, Vector2::zero());
    }

    // =========================================================
    // Update
    // =========================================================

    #[test]
    fn first_update_sets_position_and_delta_from_zero() {
        let mut axis_state = AxisState::default();

        axis_state.update(Vector2::new(1.0, 0.0));
        assert_eq!(axis_state.position, Vector2::new(1.0, 0.0));
        assert_eq!(axis_state.delta, Vector2::new(1.0, 0.0), "delta is measured from the previous position");
    }

    #[test]
    fn delta_is_the_motion_since_last_poll() {
        let mut axis_state = AxisState::default();

        axis_state.update(Vector2::new(1.0, 0.0));
        axis_state.update(Vector2::new(1.0, 2.0));
        assert_eq!(axis_state.position, Vector2::new(1.0, 2.0));
        assert_eq!(axis_state.delta, Vector2::new(0.0, 2.0));

        axis_state.update(Vector2::new(0.0, 0.0));
        assert_eq!(axis_state.position, Vector2::zero());
        assert_eq!(axis_state.delta, Vector2::new(-1.0, -2.0), "negative delta records reverse motion");
    }

    #[test]
    fn holding_position_reports_zero_delta() {
        let mut axis_state = AxisState::default();

        axis_state.update(Vector2::new(0.5, -0.5));
        axis_state.update(Vector2::new(0.5, -0.5));
        assert_eq!(axis_state.delta, Vector2::zero(), "no motion means no delta");
    }

    #[test]
    fn update_is_by_copy_not_reference() {
        // AxisState is Copy (its Vector2 is Copy): polling a stored copy
        // must not move the original.
        let mut axis_state = AxisState::default();
        let copy = axis_state;

        axis_state.update(Vector2::new(1.0, 0.0));
        assert_eq!(axis_state.position, Vector2::new(1.0, 0.0));
        assert_eq!(copy.position, Vector2::zero(), "the original is untouched");
    }
}
