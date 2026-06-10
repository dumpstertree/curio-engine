use crate::{
    built_in::tween_transform3d_rotation::TweenTransform3DRotation,
    data::{curve::TweenCurve, tween_state::TweenState},
};
use curio_core::Quaternion;

pub struct TweenTransform3DRotationBuilder {
    pub(crate) p0: Quaternion,
    pub(crate) p1: Quaternion,
    pub(crate) delay: f32,
    pub(crate) duration: f32,
    pub(crate) curve: TweenCurve,
    pub(crate) complete: Option<Box<dyn FnOnce() + Send + Sync>>,
}
impl TweenTransform3DRotationBuilder {
    pub fn p0(mut self, p0: Quaternion) -> Self {
        self.p0 = p0;
        self
    }
    pub fn p1(mut self, p1: Quaternion) -> Self {
        self.p1 = p1;
        self
    }
    pub fn curve(mut self, curve: TweenCurve) -> Self {
        self.curve = curve;
        self
    }
    pub fn duration(mut self, duration: f32) -> Self {
        self.duration = duration;
        self
    }
    pub fn delay(mut self, delay: f32) -> Self {
        self.delay = delay;
        self
    }
    pub fn on_complete(mut self, complete: Option<Box<dyn FnOnce() + Send + Sync>>) -> Self {
        self.complete = complete;
        self
    }
    pub fn build(self) -> TweenTransform3DRotation {
        TweenTransform3DRotation {
            state: TweenState::new(self.duration, self.delay, self.curve, self.complete),
            p0: self.p0,
            p1: self.p1,
            p_t: self.p0,
        }
    }
}
