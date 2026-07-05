use crate::RecordCommon;
use std::rc::Rc;

/// An entry in a Ledger containing the read/write data for a Record
pub struct LedgerEntry {
    pub write: Box<dyn RecordCommon>,
    pub read: Rc<dyn RecordCommon>,
}

impl LedgerEntry {
    /// Build from a boxed value. Clones once to seed the read Rc.
    pub fn new(value: Box<dyn RecordCommon>) -> Self {
        let read = Rc::from(value.clone_box());
        LedgerEntry { write: value, read }
    }

    /// Synch cread value to write value
    pub fn sync_read(&mut self) {
        self.read = Rc::from(self.write.clone_box());
    }
}

impl Clone for LedgerEntry {
    fn clone(&self) -> Self {
        LedgerEntry {
            write: self.write.clone_box(),
            read: Rc::from(self.write.clone_box()),
        }
    }
}
