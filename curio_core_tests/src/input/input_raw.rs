#[cfg(test)]
mod tests {
    use curio_core::InputRaw;
    use curio_core::ButtonState;
    use curio_core::AxisState;
    use curio_core::ButtonCode;
    use curio_core::AxisCode;
    use curio_core::Vector2;
    use std::collections::HashMap;

    // =========================================================
    // Fresh state
    // =========================================================

    #[test]
    fn new_is_empty_but_reads_default_states() {
        let input_raw = InputRaw::new();

        let jump = input_raw.get_button(&ButtonCode::KeyA);
        assert_eq!(jump, ButtonState::default(), "unknown codes read as default");

        let move_state = input_raw.get_axis(&AxisCode::Cursor);
        assert_eq!(move_state, AxisState::default());
    }

    // =========================================================
    // Update
    // =========================================================

    #[test]
    fn update_records_only_codes_present_in_the_poll() {
        let input_raw = InputRaw::new();

        let raw_buttons: HashMap<ButtonCode, bool> = HashMap::new();
        raw_buttons.insert(ButtonCode::KeyA, true);
        let raw_axis: HashMap<AxisCode, Vector2> = HashMap::new();
        raw_axis.insert(AxisCode::Cursor, Vector2::new(1.0, 0.0));
        input_raw.update(&raw_buttons, &raw_axis);

        let jump = input_raw.get_button(&ButtonCode::KeyA);
        assert!(jump.is_down);
        assert!(jump.went_down, "the press is visible on the poll it happened");

        let move_state = input_raw.get_axis(&AxisCode::Cursor);
        assert_eq!(move_state.position, Vector2::new(1.0, 0.0));
        assert_eq!(move_state.delta, Vector2::new(1.0, 0.0));
    }

    #[test]
    fn press_then_release_across_polls_reports_went_up() {
        let input_raw = InputRaw::new();

        let raw_buttons: HashMap<ButtonCode, bool> = HashMap::new();
        raw_buttons.insert(ButtonCode::KeyA, true);
        let raw_axis: HashMap<AxisCode, Vector2> = HashMap::new();
        input_raw.update(&raw_buttons, &raw_axis);

        let raw_buttons: HashMap<ButtonCode, bool> = HashMap::new();
        raw_buttons.insert(ButtonCode::KeyA, false);
        input_raw.update(&raw_buttons, &raw_axis);

        let jump = input_raw.get_button(&ButtonCode::KeyA);
        assert!(jump.is_down == false);
        assert!(jump.went_up, "the release is visible on the poll it happened");
    }

    #[test]
    fn codes_absent_from_a_poll_keep_their_last_state() {
        // By design InputRaw::update iterates the POLL, not a fixed mapping:
        // codes that disappear from the poll keep their previous state and
        // are NOT forced to released. (Contrast InputMapped, which forces.)
        let input_raw = InputRaw::new();

        let raw_buttons: HashMap<ButtonCode, bool> = HashMap::new();
        raw_buttons.insert(ButtonCode::KeyA, true);
        let raw_axis: HashMap<AxisCode, Vector2> = HashMap::new();
        input_raw.update(&raw_buttons, &raw_axis);

        let raw_buttons: HashMap<ButtonCode, bool> = HashMap::new();
        raw_buttons.insert(ButtonCode::KeyB, false);
        input_raw.update(&raw_buttons, &raw_axis);

        let jump = input_raw.get_button(&ButtonCode::KeyA);
        assert!(jump.is_down, "KeyA was absent from the second poll and keeps its pressed state");
    }

    #[test]
    fn codes_never_seen_read_default_not_stale() {
        let input_raw = InputRaw::new();

        let raw_buttons: HashMap<ButtonCode, bool> = HashMap::new();
        raw_buttons.insert(ButtonCode::KeyA, true);
        let raw_axis: HashMap<AxisCode, Vector2> = HashMap::new();
        input_raw.update(&raw_buttons, &raw_axis);

        let never = input_raw.get_button(&ButtonCode::KeyB);
        assert_eq!(never, ButtonState::default(), "codes never polled are default, not stale copies");
    }

    #[test]
    fn axis_delta_accumulates_across_polls() {
        let input_raw = InputRaw::new();

        let raw_axis: HashMap<AxisCode, Vector2> = HashMap::new();
        raw_axis.insert(AxisCode::Cursor, Vector2::new(1.0, 0.0));
        let raw_buttons: HashMap<ButtonCode, bool> = HashMap::new();
        input_raw.update(&raw_buttons, &raw_axis);

        let raw_axis: HashMap<AxisCode, Vector2> = HashMap::new();
        raw_axis.insert(AxisCode::Cursor, Vector2::new(1.0, 3.0));
        let raw_buttons: HashMap<ButtonCode, bool> = HashMap::new();
        input_raw.update(&raw_buttons, &raw_axis);

        let move_state = input_raw.get_axis(&AxisCode::Cursor);
        assert_eq!(move_state.position, Vector2::new(1.0, 3.0));
        assert_eq!(move_state.delta, Vector2::new(0.0, 3.0));
    }

    #[test]
    fn default_fallback_stays_untouched_while_other_codes_poll() {
        // get_button/get_axis hand out references into the container and a
        // shared default view; polling other codes must not disturb it.
        let input_raw = InputRaw::new();
        let never = input_raw.get_button(&ButtonCode::KeyB);

        let raw_buttons: HashMap<ButtonCode, bool> = HashMap::new();
        raw_buttons.insert(ButtonCode::KeyA, true);
        let raw_axis: HashMap<AxisCode, Vector2> = HashMap::new();
        input_raw.update(&raw_buttons, &raw_axis);

        assert_eq!(never, ButtonState::default(), "the shared default must stay untouched");
    }
}
