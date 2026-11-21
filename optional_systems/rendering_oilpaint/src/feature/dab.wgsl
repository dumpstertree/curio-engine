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
    wind: vec4<f32>, // xyz: breeze direction, w: strength
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
    @location(9) flex: f32, // wind bendiness: 0 rigid, 1 grass
}

// phase 4 rigid pin: the form's matrix as a second per-instance attribute
// (four columns, locations 5-8 — same convention as the engine's lit.wgsl).
// the paint is sampled ONCE in object space; moving the form moves its
// strokes for free — no re-sampling, no CPU bake
struct ModelIn {
    @location(5) col0: vec4<f32>,
    @location(6) col1: vec4<f32>,
    @location(7) col2: vec4<f32>,
    @location(8) col3: vec4<f32>,
}

struct VSOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>, // u along stroke, v across — [0,1]
    @location(1) tint: vec3<f32>, // raw hue; tone/sun applied in fs
    @location(2) rnd: vec3<f32>, // per-dab randoms for the hashes
    @location(3) info: vec4<f32>, // tone, sun_k, along_px, across_px — constant per
    // dab so plain interpolation reproduces it exactly ("meta" was a WGSL keyword)
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
fn vs_main(quad: QuadCorner, inst: DabIn, model_in: ModelIn) -> VSOut {
    let right = scene.cam_right.xyz;
    let up = scene.cam_up.xyz;

    // rigid pin: object-local → world through the form's matrix (point
    // with w=1, direction with w=0); the stroke size picks up the form's
    // scale as the mean basis-column length
    let model = mat4x4<f32>(model_in.col0, model_in.col1, model_in.col2, model_in.col3);
    let pin_world = (model * vec4<f32>(inst.pin, 1.0)).xyz;
    let dir = (model * vec4<f32>(inst.dir, 0.0)).xyz;
    let scale = (length(model[0]) + length(model[1]) + length(model[2])) / 3.0;
    let size = inst.size * scale;

    // wind (phase 5): a gust traveling along the breeze — pins bend by
    // their height above the form's own origin (model column 3 is the
    // translation, so a reed's tip swings and its base stays planted),
    // seeded per dab so neighbours wave out of step; inst.flex is the
    // bendiness: walls and floors keep 0, grass 1
    let wind_dir = scene.wind.xyz / max(length(scene.wind.xyz), 0.0001);
    let gust = sin(dot(pin_world.xz, wind_dir.xz) * 0.6 + scene.misc.x * 0.12 + inst.seed.y * 2.0);
    let lever = pin_world.y - model[3][1];
    let bend = min(max(lever, 0.0), 1.6) * inst.flex;
    let pin = pin_world + wind_dir * (scene.wind.w * gust * bend);

    // stroke direction projected onto the view plane — screen angle of the
    // brush regardless of camera roll/orbit
    var d2 = vec2<f32>(dot(dir, right), dot(dir, up));
    if length(d2) < 0.0001 {
        d2 = vec2<f32>(1.0, 0.0); // dir points at camera: fallback run
    }
    let run = normalize(d2);

    // world-space basis of the quad, in the view plane through pin
    let along = (right * run.x + up * run.y) * size.y;
    let across = (right * -run.y + up * run.x) * size.x;

    let world = pin + along * quad.corner.x + across * quad.corner.y;

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
    let dn = dir / max(length(dir), 0.0001);
    let sd = dot(dn, scene.sun.xyz);
    let sun_k = 0.78 + 0.34 * (0.5 + 0.5 * sd);

    // pixel extent of this dab: proj col 1 row 1 is 1/tan(fovy/2), half
    // viewport height turns NDC into pixels, clip.w is the depth
    let proj_scale = abs(scene.view_proj[1][1]);
    let px = 0.5 * proj_scale * scene.misc.y / max(out.clip.w, 0.0001);
    out.info = vec4<f32>(tone, sun_k, size.y * px, size.x * px);

    return out;
}

@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
    // sub-pixel strokes shrink to nothing — no distant mud
    if in.info.w < 1.3 {
        discard;
    }

    let e = in.uv - vec2<f32>(0.5); // [-0.5,0.5]; x along, y across

    // nibble amplitude is ~one pixel in uv terms, so far dabs converge
    // back to a clean quad instead of jittering per frame
    let amp_a = min(0.34, 1.6 / max(in.info.z, 1.0));
    let amp_c = min(0.42, 1.6 / max(in.info.w, 1.0));

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

    // the clock drives the wind gust in the vs; the fs paint look itself
    // stays static per dab
    // the ramp outputs full-chroma hues; real palette paint is mostly
    // mixed toward its own luminance — pull every dab halfway to gray
    // tone x sun x streak x rim routinely pushes past 1.0, and the naive
    // clamp then blew every hue toward cream (white = maxed channels). cap
    // the MAX channel at 0.95 instead: hue survives, brightness doesn't
    // blow. then mix halfway toward luminance for the painterly mute
    let raw = in.tint * in.info.x * in.info.y * streak * rim;
    let mx = max(raw.x, max(raw.y, raw.z));
    var base = raw;
    if mx > 1.0 {
        base = raw * (0.95 / mx);
    }
    let lum = dot(base, vec3<f32>(0.3, 0.6, 0.1));
    let col = mix(base, vec3<f32>(lum), 0.45);
    return vec4<f32>(clamp(col, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
