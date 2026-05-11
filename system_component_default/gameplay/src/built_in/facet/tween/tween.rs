use std::any::{Any, TypeId};

use curio_core::{
    Quaternion, Vector2, Vector3,
    built_in::record::sys_record_time::SysRecordTime,
    collections::{event_queue::Nerve, ledger::Ledger},
    network_modes::NetworkModes,
};

use crate::{
    built_in::facet::transform::transform2d::Transform2D,
    context_3d::Context3D,
    form::Form,
    traits::{facet_common::FacetCommon, field_override::FieldOverride, habit::Habit, scope::Scope},
    traits_internal::world_context_common::ContextCommon,
};

//
// ──────────────────────────────────────────────────────────────────────────
// Tween Curves
// ──────────────────────────────────────────────────────────────────────────
//

#[derive(Clone, Copy)]
pub enum TweenCurve {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Bounce,
}

impl TweenCurve {
    pub fn sample(self, t: f32) -> f32 {
        match self {
            TweenCurve::Linear => t,
            TweenCurve::EaseIn => t * t,
            TweenCurve::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
            TweenCurve::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }
            TweenCurve::Bounce => {
                let n1 = 7.5625;
                let d1 = 2.75;

                if t < 1.0 / d1 {
                    n1 * t * t
                } else if t < 2.0 / d1 {
                    let t = t - 1.5 / d1;
                    n1 * t * t + 0.75
                } else if t < 2.5 / d1 {
                    let t = t - 2.25 / d1;
                    n1 * t * t + 0.9375
                } else {
                    let t = t - 2.625 / d1;
                    n1 * t * t + 0.984375
                }
            }
        }
    }
}

//
// ──────────────────────────────────────────────────────────────────────────
// Tween Target (dynamic uniqueness key)
// ──────────────────────────────────────────────────────────────────────────
//

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

//
// ──────────────────────────────────────────────────────────────────────────
// Tween Trait (dyn-safe)
// ──────────────────────────────────────────────────────────────────────────
//

pub trait TweenCommon: Any + Send + Sync {
    fn as_any(&self) -> &dyn Any;

    fn update(&mut self, dt: f32) -> bool;
    fn target(&self) -> TweenTarget;

    fn set_duration(&mut self, dur: f32);
    fn set_delay(&mut self, delay: f32);
    fn set_curve(&mut self, curve: TweenCurve);
    fn set_paused(&mut self, paused: bool);
}

//
// ──────────────────────────────────────────────────────────────────────────
// Tween Component
// ──────────────────────────────────────────────────────────────────────────
//

pub struct Tween {
    pub tweens: Vec<Box<dyn TweenCommon>>,
    owner: Option<Form>,
}

unsafe impl Send for Tween {}
unsafe impl Sync for Tween {}

impl Default for Tween {
    fn default() -> Self {
        Self { tweens: Vec::new(), owner: None }
    }
}

impl FieldOverride for Tween {
    fn apply(&mut self, _field: &str, _val: &str) {}
}

impl Tween {
    pub fn add_tween<T: TweenCommon + 'static>(&mut self, tween: T) {
        let target = tween.target();

        // Remove existing tweens that target the same facet + field
        self.tweens.retain(|t| t.target() != target);
        self.tweens.push(Box::new(tween));
    }

    pub fn update(&mut self, dt: f32) {
        for i in (0..self.tweens.len()).rev() {
            if self.tweens[i].update(dt) {
                self.tweens.swap_remove(i);
            }
        }
    }
}

impl FacetCommon for Tween {
    fn set_ownership(&mut self, owner: Form) {
        self.owner = Some(owner);
    }

    fn form(&self) -> Form {
        self.owner.clone().unwrap()
    }
}

//
// ──────────────────────────────────────────────────────────────────────────
// Base Tween State
// ──────────────────────────────────────────────────────────────────────────
//

struct TweenState {
    duration: f32,
    elapsed: f32,
    delay: f32,
    curve: TweenCurve,
    paused: bool,
    complete: Option<Box<dyn FnOnce() + Send + Sync>>,
}

impl TweenState {
    fn new() -> Self {
        Self {
            duration: 1.0,
            elapsed: 0.0,
            delay: 0.0,
            curve: TweenCurve::Linear,
            paused: false,
            complete: None,
        }
    }

    fn advance(&mut self, dt: f32) -> Option<f32> {
        if self.paused {
            return None;
        }

        if self.delay > 0.0 {
            self.delay -= dt;
            return None;
        }

        self.elapsed += dt;
        let t = (self.elapsed / self.duration).clamp(0.0, 1.0);
        Some(self.curve.sample(t))
    }

    fn finish(&mut self) {
        println!("finished");
        if let Some(cb) = self.complete.take() {
            cb();
        }
    }
    fn done(&self) -> bool {
        self.elapsed >= self.duration
    }
}

//
// ──────────────────────────────────────────────────────────────────────────
// Transform2D Tweens
// ──────────────────────────────────────────────────────────────────────────
//

