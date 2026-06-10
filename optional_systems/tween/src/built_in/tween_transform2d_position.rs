use crate::{
    built_in::tween_transform2d_position_builder::TweenTransform2DPositionBuilder,
    data::{curve::TweenCurve, tween_common::TweenCommon, tween_state::TweenState, tween_target::TweenTarget},
};
use curio_core::Vector2;
use gameplay::built_in::facet::transform::transform2d::Transform2D;
use std::any::Any;

#[derive(Clone)]
pub struct TweenTransform2DPosition {
    pub(crate) state: TweenState,
    pub(crate) p0: Vector2,
    pub(crate) p1: Vector2,
    pub(crate) p_t: Vector2,
}
impl TweenTransform2DPosition {
    pub fn builder() -> TweenTransform2DPositionBuilder {
        TweenTransform2DPositionBuilder {
            p0: Vector2::zero(),
            p1: Vector2::zero(),
            delay: 0.0,
            duration: 1.0,
            complete: None,
            curve: TweenCurve::Linear,
        }
    }
}
impl TweenCommon for TweenTransform2DPosition {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn update(&mut self, dt: f32) -> bool {
        if let Some(t) = self.state.advance(dt) {
            self.p_t = Vector2::lerp(self.p0, self.p1, t);
        }

        if self.state.done() {
            self.state.finish();
            true
        } else {
            false
        }
    }

    fn target(&self) -> TweenTarget {
        TweenTarget::new::<Transform2D>("position")
    }
}
