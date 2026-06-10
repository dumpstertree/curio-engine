use std::any::TypeId;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct TweenTarget {
    pub facet: TypeId,
    pub field: &'static str,
}

impl TweenTarget {
    pub fn new<T: 'static>(field: &'static str) -> Self {
        Self { facet: TypeId::of::<T>(), field }
    }
}
