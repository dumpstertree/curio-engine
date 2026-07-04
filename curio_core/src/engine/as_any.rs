pub trait AsAny {
    fn as_any(&self) -> Option<&dyn std::any::Any> {
        None
    }
    fn as_mut_any(&mut self) -> Option<&mut dyn std::any::Any> {
        None
    }
}
