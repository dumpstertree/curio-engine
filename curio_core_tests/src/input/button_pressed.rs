#[cfg(test)]
mod tests {
    use curio_core::ButtonPressed;

    #[test]
    fn default_is_down() {
        assert_eq!(ButtonPressed::default(), ButtonPressed::Down, "a resting button reads down (not pressed)");
    }

    #[test]
    fn down_and_up_are_distinct() {
        assert!(ButtonPressed::Down != ButtonPressed::Up);
        assert_eq!(ButtonPressed::Down, ButtonPressed::Down);
    }
}
