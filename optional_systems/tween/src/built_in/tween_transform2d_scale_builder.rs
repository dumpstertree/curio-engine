use curio_core::Vector3;

use crate::{
    built_in::tween_transform2d_scale::TweenTransform2DScale,
    data::{curve::TweenCurve, tween_state::TweenState},
};

pub struct TweenTransform2DScaleBuilder {
    pub(crate) p0: Vector3,
    pub(crate) p1: Vector3,
    pub(crate) delay: f32,
    pub(crate) duration: f32,
    pub(crate) curve: TweenCurve,
    pub(crate) complete: Option<Box<dyn FnOnce() + Send + Sync>>,
}
impl TweenTransform2DScaleBuilder {
    pub fn p0(mut self, p0: Vector3) -> Self {
        self.p0 = p0;
        self
    }
    pub fn p1(mut self, p1: Vector3) -> Self {
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
    pub fn build(self) -> TweenTransform2DScale {
        TweenTransform2DScale {
            state: TweenState::new(self.duration, self.delay, self.curve, self.complete),
            p0: self.p0,
            p1: self.p1,
            p_t: self.p0,
        }
    }
}
