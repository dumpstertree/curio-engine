// === Bindings ===
// group(0) binding(0): sampler
// group(0) binding(1): texture2D

@group(0) @binding(0)
var mySampler: sampler;

@group(0) @binding(1)
var myTexture: texture_2d<f32>;

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

    let dims = vec2<f32>(textureDimensions(myTexture));
    let texel = 1.0 / dims;

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

// --- fragment shader ---
@fragment
fn fs_fullscreen(in: VSOut) -> @location(0) vec4<f32> {
    let color = Kuwahara(in.uv);
    return vec4<f32>(color.rgb, 1.0);
}
