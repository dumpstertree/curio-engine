use inline_tweak::tweak;
pub struct Constants {}
impl Constants {
    pub fn paddle_speed_acceleration() -> f32 {
        tweak!(30.0)
    }
    pub fn paddle_speed_decceleration() -> f32 {
        tweak!(30.0)
    }
    pub fn paddle_speed_terminal() -> f32 {
        tweak!(15.0)
    }
}
