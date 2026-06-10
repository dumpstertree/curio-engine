use std::any::Any;

use crate::{
    built_in::tween_transform3d_rotation_builder::TweenTransform3DRotationBuilder,
    data::{curve::TweenCurve, tween_common::TweenCommon, tween_state::TweenState, tween_target::TweenTarget},
};
use curio_core::Quaternion;
use gameplay::built_in::facet::transform::transform3d::Transform3D;

#[derive(Clone)]
pub struct TweenTransform3DRotation {
    pub(crate) state: TweenState,
    pub(crate) p0: Quaternion,
    pub(crate) p1: Quaternion,
    pub(crate) p_t: Quaternion,
}
impl TweenTransform3DRotation {
    pub fn builder() -> TweenTransform3DRotationBuilder {
        TweenTransform3DRotationBuilder {
            p0: Quaternion::identity(),
            p1: Quaternion::identity(),
            delay: 0.0,
            duration: 1.0,
            complete: None,
            curve: TweenCurve::Linear,
        }
    }
}
impl TweenCommon for TweenTransform3DRotation {
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
        TweenTarget::new::<Transform3D>("rotation")
    }
}
