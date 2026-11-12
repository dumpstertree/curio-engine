#[cfg(test)]
mod tests {
    use curio_core::{FieldState, LedgerEntry, RecordCommon, RecordOverride};
    use serde::Deserialize;
    use serde::Serialize;
    use std::rc::Rc;

    #[derive(Clone, Serialize, Deserialize)]
    struct EntryProbe {
        pub value: i32,
    }
    impl RecordCommon for EntryProbe {
        fn name(&self) -> String {
            String::from("EntryProbe")
        }
        fn id() -> i32 {
            9501
        }
    }
    impl RecordOverride for EntryProbe {
        fn set_state(&mut self, _field: &str, _val: &str) {}
        fn get_state(&self) -> Vec<FieldState> {
            vec![FieldState::new("value", self.value)]
        }
    }

    fn boxed_probe(value: i32) -> Box<dyn RecordCommon> {
        Box::new(EntryProbe { value })
    }

    // =========================================================
    // Construction
    // =========================================================

    #[test]
    fn new_seeds_read_from_write() {
        let entry = LedgerEntry::new(boxed_probe(5));

        let Some(read) = entry.read.downcast_ref::<EntryProbe>() else { panic!("expected Some") };
        assert_eq!(read.value, 5);
    }

    // =========================================================
    // Read/write separation
    // =========================================================

    #[test]
    fn write_edits_are_not_reflected_in_read_until_sync() {
        let mut entry = LedgerEntry::new(boxed_probe(1));

        let Some(record) = entry.write.downcast_mut::<EntryProbe>() else { panic!("expected Some") };
        record.value = 9;

        let Some(read) = entry.read.downcast_ref::<EntryProbe>() else { panic!("expected Some") };
        assert_eq!(read.value, 1, "read must lag behind write until sync_read");
    }

    #[test]
    fn sync_read_copies_write_into_read() {
        let mut entry = LedgerEntry::new(boxed_probe(1));

        let Some(record) = entry.write.downcast_mut::<EntryProbe>() else { panic!("expected Some") };
        record.value = 9;

        entry.sync_read();

        let Some(read) = entry.read.downcast_ref::<EntryProbe>() else { panic!("expected Some") };
        assert_eq!(read.value, 9, "read must match write after sync_read");
    }

    #[test]
    fn sync_read_replaces_previous_read_rc() {
        let mut entry = LedgerEntry::new(boxed_probe(2));
        let first_read = Rc::as_ptr(&entry.read);

        entry.sync_read();

        assert!(Rc::as_ptr(&entry.read) != first_read);
    }

    // =========================================================
    // Cloning
    // =========================================================

    #[test]
    fn clone_reseeds_read_from_write() {
        let mut entry = LedgerEntry::new(boxed_probe(1));

        let Some(record) = entry.write.downcast_mut::<EntryProbe>() else { panic!("expected Some") };
        record.value = 8;

        let copy = entry.clone();

        let Some(write) = copy.write.downcast_ref::<EntryProbe>() else { panic!("expected Some") };
        assert_eq!(write.value, 8);

        let Some(read) = copy.read.downcast_ref::<EntryProbe>() else { panic!("expected Some") };
        assert_eq!(read.value, 8, "clone() must reseed read from write, not from the old read");
    }

    // =========================================================
    // Field reporting
    // =========================================================

    #[test]
    fn get_state_reports_value_field() {
        let probe = EntryProbe { value: 12 };
        let fields = probe.get_state();

        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].field_name, "value");
    }
}
