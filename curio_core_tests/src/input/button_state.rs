#[cfg(test)]
mod tests {
    use curio_core::ButtonState;

    // =========================================================
    // Defaults
    // =========================================================

    #[test]
    fn default_is_up_with_no_change_flags() {
        let mut button_state = ButtonState::default();
        assert!(button_state.is_down == false);
        assert!(button_state.went_down == false);
        assert!(button_state.went_up == false, "fresh state reports no change");
    }

    // =========================================================
    // Transitions
    // =========================================================

    #[test]
    fn press_reports_went_down_once() {
        let mut button_state = ButtonState::default();

        button_state.update(&true);
        assert!(button_state.is_down, "poll with down=true is down");
        assert!(button_state.went_down, "the press is reported on the poll it happened");
        assert!(button_state.went_up == false);

        button_state.update(&true);
        assert!(button_state.is_down);
        assert!(button_state.went_down == false, "holding does not repeat went_down");
        assert!(button_state.went_up == false);
    }

    #[test]
    fn release_reports_went_up_once() {
        let mut button_state = ButtonState::default();

        button_state.update(&true);

        button_state.update(&false);
        assert!(button_state.is_down == false);
        assert!(button_state.went_up, "the release is reported on the poll it happened");
        assert!(button_state.went_down == false);

        button_state.update(&false);
        assert!(button_state.is_down == false);
        assert!(button_state.went_up == false, "holding up does not repeat went_up");
    }

    #[test]
    fn re_press_after_release_reports_went_down() {
        let mut button_state = ButtonState::default();

        button_state.update(&true);
        button_state.update(&false);

        button_state.update(&true);
        assert!(button_state.is_down);
        assert!(button_state.went_down, "a second press is a fresh change");
        assert!(button_state.went_up == false);
    }

    #[test]
    fn poll_down_then_poll_up_in_same_call_order_is_stable() {
        // went_down/went_up are computed from the OLD is_down, never from
        // each other, so a down poll can never also report went_up.
        let mut button_state = ButtonState::default();

        button_state.update(&true);
        assert!(button_state.went_down && button_state.went_up == false);

        button_state.update(&false);
        assert!(button_state.went_up && button_state.went_down == false);
    }

    #[test]
    fn update_is_by_copy_not_reference() {
        // ButtonState is Copy: polling a stored copy must not move the
        // original, so the System Component keeps one owner.
        let mut button_state = ButtonState::default();
        let copy = button_state;

        button_state.update(&true);
        assert!(button_state.is_down);
        assert!(copy.is_down == false, "the original is untouched");
    }
}
