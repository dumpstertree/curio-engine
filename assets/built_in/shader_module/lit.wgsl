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
    @location(1) normal: vec3<f32>,
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
    @location(1) normal: vec3<f32>,
    @location(2) world_pos: vec3<f32>,
    @location(3) shadow_pos: vec4<f32>,
}

// ----------------- Shadow Uniform -----------------
struct ShadowCamera {
    light_view_proj: mat4x4<f32>,
}
struct ColorUniform {
    color: vec4<f32>,
}
@group(3) @binding(0)
var<uniform> shadow_camera: ShadowCamera;

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
    let normal_matrix = mat3x3<f32>(
        model_matrix[0].xyz,
        model_matrix[1].xyz,
        model_matrix[2].xyz
    );

    var out: VSOut;
    out.clip_position = camera.view_proj * world_position;
    out.tex_coords = model.uv0;
    out.world_pos = world_position.xyz;
    out.normal = normalize(normal_matrix * model.normal);

    // transform vertex into light clip space
    out.shadow_pos = shadow_camera.light_view_proj * world_position;

    return out;
}

// ----------------- Colors -----------------
@group(0) @binding(0)
var<uniform> tint_data:ColorUniform;

// ----------------- Textures -----------------
@group(0) @binding(1)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(2)
var s_diffuse: sampler;

// ----------------- Shadow Map -----------------
@group(3) @binding(1)
var shadow_map: texture_depth_2d;
@group(3) @binding(2)
var shadow_sampler: sampler_comparison;

// ----------------- Lights -----------------
struct GpuLight {
    position: vec4<f32>,
    color_intensity: vec4<f32>,
    direction_radius: vec4<f32>,
    _padding: vec4<f32>,
};

struct LightBuffer {
    @align(16)
    count: u32,
    @align(16)
    lights: array<GpuLight, 16>,
};

@group(2) @binding(0)
var<storage, read> light_buffer: LightBuffer;

const AMBIENT_LIGHT: f32 = 0.1;
const SHADOW_STRENGTH: f32 = 0.75;

// ----------------- Shadow Calculation -----------------
fn compute_shadow_factor(shadow_pos: vec4<f32>) -> f32 {
    let proj_coords = shadow_pos.xyz / shadow_pos.w;
    let uv = proj_coords.xy * 0.5 + vec2<f32>(0.5, 0.5);
    let depth = proj_coords.z;

    if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0) {
        return 1.0;
    }

    return textureSampleCompare(shadow_map, shadow_sampler, uv, depth - 0.001);
}

// ----------------- Lighting -----------------
fn compute_forward_lighting(
    N: vec3<f32>,
    V: vec3<f32>,
    world_pos: vec3<f32>,
    shadow_factor: f32
) -> vec3<f32> {
    var total_light = vec3<f32>(0.0);
    let count = light_buffer.count;

    // blend shadow_factor based on strength
    let final_shadow = mix(1.0, shadow_factor, SHADOW_STRENGTH);

    for (var i = 0u; i < count; i = i + 1u) {
        let light = light_buffer.lights[i];
        let light_type = u32(light.position.w);

        if (light_type == 0u) {
            let L = normalize(-light.direction_radius.xyz);
            let diff = max(dot(N, L), 0.0);
            let H = normalize(L + V);
            let spec = pow(max(dot(N, H), 0.0), 32.0);

            total_light += light.color_intensity.rgb * light.color_intensity.a 
                         * (diff + spec * 0.3) * final_shadow;
        } else if (light_type == 1u) {
            let L_vec = light.position.xyz - world_pos;
            let dist = length(L_vec);
            if (dist < light.direction_radius.w) {
                let L = normalize(L_vec);
                let attenuation = 1.0 / (1.0 + 0.09 * dist + 0.032 * dist * dist);
                let diff = max(dot(N, L), 0.0);
                let H = normalize(L + V);
                let spec = pow(max(dot(N, H), 0.0), 32.0);

                total_light += light.color_intensity.rgb * light.color_intensity.a 
                             * attenuation * (diff + spec * 0.3) * final_shadow;
            }
        }
    }

    return vec3<f32>(AMBIENT_LIGHT) + total_light;
}

// ----------------- Fragment Shader -----------------
@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
    let tex_sample = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    if (tex_sample.a < 0.5) {
        discard;
    }

    let albedo = tex_sample.rgb;
    let N = normalize(in.normal);
    let V = normalize(camera.view_pos.xyz - in.world_pos);

    let shadow_factor = compute_shadow_factor(in.shadow_pos) * SHADOW_STRENGTH;
    let light_color = compute_forward_lighting(N, V, in.world_pos, shadow_factor);

    let tinted_color = albedo * tint_data.color.rgb;
    return vec4<f32>(tinted_color * light_color, tex_sample.a * tint_data.color.a);
}
