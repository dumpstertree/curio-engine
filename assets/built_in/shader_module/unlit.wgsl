// ----------------- Camera -----------------
struct Camera {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
}

@group(1) @binding(0)
var<uniform> camera: Camera;

// ----------------- Vertex Input -----------------
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>, // kept for layout compatibility
    @location(2) color: vec3<f32>,
    @location(3) uv0: vec2<f32>,
    @location(4) uv1: vec2<f32>,
}

struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
}

// ----------------- Vertex Output -----------------
struct VSOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

// ----------------- Color Uniform -----------------
struct ColorUniform {
    color: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> tint_data: ColorUniform;

// ----------------- Textures -----------------
@group(0) @binding(1)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(2)
var s_diffuse: sampler;

// ----------------- Vertex Shader -----------------
@vertex
fn vs_main(model: VertexInput, instance: InstanceInput) -> VSOut {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    let world_position = model_matrix * vec4<f32>(model.position, 1.0);

    var out: VSOut;
    out.clip_position = camera.view_proj * world_position;
    out.tex_coords = model.uv0;

    return out;
}

// ----------------- Fragment Shader -----------------
@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
    let tex_sample = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    if (tex_sample.a < 0.5) {
        discard;
    }

    // Simple unlit shading: texture * tint
    let tinted_color = tex_sample * tint_data.color;
    return tinted_color;
}
