#[cfg(test)]
mod tests {
    use curio_core::InputMapped;
    use curio_core::InputMapping;
    use curio_core::ButtonState;
    use curio_core::AxisState;
    use curio_core::ButtonCode;
    use curio_core::AxisCode;
    use curio_core::Vector2;
    use std::collections::HashMap;

    #[test]
    fn mapping_seeds_one_state_per_uid() {
        let mapping = InputMapping::new(
            [("jump", ButtonCode::KeyA), ("fire", ButtonCode::MousePrimary)],
            [("move", AxisCode::Cursor)],
        );
        let input_mapped = InputMapped::new(mapping);

        let Some(jump) = input_mapped.get_button("jump");
        assert_eq!(jump, ButtonState::default(), "mapped uids start fresh");

        let Some(move_state) = input_mapped.get_axis("move");
        assert_eq!(move_state, AxisState::default());
    }

    #[test]
    fn unknown_uids_are_none_but_defaults_are_available() {
        let mapping = InputMapping::new([("jump", ButtonCode::KeyA)], []);
        let input_mapped = InputMapped::new(mapping);

        let unknown = input_mapped.get_button("nope");
        assert(unknown.is_none(), "uids outside the mapping report none");

        let fallback = input_mapped.get_button_or_default("nope");
        assert_eq!(fallback, ButtonState::default(), "the or_default view returns the shared default");

        let unknown_axis = input_mapped.get_axis("nope");
        assert(unknown_axis.is_none());

        let fallback_axis = input_mapped.get_axis_or_default("nope");
        assert_eq!(fallback_axis, AxisState::default());
    }

    // =========================================================
    // Update
    // =========================================================

    #[test]
    fn update_drives_states_through_mapped_codes() {
        let mapping = InputMapping::new([("jump", ButtonCode::KeyA)], [("move", AxisCode::Cursor)]);
        let input_mapped = InputMapped::new(mapping);

        let raw_buttons: HashMap<ButtonCode, bool> = HashMap::new();
        raw_buttons.insert(ButtonCode::KeyA, true);
        let raw_axis: HashMap<AxisCode, Vector2> = HashMap::new();
        raw_axis.insert(AxisCode::Cursor, Vector2::new(2.0, 0.0));
        input_mapped.update(&raw_buttons, &raw_axis);

        let Some(jump) = input_mapped.get_button("jump");
        assert!(jump.is_down);
        assert!(jump.went_down);

        let Some(move_state) = input_mapped.get_axis("move");
        assert_eq!(move_state.position, Vector2::new(2.0, 0.0));
        assert_eq!(move_state.delta, Vector2::new(2.0, 0.0));
    }

    #[test]
    fn mapped_uid_forces_absent_codes_to_released() {
        // Contrast with InputRaw: a mapped uid polls every poll. If its
        // raw code is absent, update() feeds it false, so holding a key
        // that stops being reported reads as a release.
        let mapping = InputMapping::new([("jump", ButtonCode::KeyA)], []);
        let input_mapped = InputMapped::new(mapping);

        let raw_buttons: HashMap<ButtonCode, bool> = HashMap::new();
        raw_buttons.insert(ButtonCode::KeyA, true);
        let raw_axis: HashMap<AxisCode, Vector2> = HashMap::new();
        input_mapped.update(&raw_buttons, &raw_axis);

        let empty_buttons: HashMap<ButtonCode, bool> = HashMap::new();
        let empty_axis: HashMap<AxisCode, Vector2> = HashMap::new();
        input_mapped.update(&empty_buttons, &empty_axis);

        let Some(jump) = input_mapped.get_button("jump");
        assert!(jump.is_down == false, "the absent raw code was forced to released");
        assert!(jump.went_up, "and the release was recorded");
    }

    #[test]
    fn absent_axis_is_forced_to_zero() {
        let mapping = InputMapping::new([], [("move", AxisCode::Cursor)]);
        let input_mapped = InputMapped::new(mapping);

        let raw_buttons: HashMap<ButtonCode, bool> = HashMap::new();
        let raw_axis: HashMap<AxisCode, Vector2> = HashMap::new();
        raw_axis.insert(AxisCode::Cursor, Vector2::new(1.0, 2.0));
        input_mapped.update(&raw_buttons, &raw_axis);

        let empty_buttons: HashMap<ButtonCode, bool> = HashMap::new();
        let empty_axis: HashMap<AxisCode, Vector2> = HashMap::new();
        input_mapped.update(&empty_buttons, &empty_axis);

        let Some(move_state) = input_mapped.get_axis("move");
        assert_eq!(move_state.position, Vector2::zero(), "an absent raw axis is polled as zero");
        assert_eq!(move_state.delta, Vector2::new(-1.0, -2.0), "the snap back to zero shows as motion");
    }

    #[test]
    fn unmapped_raw_codes_are_ignored() {
        // Raw codes that no mapping references never reach any uid.
        let mapping = InputMapping::new([("jump", ButtonCode::KeyA)], []);
        let input_mapped = InputMapped::new(mapping);

        let raw_buttons: HashMap<ButtonCode, bool> = HashMap::new();
        raw_buttons.insert(ButtonCode::MousePrimary, true);
        let raw_axis: HashMap<AxisCode, Vector2> = HashMap::new();
        input_mapped.update(&raw_buttons, &raw_axis);

        let Some(jump) = input_mapped.get_button("jump");
        assert(jump.is_down == false, "an unmapped code cannot press a mapped uid");
    }

    #[test]
    fn two_polls_keep_the_change_flags_per_poll() {
        let mapping = InputMapping::new([("jump", ButtonCode::KeyA)], []);
        let input_mapped = InputMapped::new(mapping);

        let raw_buttons: HashMap<ButtonCode, bool> = HashMap::new();
        raw_buttons.insert(ButtonCode::KeyA, true);
        let raw_axis: HashMap<AxisCode, Vector2> = HashMap::new();
        input_mapped.update(&raw_buttons, &raw_axis);

        let raw_buttons: HashMap<ButtonCode, bool> = HashMap::new();
        raw_buttons.insert(ButtonCode::KeyA, true);
        input_mapped.update(&raw_buttons, &raw_axis);

        let Some(jump) = input_mapped.get_button("jump");
        assert!(jump.is_down);
        assert!(jump.went_down == false, "holding does not repeat went_down");
    }
}
