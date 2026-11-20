// one brush dab = an oriented, anisotropic quad welded to the world.
// phase 2 paint look: hashed edge nibbles, LIGHT/HALF/SHADE tones,
// bristle streaks running ALONG the stroke, a sun term from the stroke's
// flow against light, and pixel-size convergence so distant dabs shrink
// to nothing instead of shimmering. there is no post filter — patch's own
// renderer puts the whole look on the dabs, and it shows up here too.

struct Scene {
    view_proj: mat4x4<f32>,
    view_pos: vec4<f32>,
    cam_right: vec4<f32>, // world-space camera basis (view plane)
    cam_up: vec4<f32>,
    sun: vec4<f32>, // normalized sun direction
    misc: vec4<f32>, // x: clock (ticks), y: screen pixel height
}
@group(0) @binding(0)
var<uniform> scene: Scene;

// per-quad corner, [-0.5, 0.5]; x runs ALONG the stroke, y ACROSS it
struct QuadCorner {
    @location(0) corner: vec2<f32>,
}

// per-instance dab — layout mirrors the CPU-side Dab struct exactly
struct DabIn {
    @location(1) pin: vec3<f32>, // object-LOCAL space
    @location(2) dir: vec3<f32>, // stroke run direction
    @location(3) size: vec2<f32>, // across, along — metres
    @location(4) seed: vec4<f32>, // x hue/palette, yzw per-dab randoms
}

struct VSOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>, // u along stroke, v across — [0,1]
    @location(1) tint: vec3<f32>, // raw hue; tone/sun applied in fs
    @location(2) rnd: vec3<f32>, // per-dab randoms for the hashes
    @location(3) flat meta: vec4<f32>, // tone, sun_k, along_px, across_px
}

// cheap 2D→1D hash; inputs already carry per-dab randoms so neighbouring
// dabs never share nibbles (the "mix the slot into the cell hash" lesson
// from the painterly branches, applied to the instance seed instead)
fn hash11(p: vec2<f32>) -> f32 {
    var q = fract(p * vec2<f32>(123.34, 457.34));
    let d = dot(q, q + vec2<f32>(34.345, 34.345));
    q += vec2<f32>(d, d);
    return fract(q.x * q.y);
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
    out.rnd = inst.seed.yzw;

    // hue base — chroma kept, palette slot chosen by seed.x
    let hue = fract(inst.seed.x * 0.6180339887 + inst.seed.y * 0.12345);
    let r = abs(hue * 6.0 - 3.0) - 1.0;
    let g = 2.0 - abs(hue * 6.0 - 2.0);
    let b = 2.0 - abs(hue * 6.0 - 4.0);
    out.tint = clamp(vec3<f32>(r, g, b), vec3<f32>(0.12), vec3<f32>(1.0));

    // LIGHT / HALF / SHADE tone band, hashed per dab
    let band = inst.seed.y;
    var tone = 0.62;
    if band >= 0.67 {
        tone = 1.12;
    } else if band >= 0.34 {
        tone = 0.85;
    }

    // sun term from the stroke's flow against light — dabs running with
    // the light read lit, against it read shadowed
    let dn = inst.dir / max(length(inst.dir), 0.0001);
    let sd = dot(dn, scene.sun.xyz);
    let sun_k = 0.78 + 0.34 * (0.5 + 0.5 * sd);

    // pixel extent of this dab: proj col 1 row 1 is 1/tan(fovy/2), half
    // viewport height turns NDC into pixels, clip.w is the depth
    let proj_scale = abs(scene.view_proj[1][1]);
    let px = 0.5 * proj_scale * scene.misc.y / max(out.clip.w, 0.0001);
    out.meta = vec4<f32>(tone, sun_k, inst.size.y * px, inst.size.x * px);

    return out;
}

@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
    // sub-pixel strokes shrink to nothing — no distant mud
    if in.meta.w < 1.3 {
        discard;
    }

    let e = in.uv - vec2<f32>(0.5); // [-0.5,0.5]; x along, y across

    // nibble amplitude is ~one pixel in uv terms, so far dabs converge
    // back to a clean quad instead of jittering per frame
    let amp_a = min(0.34, 1.6 / max(in.meta.z, 1.0));
    let amp_c = min(0.42, 1.6 / max(in.meta.w, 1.0));

    // edge hash: along-position drives the run-edge nibbles, across the
    // biting ends — seeded with this dab's randoms
    let ha = hash11(vec2<f32>(e.x * 9.0 + 13.0, in.rnd.y * 31.0 + e.y * 4.0));
    let hc = hash11(vec2<f32>(e.y * 7.0 + 7.0, in.rnd.z * 29.0 + e.x * 3.0));

    let ex = 0.5 + (ha - 0.5) * amp_a;
    let ey = 0.5 + (hc - 0.5) * amp_c;
    if abs(e.x) > ex || abs(e.y) > ey {
        discard;
    }

    // inside-ness for the rim: stroke edges sit darker, like thick paint
    let d = max(abs(e.x) / ex, abs(e.y) / ey);
    let rim = 1.0 - 0.38 * smoothstep(0.5, 1.0, d);

    // bristle streaks run ALONG the stroke — pattern varies with v
    let br = sin((in.uv.y - 0.5) * 2.0 * 3.14159 * (3.0 + in.rnd.x * 5.0) + in.rnd.z * 6.283);
    let streak = 1.0 + 0.13 * br;

    // misc.x (the clock) is deliberately unsampled here: the paint look is
    // static, wind (P5) is what will put it to work
    let col = in.tint * in.meta.x * in.meta.y * streak * rim;
    return vec4<f32>(clamp(col, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
