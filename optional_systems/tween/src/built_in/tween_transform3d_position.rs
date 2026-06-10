use std::any::Any;

use curio_core::Vector3;
use gameplay::built_in::facet::transform::transform3d::Transform3D;

use crate::{
    built_in::tween_transform3d_position_builder::TweenTransform3DPositionBuilder,
    data::{curve::TweenCurve, tween_common::TweenCommon, tween_state::TweenState, tween_target::TweenTarget},
};

#[derive(Clone)]
pub struct TweenTransform3DPosition {
    pub(crate) state: TweenState,
    pub(crate) p0: Vector3,
    pub(crate) p1: Vector3,
    pub(crate) p_t: Vector3,
}
impl TweenTransform3DPosition {
    pub fn builder() -> TweenTransform3DPositionBuilder {
        TweenTransform3DPositionBuilder {
            p0: Vector3::zero(),
            p1: Vector3::zero(),
            delay: 0.0,
            duration: 1.0,
            complete: None,
            curve: TweenCurve::Linear,
        }
    }
}
impl TweenCommon for TweenTransform3DPosition {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn update(&mut self, dt: f32) -> bool {
        if let Some(t) = self.state.advance(dt) {
            self.p_t = Vector3::lerp(self.p0, self.p1, t);
        }

        if self.state.done() {
            self.state.finish();
            true
        } else {
            false
        }
    }

    fn target(&self) -> TweenTarget {
        TweenTarget::new::<Transform3D>("position")
    }
}
