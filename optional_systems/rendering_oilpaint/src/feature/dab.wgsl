// one brush dab = an oriented, anisotropic quad welded to the world.
// phase 1 proof: pin/dir/size are honoured; flat debug colours only.

struct Scene {
    view_proj: mat4x4<f32>,
    view_pos: vec4<f32>,
    cam_right: vec4<f32>, // world-space camera basis (view plane)
    cam_up: vec4<f32>,
}
@group(0) @binding(0)
var<uniform> scene: Scene;

// per-quad corner, [-0.5, 0.5]; x runs ALONG the stroke, y ACROSS it
struct QuadCorner {
    @location(0) corner: vec2<f32>,
}

// per-instance dab — layout mirrors the CPU-side Dab struct exactly
struct DabIn {
    @location(1) pin: vec3<f32>,   // object-LOCAL space (phase 4: model mult)
    @location(2) dir: vec3<f32>,   // stroke run direction
    @location(3) size: vec2<f32>,  // across, along — metres
    @location(4) seed: vec4<f32>,  // x: palette/hue, yzw: per-dab randoms
}

struct VSOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,   // u along stroke, v across
    @location(1) tint: vec3<f32>,
}

@vertex
fn vs_main(quad: QuadCorner, inst: DabIn) -> VSOut {
    let right = scene.cam_right.xyz;
    let up = scene.cam_up.xyz;

    // stroke direction projected onto the view plane — screen angle of the
    // brush regardless of camera roll/orbit
    var d2 = vec2<f32>(dot(inst.dir, right), dot(inst.dir, up));
    if length(d2) < 0.0001 {
        d2 = vec2<f32>(1.0, 0.0); // dir points at camera: fallback run
    }
    let run = normalize(d2);

    // world-space basis of the quad, in the view plane through pin
    let along = (right * run.x + up * run.y) * inst.size.y;
    let across = (right * -run.y + up * run.x) * inst.size.x;

    let world = inst.pin + along * quad.corner.x + across * quad.corner.y;

    var out: VSOut;
    out.clip = scene.view_proj * vec4<f32>(world, 1.0);
    out.uv = quad.corner + vec2<f32>(0.5);

    // phase 1: flat debug colour, hue from seed.x, slight edge falloff so
    // quad shape + anisotropy read at a glance
    let hue = fract(inst.seed.x * 0.6180339887 + inst.seed.y * 0.12345);
    let r = abs(hue * 6.0 - 3.0) - 1.0;
    let g = 2.0 - abs(hue * 6.0 - 2.0);
    let b = 2.0 - abs(hue * 6.0 - 4.0);
    let edge = 1.0 - 0.45 * abs(quad.corner.y); // darker rim ACROSS the stroke
    out.tint = clamp(vec3<f32>(r, g, b), vec3<f32>(0.15), vec3<f32>(1.0)) * edge;
    return out;
}

@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
    return vec4<f32>(in.tint, 1.0);
}
