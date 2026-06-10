use crate::{
    built_in::tween_transform2d_rotation_builder::TweenTransform2DRotationBuilder,
    data::{curve::TweenCurve, tween_common::TweenCommon, tween_state::TweenState, tween_target::TweenTarget},
};
use curio_core::Quaternion;
use gameplay::built_in::facet::transform::transform2d::Transform2D;
use std::any::Any;

#[derive(Clone)]
pub struct TweenTransform2DRotation {
    pub(crate) state: TweenState,
    pub(crate) p0: Quaternion,
    pub(crate) p1: Quaternion,
    pub(crate) p_t: Quaternion,
}
impl TweenTransform2DRotation {
    pub fn builder() -> TweenTransform2DRotationBuilder {
        TweenTransform2DRotationBuilder {
            p0: Quaternion::identity(),
            p1: Quaternion::identity(),
            delay: 0.0,
            duration: 1.0,
            complete: None,
            curve: TweenCurve::Linear,
        }
    }
}
impl TweenCommon for TweenTransform2DRotation {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn update(&mut self, dt: f32) -> bool {
        if let Some(t) = self.state.advance(dt) {
            self.p_t = Quaternion::slerp(self.p0, self.p1, t);
        }
        self.state.done()
    }

    fn target(&self) -> TweenTarget {
        TweenTarget::new::<Transform2D>("rotation")
    }
}
