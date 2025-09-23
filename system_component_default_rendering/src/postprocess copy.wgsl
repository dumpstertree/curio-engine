// === Bindings ===
// group(0) binding(0): sampler
// group(0) binding(1): texture2D
// group(0) binding(2): uniform vec2<i32> or vec2<f32> resolution

@group(0) @binding(0)
var mySampler: sampler;

@group(0) @binding(1)
var myTexture: texture_2d<f32>;

@group(0) @binding(2)
var<uniform> iResolution: vec2<f32>;

// --- structs for pipeline ---
struct VertexOut {
    @builtin(position) pos : vec4<f32>,
    @location(0) uv : vec2<f32>,
};

// --- full screen quad ---
@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> VertexOut {
    var positions = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>(-1.0,  1.0),
        vec2<f32>(-1.0,  1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>( 1.0,  1.0)
    );

    var uvs = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 1.0)
    );

    var out: VertexOut;
    out.pos = vec4<f32>(positions[vid], 0.0, 1.0);
    out.uv = uvs[vid];
    return out;
}

// === Sobel Edge Detection ===
fn Sobel(uv: vec2<f32>) -> f32 {
    var gx: f32 = 0.0;
    var gy: f32 = 0.0;
    let texel = 1.0 / iResolution;

    for (var xi: i32 = -1; xi <= 1; xi = xi + 1) {
        for (var yi: i32 = -1; yi <= 1; yi = yi + 1) {
            let offset = vec2<f32>(f32(xi), f32(yi)) * texel;
            let c = textureSample(myTexture, mySampler, uv + offset).rgb;
            let lum = dot(c, vec3<f32>(0.3, 0.59, 0.11));

            var kx: f32 = 0.0;
            var ky: f32 = 0.0;
            if (xi == -1 && yi == -1) { kx = -1.0; ky = -1.0; }
            if (xi ==  0 && yi == -1) { kx = -2.0; ky =  0.0; }
            if (xi ==  1 && yi == -1) { kx = -1.0; ky =  1.0; }
            if (xi == -1 && yi ==  0) { kx =  0.0; ky = -2.0; }
            if (xi ==  0 && yi ==  0) { kx =  0.0; ky =  0.0; }
            if (xi ==  1 && yi ==  0) { kx =  0.0; ky =  2.0; }
            if (xi == -1 && yi ==  1) { kx =  1.0; ky = -1.0; }
            if (xi ==  0 && yi ==  1) { kx =  2.0; ky =  0.0; }
            if (xi ==  1 && yi ==  1) { kx =  1.0; ky =  1.0; }

            gx = gx + lum * kx;
            gy = gy + lum * ky;
        }
    }
    return length(vec2<f32>(gx, gy));
}

// === Kuwahara Filter ===
const RADIUS: i32 = 4;

fn Kuwahara(uv: vec2<f32>) -> vec3<f32> {
    var mean: array<vec3<f32>, 4>;
    var varSum: array<f32, 4>;
    var count: array<f32, 4>;
    for (var i: i32 = 0; i < 4; i = i + 1) {
        mean[i] = vec3<f32>(0.0);
        varSum[i] = 0.0;
        count[i] = 0.0;
    }

    let texel = 1.0 / iResolution;

    for (var xi: i32 = -RADIUS; xi <= RADIUS; xi = xi + 1) {
        for (var yi: i32 = -RADIUS; yi <= RADIUS; yi = yi + 1) {
            let offset = vec2<f32>(f32(xi), f32(yi)) * texel;
            let c = textureSample(myTexture, mySampler, uv + offset).rgb;

            if (xi <= 0 && yi <= 0) {
                mean[0] = mean[0] + c;
                varSum[0] = varSum[0] + dot(c, c);
                count[0] = count[0] + 1.0;
            }
            if (xi > 0 && yi <= 0) {
                mean[1] = mean[1] + c;
                varSum[1] = varSum[1] + dot(c, c);
                count[1] = count[1] + 1.0;
            }
            if (xi <= 0 && yi > 0) {
                mean[2] = mean[2] + c;
                varSum[2] = varSum[2] + dot(c, c);
                count[2] = count[2] + 1.0;
            }
            if (xi > 0 && yi > 0) {
                mean[3] = mean[3] + c;
                varSum[3] = varSum[3] + dot(c, c);
                count[3] = count[3] + 1.0;
            }
        }
    }

    var final_col: vec3<f32> = mean[0] / count[0];
    var minVar: f32 = varSum[0] / count[0] - dot(final_col, final_col);

    for (var i: i32 = 1; i < 4; i = i + 1) {
        let m = mean[i] / count[i];
        let v = varSum[i] / count[i] - dot(m, m);
        if (v < minVar) {
            final_col = m;
            minVar = v;
        }
    }

    return final_col;
}

// === Fragment ===
@fragment
fn fs_main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    let color = Kuwahara(uv);
    let edge = Sobel(uv);
    let edgeStrength = clamp(edge * 0.25, 0.0, 1.0);

    let finalColor = mix(color, vec3<f32>(0.0), edgeStrength);
    return vec4<f32>(finalColor, 1.0);
}