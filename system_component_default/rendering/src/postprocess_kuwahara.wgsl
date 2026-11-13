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

// --- Kuwahara filter, contrast-invariant via structure tensor ---
// For each of the four quadrant windows we accumulate the 2x2 structure tensor
//     S = sum( lum * w * w^T )
// over the sample offsets w inside the window, weighted by luminance.
// Eigen-decomposing S gives the window's dominant direction and how much
// variation runs ACROSS it:
//     lambda1 >= lambda2;  lambda2 is the variation across the dominant axis.
// Picking the window with the SMALLEST lambda2 keeps windows that run ALONG
// an edge rather than across it (Fuyama, Schuippel, Kuwahara 2007). Unlike the
// classic mean/variance rule this is invariant to region brightness, so it
// survives soft gradients, speculars and shading boundaries.

fn Kuahawara(uv: vec2<f32>, radius: i32) -> vec3<f32> {
    var mean: array<vec3<f32>, 4>;
    var count: array<f32, 4>;
    var sl: array<f32, 4>; // sum of the luminance weights
    var slx: array<f32, 4>; // first moments of the sample offsets
    var sly: array<f32, 4>;
    var sxx: array<f32, 4>; // second moments: the structure tensor entries
    var syy: array<f32, 4>;
    var sxy: array<f32, 4>;

    for (var i: i32 = 0; i < 4; i = i + 1) {
        mean[i] = vec3<f32>(0.0);
        count[i] = 0.0;
        sl[i] = 0.0;
        slx[i] = 0.0;
        sly[i] = 0.0;
        sxx[i] = 0.0;
        syy[i] = 0.0;
        sxy[i] = 0.0;
    }

    let dims = vec2<f32>(textureDimensions(myTexture));
    let texel = 1.0 / dims;

    for (var xi: i32 = -radius; xi <= radius; xi = xi + 1) {
        for (var yi: i32 = -radius; yi <= radius; yi = yi + 1) {
            let offset = vec2<f32>(f32(xi), f32(yi)) * texel;
            let c = textureSample(myTexture, mySampler, uv + offset).rgb;
            let lum = max(dot(c, vec3<f32>(0.2126, 0.7152, 0.0722)), 0.001);
            let wx = f32(xi);
            let wy = f32(yi);

            // 0:(-x,-y) 1:(+x,-y) 2:(-x,+y) 3:(+x,+y)
            let sx = select(0, 1, xi > 0);
            let sy = select(0, 1, yi > 0);
            let q = sx + 2 * sy;

            mean[q] = mean[q] + c;
            count[q] = count[q] + 1.0;
            sl[q] = sl[q] + lum;
            slx[q] = slx[q] + lum * wx;
            sly[q] = sly[q] + lum * wy;
            sxx[q] = sxx[q] + lum * wx * wx;
            syy[q] = syy[q] + lum * wy * wy;
            sxy[q] = sxy[q] + lum * wx * wy;
        }
    }

    // pick the quadrant whose structure tensor has the smallest minor eigenvalue
    var best: i32 = 0;
    var bestLambda2: f32 = 1000000.0;
    for (var q: i32 = 0; q < 4; q = q + 1) {
        let lw = max(sl[q], 0.001);
        let mx = slx[q] / lw;
        let my = sly[q] / lw;
        // central second moments: the tensor covariance of the window
        let cxx = max(sxx[q] / lw - mx * mx, 0.0);
        let cyy = max(syy[q] / lw - my * my, 0.0);
        let cxy = sxy[q] / lw - mx * my;
        // eigenvalues of the 2x2 tensor
        let halfTrace = 0.5 * (cxx + cyy);
        let disc = sqrt(0.25 * (cxx - cyy) * (cxx - cyy) + cxy * cxy);
        let lambda2 = max(halfTrace - disc, 0.0);
        if (lambda2 < bestLambda2) {
            bestLambda2 = lambda2;
            best = q;
        }
    }

    return mean[best] / max(count[best], 1.0);
}

// --- fragment shader ---
@fragment
fn fs_fullscreen(in: VSOut) -> @location(0) vec4<f32> {
    let radius = max(1, i32(round(kuwahara_params.radius)));
    let original = textureSample(myTexture, mySampler, in.uv).rgb;
    let painterly = Kuahawara(in.uv, radius);
    let color = mix(original, painterly, clamp(kuwahara_params.strength, 0.0, 1.0));
    return vec4<f32>(color, 1.0);
}
