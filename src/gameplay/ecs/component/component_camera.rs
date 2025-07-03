pub struct Camera {
    pub fov: f32,
}

impl Camera {
    pub fn default() -> Camera {
        Camera { fov: 60.0 }
    }
}
