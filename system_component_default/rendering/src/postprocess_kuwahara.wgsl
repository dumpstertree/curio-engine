// === Bindings ===
// group(0) binding(0): sampler
// group(0) binding(1): texture2D (previous post-process stage)
// group(0) binding(2): depth texture (reserved for the depth-aware stage)
// group(0) binding(3): KuwaharaParams uniform

struct KuwaharaParams {
    radius: f32, // kernel radius in pixels
    strength: f32, // mix between the original color and the painterly color
};

@group(0) @binding(0)
var mySampler: sampler;

@group(0) @binding(1)
var myTexture: texture_2d<f32>;

@group(0) @binding(3)
var<uniform> kuwahara_params: KuwaharaParams;

// --- vertex output ---
struct VSOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

// --- fullscreen triangle ---
@vertex
fn vs_fullscreen(@builtin(vertex_index) i: u32) -> VSOut {
    var pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>( 3.0,  1.0),
        vec2<f32>(-1.0,  1.0),
    );

    var out: VSOut;
    out.pos = vec4<f32>(pos[i], 0.0, 1.0);

    // map NDC [-1,1] to UV [0,1]
    out.uv = (out.pos.xy + vec2<f32>(1.0, 1.0)) * 0.5;

    // flip Y if your texture is upside down
    out.uv.y = 1.0 - out.uv.y;

    return out;
}

// --- Kuwahara filter ---
// radius comes from the uniform so the host can drive it per frame
fn Kuwahara(uv: vec2<f32>, radius: i32) -> vec3<f32> {
    var mean: array<vec3<f32>, 4>;
    var varSum: array<f32, 4>;
    var count: array<f32, 4>;
    for (var i: i32 = 0; i < 4; i = i + 1) {
        mean[i] = vec3<f32>(0.0);
        varSum[i] = 0.0;
        count[i] = 0.0;
    }

    let dims = vec2<f32>(textureDimensions(myTexture));
    let texel = 1.0 / dims;

    for (var xi: i32 = -radius; xi <= radius; xi = xi + 1) {
        for (var yi: i32 = -radius; yi <= radius; yi = yi + 1) {
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

// --- fragment shader ---
@fragment
fn fs_fullscreen(in: VSOut) -> @location(0) vec4<f32> {
    let radius = max(1, i32(round(kuwahara_params.radius)));
    let original = textureSample(myTexture, mySampler, in.uv).rgb;
    let painterly = Kuwahara(in.uv, radius);
    let color = mix(original, painterly, clamp(kuwahara_params.strength, 0.0, 1.0));
    return vec4<f32>(color, 1.0);
}
