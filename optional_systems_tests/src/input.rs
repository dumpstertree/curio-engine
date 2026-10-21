#[cfg(test)]
mod tests {
    use curio_core::Vector2;
    use input::SysRecordInput;

    #[test]
    fn unmapped_actions_are_quiet() {
        let input = SysRecordInput::default();
        assert!(!input.is_down("jump"));
        assert!(!input.is_pressed("jump"));
        assert!(!input.is_released("jump"));
        assert_eq!(input.axis("move"), Vector2::zero());
        assert!(input.first_mapped().is_none());
    }
}
