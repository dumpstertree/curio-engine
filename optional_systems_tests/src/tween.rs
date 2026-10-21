#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicI32, Ordering};
    use std::sync::Arc;

    use tween::built_in::tween_transform3d_position::TweenTransform3DPosition;
    use tween::built_in::tween_transform3d_rotation::TweenTransform3DRotation;
    use tween::data::curve::TweenCurve;
    use tween::data::tween_common::TweenCommon;
    use tween::data::tween_state::TweenState;
    use tween::data::tween_target::TweenTarget;
    use tween::facet::tween::Tween;
    use gameplay::traits::field_override::FieldOverride;

    const EPSILON: f32 = 0.0001;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < EPSILON
    }

    struct TargetA;
    struct TargetB;

    // ---------- curves ----------

    #[test]
    fn all_curves_have_unit_endpoints() {
        let curves = [TweenCurve::Linear, TweenCurve::EaseIn, TweenCurve::EaseOut, TweenCurve::EaseInOut, TweenCurve::Bounce];
        for curve in curves {
            assert!(approx_eq(curve.sample(0.0), 0.0), "{:?} at 0", curve);
            assert!(approx_eq(curve.sample(1.0), 1.0), "{:?} at 1", curve);
        }
    }

    #[test]
    fn curve_midpoints() {
        assert!(approx_eq(TweenCurve::Linear.sample(0.5), 0.5));
        assert!(approx_eq(TweenCurve::EaseIn.sample(0.5), 0.25));
        assert!(approx_eq(TweenCurve::EaseOut.sample(0.5), 0.75));
        assert!(approx_eq(TweenCurve::EaseInOut.sample(0.5), 0.5));
        assert!(approx_eq(TweenCurve::Bounce.sample(0.5), 0.765625));
    }

    #[test]
    fn curves_are_monotonic_on_samples() {
        let curves = [TweenCurve::Linear, TweenCurve::EaseIn, TweenCurve::EaseOut, TweenCurve::EaseInOut];
        for curve in curves {
            let mut last = -1.0;
            for i in 0..=10 {
                let t = i as f32 / 10.0;
                let value = curve.sample(t);
                assert!(value >= last - EPSILON, "{:?} not monotonic at {}", curve, t);
                assert!(value >= -EPSILON && value <= 1.0 + EPSILON, "{:?} out of range at {}", curve, t);
                last = value;
            }
        }
    }

    // ---------- tween state ----------

    #[test]
    fn state_defaults() {
        let state = TweenState::default();
        assert_eq!(state.duration, 1.0);
        assert_eq!(state.elapsed, 0.0);
        assert_eq!(state.delay, 0.0);
        assert!(!state.paused);
        assert!(state.complete.is_none());
        assert!(!state.done());
    }

    #[test]
    fn paused_state_does_not_advance() {
        let mut state = TweenState::new(1.0, 0.0, TweenCurve::Linear, None);
        state.paused = true;
        assert_eq!(state.advance(0.5), None);
        assert_eq!(state.elapsed, 0.0);
    }

    #[test]
    fn delay_is_consumed_before_progress() {
        let mut state = TweenState::new(1.0, 0.5, TweenCurve::Linear, None);
        assert_eq!(state.advance(0.25), None);
        assert_eq!(state.elapsed, 0.0);
        assert_eq!(state.delay, 0.25);
        // the dt spent finishing the delay does not count toward the tween
        assert_eq!(state.advance(0.25), None);
        assert_eq!(state.delay, 0.0);
        let t = state.advance(0.5);
        assert!(t.is_some());
        assert!(approx_eq(t.unwrap(), 0.5));
    }

    #[test]
    fn progress_clamps_to_one() {
        let mut state = TweenState::new(1.0, 0.0, TweenCurve::Linear, None);
        state.advance(2.0);
        assert!(approx_eq(state.advance(0.0).unwrap(), 1.0));
        assert!(state.done());
    }

    #[test]
    fn finish_fires_completion_once() {
        let fired = Arc::new(AtomicI32::new(0));
        let counter = fired.clone();
        let mut state = TweenState::new(1.0, 0.0, TweenCurve::Linear, Some(Box::new(move || {
            counter.fetch_add(1, Ordering::SeqCst);
        })));
        state.finish();
        state.finish();
        assert_eq!(fired.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn clone_drops_completion_callback() {
        let mut state = TweenState::new(1.0, 0.0, TweenCurve::EaseIn, Some(Box::new(|| {})));
        state.advance(0.3);
        let clone = state.clone();
        assert!(clone.complete.is_none());
        assert_eq!(clone.duration, state.duration);
        assert_eq!(clone.elapsed, state.elapsed);
        assert_eq!(clone.paused, state.paused);
    }

    // ---------- tween target ----------

    #[test]
    fn targets_compare_by_type_and_field() {
        assert_eq!(TweenTarget::new::<TargetA>("position"), TweenTarget::new::<TargetA>("position"));
        assert_ne!(TweenTarget::new::<TargetA>("position"), TweenTarget::new::<TargetA>("rotation"));
        assert_ne!(TweenTarget::new::<TargetA>("position"), TweenTarget::new::<TargetB>("position"));
    }

    // ---------- built-in tweens ----------

    #[test]
    fn position_tween_completes_and_fires_callback() {
        let fired = Arc::new(AtomicI32::new(0));
        let counter = fired.clone();
        let mut tween = TweenTransform3DPosition::builder()
            .duration(1.0)
            .on_complete(Some(Box::new(move || {
                counter.fetch_add(1, Ordering::SeqCst);
            })))
            .build();
        assert!(!tween.update(0.5));
        assert!(tween.update(0.5));
        assert_eq!(fired.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn position_tween_holds_during_delay() {
        let mut tween = TweenTransform3DPosition::builder().delay(0.5).duration(1.0).build();
        assert!(!tween.update(0.25));
        assert!(!tween.update(0.25));
        assert!(!tween.update(0.9));
        assert!(tween.update(0.6));
    }

    #[test]
    fn position_tween_targets_transform_position() {
        let tween = TweenTransform3DPosition::builder().build();
        assert_eq!(tween.target(), TweenTarget::new::<gameplay::built_in::facet::transform::transform3d::Transform3D>("position"));
    }

    // ---------- tween facet ----------

    #[test]
    fn builder_replaces_tween_on_same_target() {
        let tween = Tween::builder()
            .add_tween(TweenTransform3DPosition::builder().build())
            .add_tween(TweenTransform3DPosition::builder().duration(2.0).build())
            .build();
        assert_eq!(tween.tweens.len(), 1);
    }

    #[test]
    fn builder_keeps_tweens_on_different_targets() {
        let tween = Tween::builder()
            .add_tween(TweenTransform3DPosition::builder().build())
            .add_tween(TweenTransform3DRotation::builder().build())
            .build();
        assert_eq!(tween.tweens.len(), 2);
    }

    #[test]
    fn tween_facet_apply_is_inert() {
        let mut tween = Tween::builder().add_tween(TweenTransform3DPosition::builder().build()).build();
        tween.apply("anything", "whatever");
        assert_eq!(tween.tweens.len(), 1);
        assert!(tween.get_state().is_empty());
    }
}
