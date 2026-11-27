#[cfg(test)]
mod tests {
    use curio_core::{RecordOverride, Vector2};
    use ui_overlay::SysRecordUi;

    #[test]
    fn ui_record_fills_an_unauthored_viewport_and_follows_resizes() {
        let mut record = SysRecordUi::default();
        assert!(!record.viewport_is_set());
        assert!(record.fill_viewport(Vector2::new(1920.0, 1080.0)));
        assert_eq!(record.viewport, Vector2::new(1920.0, 1080.0));
        // the engine's own value still counts as unauthored, so a resize refreshes it
        assert!(!record.viewport_is_set());
        assert!(record.fill_viewport(Vector2::new(800.0, 600.0)));
        assert_eq!(record.viewport, Vector2::new(800.0, 600.0));
        assert!(!record.fill_viewport(Vector2::new(800.0, 600.0)));
    }

    #[test]
    fn ui_record_keeps_an_authored_viewport() {
        let mut record = SysRecordUi::default();
        record.fill_viewport(Vector2::new(1920.0, 1080.0));
        record.viewport = Vector2::new(1280.0, 720.0);
        assert!(record.viewport_is_set());
        assert!(!record.fill_viewport(Vector2::new(800.0, 600.0)));
        assert_eq!(record.viewport, Vector2::new(1280.0, 720.0));
        let mut overridden = SysRecordUi::default();
        overridden.set_state("width", "640");
        overridden.set_state("height", "360");
        assert!(!overridden.fill_viewport(Vector2::new(1920.0, 1080.0)));
        assert_eq!(overridden.viewport, Vector2::new(640.0, 360.0));
    }
}
