// --- Light uniform layout ---
// matches GpuLight (4x vec4 per light), header: vec4<u32> with count in x
struct GpuLight {
    position: vec4<f32>;        // xyz = position, w = type (0=dir,1=point)
    color_intensity: vec4<f32>; // rgb = color, a = intensity
    direction_radius: vec4<f32>; // xyz = direction, w = radius
    padding: vec4<f32>;
};

struct LightHeader {
    count: vec4<u32>;
};

@group(2) @binding(0)
var<uniform> lights_header: LightHeader;

@group(2) @binding(1)
var<uniform> lights_array: array<GpuLight>; // NOTE: some WGSL toolchains require separate binding; if not supported, pack differently

// If you can't have two bindings for the same group, instead pack header as first vec4 of the buffer and read via array load.
// In the Rust layout we put header first then lights. To keep WGSL simple, you can read lights_array[0..count] assuming the WGSL runtime supports it.

fn to_vec3(v: vec4<f32>) -> vec3<f32> { return v.xyz; }

fn compute_light(
    normal: vec3<f32>,
    view_dir: vec3<f32>,
    world_pos: vec3<f32>,
) -> vec3<f32> {
    var col: vec3<f32> = vec3<f32>(0.0);

    let count: u32 = lights_header.count.x;
    for (var i: u32 = 0u; i < count; i = i + 1u) {
        let l = lights_array[i];

        let ltype = i32(l.position.w + 0.5); // 0 = dir, 1 = point
        let light_color = l.color_intensity.xyz;
        let intensity = l.color_intensity.w;

        if (ltype == 0) {
            // directional: direction stored in direction_radius.xyz (should be normalized)
            let L = normalize(-l.direction_radius.xyz); // assume stored as direction to light
            let NdotL = max(dot(normal, L), 0.0);
            let diff = NdotL;

            // specular (Blinn-Phong)
            let H = normalize(L + view_dir);
            let spec = pow(max(dot(normal, H), 0.0), 32.0);

            col = col + (light_color * intensity) * (diff + 0.5 * spec);
        } else {
            // point light
            let pos = l.position.xyz;
            let toLight = pos - world_pos;
            let dist = length(toLight);
            let L = normalize(toLight);
            let NdotL = max(dot(normal, L), 0.0);

            // attenuate by radius (simple)
            let radius = l.direction_radius.w;
            let att = 1.0 / (1.0 + (dist * dist) / max(0.0001, radius * radius));

            let diff = NdotL;
            let H = normalize(L + view_dir);
            let spec = pow(max(dot(normal, H), 0.0), 32.0);

            col = col + (light_color * intensity) * (diff + 0.5 * spec) * att;
        }
    }
    return col;
}
