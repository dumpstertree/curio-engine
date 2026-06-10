use crate::data::curve::TweenCurve;

impl TweenState {
    pub fn default() -> Self {
        Self {
            duration: 1.0,
            elapsed: 0.0,
            delay: 0.0,
            curve: TweenCurve::Linear,
            paused: false,
            complete: None,
        }
    }
    pub fn new(duration: f32, delay: f32, curve: TweenCurve, complete: Option<Box<dyn FnOnce() + Send + Sync>>) -> Self {
        Self {
            duration: duration,
            elapsed: 0.0,
            delay: delay,
            curve: curve,
            paused: false,
            complete: complete,
        }
    }

    pub fn advance(&mut self, dt: f32) -> Option<f32> {
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

    pub fn finish(&mut self) {
        println!("finished");
        if let Some(cb) = self.complete.take() {
            cb();
        }
    }
    pub fn done(&self) -> bool {
        self.elapsed >= self.duration
    }
}
pub struct TweenState {
    pub duration: f32,
    pub elapsed: f32,
    pub delay: f32,
    pub curve: TweenCurve,
    pub paused: bool,
    pub complete: Option<Box<dyn FnOnce() + Send + Sync>>,
}
impl Clone for TweenState {
    fn clone(&self) -> Self {
        println!("Complete is not yet cloned");
        Self {
            duration: self.duration.clone(),
            elapsed: self.elapsed.clone(),
            delay: self.delay.clone(),
            curve: self.curve.clone(),
            paused: self.paused.clone(),
            complete: None,
        }
    }
}
