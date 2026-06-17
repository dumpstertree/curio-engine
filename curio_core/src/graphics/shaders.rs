use egui_wgpu::wgpu::{ShaderModule, ShaderModuleDescriptor, ShaderSource};
use std::sync::{Arc, LazyLock};

use crate::engine::engine_services::services;

static LIT: LazyLock<Arc<ShaderModule>> = LazyLock::new(|| {
    Arc::new(
        services()
            .gpu
            .device()
            .create_shader_module(ShaderModuleDescriptor {
                label: Some("Lit Shader"),
                source: ShaderSource::Wgsl(include_str!("../../../assets/built_in/shader_module/lit.wgsl").into()),
            }),
    )
});

static UNLIT: LazyLock<Arc<ShaderModule>> = LazyLock::new(|| {
    Arc::new(
        services()
            .gpu
            .device()
            .create_shader_module(ShaderModuleDescriptor {
                label: Some("Lit Shader"),
                source: ShaderSource::Wgsl(include_str!("../../../assets/built_in/shader_module/unlit.wgsl").into()),
            }),
    )
});

static VEGETATION: LazyLock<Arc<ShaderModule>> = LazyLock::new(|| panic!());
static PARTICLE: LazyLock<Arc<ShaderModule>> = LazyLock::new(|| panic!());

pub struct Shaders {}
impl Shaders {
    pub fn lit() -> Arc<ShaderModule> {
        LIT.clone()
    }
    pub fn unlit() -> Arc<ShaderModule> {
        UNLIT.clone()
    }
}