pub struct TweenTransform2DPosition {
    state: TweenState,
    start: Vector2,
    end: Vector2,
    pub value: Vector2,
}
impl TweenTransform2DPosition {
    pub fn new(start: Vector2, end: Vector2) -> Self {
        Self { state: TweenState::new(), start, end, value: start }
    }
    pub fn duration(mut self, d: f32) -> Self {
        self.state.duration = d;
        self
    }
    pub fn delay(mut self, d: f32) -> Self {
        self.state.delay = d;
        self
    }
    pub fn curve(mut self, c: TweenCurve) -> Self {
        self.state.curve = c;
        self
    }
    pub fn on_complete(mut self, on_complete: Box<dyn FnOnce() + Send + Sync>) -> Self {
        self.state.complete = Some(on_complete);
        self
    }
}
impl TweenCommon for TweenTransform2DPosition {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn update(&mut self, dt: f32) -> bool {
        if let Some(t) = self.state.advance(dt) {
            self.value = Vector2::lerp(self.start, self.end, t);
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

    fn set_duration(&mut self, d: f32) {
        self.state.duration = d;
    }

    fn set_delay(&mut self, d: f32) {
        self.state.delay = d;
    }

    fn set_curve(&mut self, c: TweenCurve) {
        self.state.curve = c;
    }

    fn set_paused(&mut self, p: bool) {
        self.state.paused = p;
    }
}

pub struct TweenTransform2DRotation {
    state: TweenState,
    start: Quaternion,
    end: Quaternion,
    pub value: Quaternion,
}
impl TweenTransform2DRotation {
    pub fn new(start: Quaternion, end: Quaternion) -> Self {
        Self { state: TweenState::new(), start, end, value: start }
    }
    pub fn duration(mut self, d: f32) -> Self {
        self.state.duration = d;
        self
    }

    pub fn delay(mut self, d: f32) -> Self {
        self.state.delay = d;
        self
    }

    pub fn curve(mut self, c: TweenCurve) -> Self {
        self.state.curve = c;
        self
    }
}
impl TweenCommon for TweenTransform2DRotation {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn update(&mut self, dt: f32) -> bool {
        if let Some(t) = self.state.advance(dt) {
            self.value = Quaternion::slerp(self.start, self.end, t);
        }
        self.state.done()
    }

    fn target(&self) -> TweenTarget {
        TweenTarget::new::<Transform2D>("rotation")
    }

    fn set_duration(&mut self, d: f32) {
        self.state.duration = d;
    }

    fn set_delay(&mut self, d: f32) {
        self.state.delay = d;
    }

    fn set_curve(&mut self, c: TweenCurve) {
        self.state.curve = c;
    }

    fn set_paused(&mut self, p: bool) {
        self.state.paused = p;
    }
}

pub struct TweenTransform2DScale {
    state: TweenState,
    start: Vector3,
    end: Vector3,
    pub value: Vector3,
}

impl TweenTransform2DScale {
    pub fn new(start: Vector3, end: Vector3) -> Self {
        Self { state: TweenState::new(), start, end, value: start }
    }
}

impl TweenCommon for TweenTransform2DScale {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn update(&mut self, dt: f32) -> bool {
        if let Some(t) = self.state.advance(dt) {
            self.value = Vector3::lerp(self.start, self.end, t);
        }
        self.state.done()
    }

    fn target(&self) -> TweenTarget {
        TweenTarget::new::<Transform2D>("scale")
    }

    fn set_duration(&mut self, d: f32) {
        self.state.duration = d;
    }

    fn set_delay(&mut self, d: f32) {
        self.state.delay = d;
    }

    fn set_curve(&mut self, c: TweenCurve) {
        self.state.curve = c;
    }

    fn set_paused(&mut self, p: bool) {
        self.state.paused = p;
    }
}

//
// ──────────────────────────────────────────────────────────────────────────
// System / Habit
// ──────────────────────────────────────────────────────────────────────────
//

#[derive(Default)]
pub struct Instance;

impl Scope for Instance {
    fn is_enabled(&mut self, _ledger: &mut Ledger) -> bool {
        true
    }

    fn run_on_instance(&mut self, _ledger: &mut Ledger) -> Vec<NetworkModes> {
        NetworkModes::all()
    }
}

impl Habit for Instance {
    fn tick(&mut self, ledger: &mut Ledger, context: &mut Context3D, _: &mut Nerve) {
        let dt = ledger.time().scaled_delta_time;

        context.edit::<&mut Tween>(|q| {
            for (_, t) in q {
                t.update(dt);
            }
        });

        context.edit::<(&Tween, &mut Transform2D)>(|q| {
            for (_, (tween, transform)) in q {
                for t in &tween.tweens {
                    if let Some(p) = t.as_any().downcast_ref::<TweenTransform2DPosition>() {
                        transform.position = p.value;
                    } else if let Some(r) = t.as_any().downcast_ref::<TweenTransform2DRotation>() {
                        transform.rotation = r.value;
                    } else if let Some(s) = t.as_any().downcast_ref::<TweenTransform2DScale>() {
                        transform.scale = s.value;
                    }
                }
            }
        });
    }
}
