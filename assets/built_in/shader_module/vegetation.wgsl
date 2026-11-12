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
    @location(1) world_pos: vec3<f32>,
    @location(2) local_height: f32,
    @location(3) ao: f32,
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

// ----------------- World Variation -----------------
//
// Large soft patches of hue drift across the field.
fn world_variation(p: vec3<f32>) -> f32 {
    let n = sin(p.x * 0.7) + sin(p.z * 0.35 + 1.3) + sin((p.x + p.z) * 0.23);
    return fract(n * 0.5 + 0.5) - 0.5;
}

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
    out.world_pos = world_position.xyz;

    // Tip brightening uses the model-space height
    out.local_height = clamp(model.position.y, 0.0, 1.0);

    // Baked ambient occlusion stored in the vertex color
    out.ao = max(model.color.b, 0.15);

    return out;
}

// ----------------- Fragment Shader -----------------
@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
    let tex_sample = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    if (tex_sample.a < 0.5) {
        discard;
    }

    var color = tex_sample.rgb * tint_data.color.rgb;

    // Soft patches of color variation
    color *= mix(0.88, 1.06, world_variation(in.world_pos) + 0.5);

    // Brighter tips
    color *= mix(0.92, 1.10, in.local_height);

    // Ambient occlusion from baked vertex data
    color *= mix(0.75, 1.0, in.ao);

    // Tiny hue shift to make fields feel alive
    let hue_noise = world_variation(in.world_pos.yzx);
    color.r *= 1.0 + hue_noise * 0.015;
    color.g *= 1.0 + hue_noise * 0.010;
    color.b *= 1.0 - hue_noise * 0.010;

    // Soft contrast compression for a painterly look
    color = pow(color, vec3<f32>(0.92));
    color = clamp(color, vec3<f32>(0.0), vec3<f32>(1.0));

    return vec4<f32>(color, tex_sample.a * tint_data.color.a);
}
