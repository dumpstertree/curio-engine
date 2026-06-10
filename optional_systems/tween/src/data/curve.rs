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
