use crate::{
    built_in::tween_transform3d_scale_builder::TweenTransform3DScaleBuilder,
    data::{curve::TweenCurve, tween_common::TweenCommon, tween_state::TweenState, tween_target::TweenTarget},
};
use curio_core::Vector3;
use gameplay::built_in::facet::transform::transform2d::Transform2D;
use std::any::Any;

#[derive(Clone)]
pub struct TweenTransform2DScale {
    pub(crate) state: TweenState,
    pub(crate) p0: Vector3,
    pub(crate) p1: Vector3,
    pub(crate) p_t: Vector3,
}
impl TweenTransform2DScale {
    pub fn builder() -> TweenTransform3DScaleBuilder {
        TweenTransform3DScaleBuilder {
            p0: Vector3::zero(),
            p1: Vector3::zero(),
            delay: 0.0,
            duration: 1.0,
            complete: None,
            curve: TweenCurve::Linear,
        }
    }
}
impl TweenCommon for TweenTransform2DScale {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn update(&mut self, dt: f32) -> bool {
        if let Some(t) = self.state.advance(dt) {
            self.p_t = Vector3::lerp(self.p0, self.p1, t);
        }
        self.state.done()
    }

    fn target(&self) -> TweenTarget {
        TweenTarget::new::<Transform2D>("scale")
    }
}
