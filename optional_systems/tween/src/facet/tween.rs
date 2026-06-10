use crate::data::tween_common::TweenCommon;
use facet::facet;
use gameplay::traits::field_override::FieldOverride;

#[facet]
pub struct Tween {
    pub tweens: Vec<Box<dyn TweenCommon>>,
}

impl FieldOverride for Tween {
    fn apply(&mut self, _field: &str, _val: &str) {}
    fn get_state(&self) -> Vec<curio_core::FieldState> {
        vec![]
    }
}
impl Tween {
    pub fn builder() -> TweenBuilder {
        TweenBuilder { tweens: Vec::new() }
    }
    pub(crate) fn update(&mut self, dt: f32) {
        for i in (0..self.tweens.len()).rev() {
            if self.tweens[i].update(dt) {
                self.tweens.swap_remove(i);
            }
        }
    }
}

pub struct TweenBuilder {
    tweens: Vec<Box<dyn TweenCommon>>,
}

impl TweenBuilder {
    pub fn add_tween<T: TweenCommon + 'static>(mut self, tween: T) -> Self {
        //
        let target = tween.target();

        // Remove existing tweens that target the same facet + field
        self.tweens.retain(|t| t.target() != target);
        self.tweens.push(Box::new(tween));

        // return
        self
    }
    pub fn build(self) -> Tween {
        Tween { tweens: self.tweens, owner: None }
    }
}
