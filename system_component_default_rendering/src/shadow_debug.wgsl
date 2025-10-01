// shadow_debug.wgsl
const SHADOW_SIZE: f32 = 2048.0;

@group(0) @binding(0) var shadow_map: texture_depth_2d;
@group(0) @binding(1) var shadow_sampler: sampler_comparison;

struct VSOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@location(0) pos: vec2<f32>, @location(1) uv: vec2<f32>) -> VSOut {
    var out: VSOut;
    out.pos = vec4<f32>(pos, 0.0, 1.0);
    out.uv = uv;
    return out;
}

@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
    // compute integer coords safely in range [0, SHADOW_SIZE-1]
    let tex_x = i32(clamp(in.uv.x * (SHADOW_SIZE - 1.0), 0.0, SHADOW_SIZE - 1.0));
    let tex_y = i32(clamp(in.uv.y * (SHADOW_SIZE - 1.0), 0.0, SHADOW_SIZE - 1.0));
    let depth_val = textureLoad(shadow_map, vec2<i32>(tex_x, tex_y), 0);
    // depth_val is 0..1 (0 = near, 1 = far depending on projection). visualize directly
    return vec4<f32>(depth_val, depth_val, depth_val, 1.0);
}
