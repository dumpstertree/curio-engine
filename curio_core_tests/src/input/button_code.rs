#[cfg(test)]
mod tests {
    use curio_core::ButtonCode;

    // =========================================================
    // Identity
    // =========================================================

    #[test]
    fn distinct_codes_are_not_equal() {
        assert!(ButtonCode::MousePrimary != ButtonCode::MouseSecondary);
        assert!(ButtonCode::KeyA != ButtonCode::KeyB);
        assert_eq!(ButtonCode::KeyA, ButtonCode::KeyA);
    }

    #[test]
    fn codes_are_cloneable_and_usable_as_keys() {
        let code = ButtonCode::Space;
        let cloned = code.clone();
        assert_eq!(code, cloned);

        let map_key = ButtonCode::ShiftLeft;
        let copied_key = map_key;
        assert_eq!(map_key, copied_key, "ButtonCode is Copy: HashMap keys can be moved and still matched");
    }

    // =========================================================
    // Ordinal stability
    // serde encodes non_exhaustive enums by declaration order, so the
    // derived Ord (which follows that order) is pinned here for the
    // codes that cross the network via InputMapping.
    // =========================================================

    #[test]
    fn declaration_order_is_stable() {
        assert!(ButtonCode::MousePrimary < ButtonCode::MouseSecondary);
        assert!(ButtonCode::MouseSecondary < ButtonCode::KeyA);
        assert!(ButtonCode::KeyA < ButtonCode::KeyB);
        assert!(ButtonCode::ShiftLeft < ButtonCode::Space);
    }
}
