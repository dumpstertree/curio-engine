#[derive(Clone)]
pub struct CameraIndex {
    pub index: usize,
}

impl CameraIndex {
    pub fn set_index(mut self, index: usize) -> CameraIndex {
        self.index = index;
        self
    }
    pub fn default() -> CameraIndex {
        CameraIndex { index: 0 }
    }
}
