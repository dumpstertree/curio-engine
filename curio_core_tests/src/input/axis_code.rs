#[cfg(test)]
mod tests {
    use curio_core::AxisCode;

    #[test]
    fn distinct_codes_are_not_equal() {
        assert!(AxisCode::Cursor != AxisCode::Gamepad0StickLeft);
        assert_eq!(AxisCode::Cursor, AxisCode::Cursor);
    }

    #[test]
    fn codes_are_cloneable() {
        let code = AxisCode::Gamepad0TriggerLeft;
        let cloned = code.clone();
        assert_eq!(code, cloned);
    }

    #[test]
    fn declaration_order_is_stable() {
        assert!(AxisCode::Cursor < AxisCode::Gamepad0StickLeft);
        assert!(AxisCode::Gamepad0StickLeft < AxisCode::Gamepad0StickRight);
        assert!(AxisCode::Gamepad0TriggerLeft < AxisCode::Gamepad0TriggerRight);
    }
}
