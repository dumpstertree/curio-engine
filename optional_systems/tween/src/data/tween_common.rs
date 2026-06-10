use std::any::Any;

use crate::data::tween_target::TweenTarget;

pub trait TweenCommon: Any + Send + Sync + TweenClone {
    fn as_any(&self) -> &dyn Any;
    fn update(&mut self, dt: f32) -> bool;
    fn target(&self) -> TweenTarget;
}
pub trait TweenClone {
    fn clone_box(&self) -> Box<dyn TweenCommon>;
}

impl<T> TweenClone for T
where
    T: 'static + TweenCommon + Clone,
{
    fn clone_box(&self) -> Box<dyn TweenCommon> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn TweenCommon> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
