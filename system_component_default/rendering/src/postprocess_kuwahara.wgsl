// === Bindings ===
// group(0) binding(0): sampler
// group(0) binding(1): texture2D (previous post-process stage)
// group(0) binding(2): depth texture (depth2D, fetched with textureLoad)
// group(0) binding(3): KuwaharaParams uniform
// group(0) binding(4): flow field texture (smoothed E,F,G tensor from the
//                      sobel+blur passes, fetched with textureLoad)

struct KuwaharaParams {
    radius: f32, // kernel radius in pixels
    strength: f32, // mix between the original color and the painterly color
    sigma: f32, // gaussian spread ACROSS the edge (px); 0 = uniform weights
    tau: f32, // gaussian spread ALONG the edge (px); 0 = uniform weights
    dither_mode: f32, // 0 = interleaved gradient noise, >= 1 = hashed blue noise
    dither_amount: f32, // 0 = hard decisions (old behavior), 1 = fully stochastic
    seed: f32, // shifts the hash lattice (and could drive temporal dithering)
    depth_strength: f32, // 0 = depth-ignorant; > 0 falloff of window weights across depth jumps
    temporal: f32, // 0 = static spatial dither; > 0 re-randomizes the noise each frame
    frame: f32, // CPU frame counter (wraps at 1024), drives the re-randomization
    stroke: f32, // stretch sub-windows ALONG the edge (0 = square, 2 = 3x long)
    stride: f32, // sampling step inside windows; 0 = auto (2 once radius >= 8)
};

struct Tensor2 {
    xx: f32,
    yy: f32,
    xy: f32,
};

@group(0) @binding(0)
var mySampler: sampler;

@group(0) @binding(1)
var myTexture: texture_2d<f32>;

@group(0) @binding(2)
var myDepth: texture_depth_2d;

@group(0) @binding(3)
var<uniform> kuwahara_params: KuwaharaParams;

@group(0) @binding(4)
var flowTex: texture_2d<f32>;

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

// gaussian weight with axis scales in pixels; <=0 on either axis disables the
// weighting entirely (uniform weights), so sigma=tau=0 reproduces the old
// plain-window accumulation
fn gaussWeight(u: f32, v: f32, sigma: f32, tau: f32) -> f32 {
    if (sigma <= 0.0 || tau <= 0.0) {
        return 1.0;
    }
    let a = 0.5 * (u * u) / (sigma * sigma);
    let b = 0.5 * (v * v) / (tau * tau);
    return exp(-(a + b));
}

// raw window-space depth at uv (0..1 for geometry, cleared to 1.0 = far plane);
// fetched with textureLoad: depth textures cannot be linear-filtered, and every
// tap is texel-centered, so the fetch is exact (clamped like ClampToEdge)
fn depthAt(uv: vec2<f32>) -> f32 {
    let dims = textureDimensions(myDepth); // vec2<u32>
    let coord = vec2<i32>(
        clamp(i32(uv.x * f32(dims.x)), 0, i32(dims.x) - 1),
        clamp(i32(uv.y * f32(dims.y)), 0, i32(dims.y) - 1),
    );
    return textureLoad(myDepth, coord, 0);
}

// multiplicative falloff for depth discontinuities: a sample whose depth
// differs from the window center by dz gets exp(-strength * dz); strength = 0
// keeps the filter fully depth-ignorant
fn depthWeight(dz: f32, strength: f32) -> f32 {
    if (strength <= 0.0) {
        return 1.0;
    }
    return exp(-strength * abs(dz));
}

// temporal dither noise: with temporal > 0 the spatial dither noise is mixed
// with a copy whose lattice (pixel jitter for IGN) and seed (for the hash)
// change every frame. selection-boundary pixels then oscillate between the
// candidate sub-windows frame to frame, so temporally averaged output
// converges toward the weighted blend instead of a hard per-pixel decision
fn painterlyNoise(uv: vec2<f32>) -> f32 {
    let dims = vec2<f32>(textureDimensions(myTexture));
    let base = ditherNoise(uv * dims, kuwahara_params.dither_mode, kuwahara_params.seed);
    let amt = clamp(kuwahara_params.temporal, 0.0, 1.0);
    if (amt <= 0.0) {
        return base;
    }
    // irrational per-frame lattice offsets: never repeats on a short cycle
    let j = fract(kuwahara_params.frame * vec2<f32>(1.61803398, 0.61803398));
    let shifted = ditherNoise(
        uv * dims + j * 32.0,
        kuwahara_params.dither_mode,
        kuwahara_params.seed + kuwahara_params.frame,
    );
    return mix(base, shifted, amt);
}

// sampling step inside the sub-windows: explicit stride if > 0, otherwise
// auto — every-other sample once the kernel is big (radius >= 8), which keeps
// the per-pixel tap count roughly flat for the large strokes the look wants
fn subwindowStep(radius: i32) -> i32 {
    if (kuwahara_params.stride > 0.0) {
        return max(1, i32(round(kuwahara_params.stride)));
    }
    if (radius >= 8) {
        return 2;
    }
    return 1;
}

// --- Kuahara filter, anisotropic ---
// orientation comes from the smoothed flow field (E,F,G blurred gradient
// covariance of luminance + k*depth), so stroke direction is coherent across
// large areas instead of jittering with each pixel's own window. theta is the
// gradient direction ACROSS the edge; the kernel frame is rotated fully by it
// (no (1-A) fade needed: flat areas have near-null tensors AND square windows
// because stretch is gated by anisotropy below). in the rotated frame u is
// across the edge and v runs along it
fn Kuahara(uv: vec2<f32>, px: vec2<i32>, radius: i32, sigma: f32, tau: f32) -> vec3<f32> {
    let tf = textureLoad(flowTex, px, 0);
    let t = Tensor2(tf.x, tf.z, tf.y);
    let halfTrace = 0.5 * (t.xx + t.yy);
    let disc = sqrt(0.25 * (t.xx - t.yy) * (t.xx - t.yy) + t.xy * t.xy);
    let lam1 = halfTrace + disc;
    let lam2 = max(halfTrace - disc, 0.0);
    let anisotropy = clamp(lam2 / max(lam1, 0.0001), 0.0, 1.0);

    // fold the kernel rotation into [-45, 45] degrees: the four-quadrant
    // kernel is symmetric under a 90-degree rotation
    var theta = 0.5 * atan2(2.0 * t.xy, t.xx - t.yy);
    if (theta > 0.7853981) {
        theta = theta - 1.5707962;
    }
    if (theta < -0.7853981) {
        theta = theta + 1.5707962;
    }

    let c = cos(theta);
    let s = sin(theta);

    var meanColor: array<vec3<f32>, 4>;
    var weight: array<f32, 4>;
    var sl: array<f32, 4>;
    var slu: array<f32, 4>;
    var slv: array<f32, 4>;
    var suu: array<f32, 4>;
    var svv: array<f32, 4>;
    var suv: array<f32, 4>;
    for (var q: i32 = 0; q < 4; q = q + 1) {
        meanColor[q] = vec3<f32>(0.0);
        weight[q] = 0.0;
        sl[q] = 0.0;
        slu[q] = 0.0;
        slv[q] = 0.0;
        suu[q] = 0.0;
        svv[q] = 0.0;
        suv[q] = 0.0;
    }

    let dims = vec2<f32>(textureDimensions(myTexture));
    let texel = 1.0 / dims;

    // depth of the window center: sub-windows straddle a depth silhouette when
    // their samples fall off by depthWeight, so they end up covering one
    // depth layer only
    let dCenter = depthAt(uv);

    // elongated facets, gated on the SAME anisotropy factor as theta: ~1 =
    // isotropic variation (flat or blobby) -> square windows; ~0 = strong
    // edge -> long window. unstretched-flat is what keeps smooth areas clean
    // big facets instead of stamping axis-aligned checkerboard rectangles
    // whose quadrant means differ whenever a shadow or silhouette falls
    // inside the window
    let stretch = 1.0 + kuwahara_params.stroke * (1.0 - anisotropy);
    let r_u = max(1, i32(round(f32(radius) / stretch))); // across the edge
    let r_v = max(1, i32(round(f32(radius) * stretch))); // along the edge
    let step = subwindowStep(radius);

    // sample the four sub-windows in the rotated frame
    for (var q: i32 = 0; q < 4; q = q + 1) {
        let su = select(-1.0, 1.0, q == 1 || q == 3);
        let sv = select(-1.0, 1.0, q == 2 || q == 3);
        for (var iu: i32 = 0; iu <= r_u; iu = iu + step) {
            for (var iv: i32 = 0; iv <= r_v; iv = iv + step) {
                let lu = f32(iu) * su;
                let lv = f32(iv) * sv;
                // rotate the local offset into texture space
                let rx = c * lu - s * lv;
                let ry = s * lu + c * lv;
                let sample_uv = uv + vec2<f32>(rx, ry) * texel;
                let w = gaussWeight(lu, lv, sigma, tau)
                    * depthWeight(depthAt(sample_uv) - dCenter, kuwahara_params.depth_strength);
                let col = textureSample(myTexture, mySampler, sample_uv).rgb;
                let lum = max(dot(col, vec3<f32>(0.2126, 0.7152, 0.0722)), 0.0001);
                let lw = lum * w;

                meanColor[q] = meanColor[q] + col * w;
                weight[q] = weight[q] + w;
                sl[q] = sl[q] + lw;
                slu[q] = slu[q] + lw * lu;
                slv[q] = slv[q] + lw * lv;
                suu[q] = suu[q] + lw * lu * lu;
                svv[q] = svv[q] + lw * lv * lv;
                suv[q] = suv[q] + lw * lu * lv;
            }
        }
    }

    // pick the sub-window whose structure tensor has the smallest minor
    // eigenvalue: least variation across the edge
    // track the winner AND the runner-up so the selection can be dithered
    // between the two most plausible sub-windows instead of always taking
    // the argmin
    var best: i32 = 0;
    var runner: i32 = 0;
    var bestLambda2: f32 = 1000000.0;
    var runnerLambda2: f32 = 1000000.0;
    var lam1Best: f32 = 0.0001;
    for (var q: i32 = 0; q < 4; q = q + 1) {
        let lw = max(sl[q], 0.0001);
        let mu = slu[q] / lw;
        let mv = slv[q] / lw;
        let cuu = max(suu[q] / lw - mu * mu, 0.0);
        let cvv = max(svv[q] / lw - mv * mv, 0.0);
        let cuv = suv[q] / lw - mu * mv;
        let tr = 0.5 * (cuu + cvv);
        let d = sqrt(0.25 * (cuu - cvv) * (cuu - cvv) + cuv * cuv);
        let l2 = max(tr - d, 0.0);
        if (l2 < bestLambda2) {
            runnerLambda2 = bestLambda2;
            runner = best;
            bestLambda2 = l2;
            lam1Best = tr + d;
            best = q;
        } else if (l2 < runnerLambda2 && q != best) {
            runnerLambda2 = l2;
            runner = q;
        }
    }

    // dither the selection: w1 = 1 - lambda2/lambda1 of the winning quadrant's
    // tensor is the confidence that it really wins. dither_amount scales the
    // noise jitter: 0 = hard decision (old behavior), 1 = fully stochastic
    let noise = painterlyNoise(uv);
    let amount = max(0.0, kuwahara_params.dither_amount);
    let w1 = clamp(1.0 - bestLambda2 / max(lam1Best, 0.0001), 0.0, 1.0);
    let wsel = clamp(w1 + (noise - 0.5) * amount, 0.0, 1.0);
    let c0 = meanColor[best] / max(weight[best], 0.0001);
    let c1 = meanColor[runner] / max(weight[runner], 0.0001);
    return ditherPickVec3(c0, c1, wsel, noise);
}

// --- fragment shader ---
@fragment
fn fs_fullscreen(in: VSOut) -> @location(0) vec4<f32> {
    let radius = max(1, i32(round(kuwahara_params.radius)));
    let sigma = max(0.0, kuwahara_params.sigma);
    let tau = max(0.0, kuwahara_params.tau);
    let original = textureSample(myTexture, mySampler, in.uv).rgb;
    let painterly = Kuahara(
        in.uv,
        vec2<i32>(i32(in.pos.x), i32(in.pos.y)),
        radius,
        sigma,
        tau,
    );
    // same jitter budget on the strength blend as on the quadrant selection:
    // amount = 0 reproduces the old uniform mix, 1 = fully stochastic per pixel
    let dims = vec2<f32>(textureDimensions(myTexture));
    let noise = painterlyNoise(in.uv);
    let amount = max(0.0, kuwahara_params.dither_amount);
    var strength = clamp(kuwahara_params.strength + (noise - 0.5) * amount, 0.0, 1.0);
    // depth-edge strength restoration removed: it painted a 1-px dotted
    // "original" border along every silhouette (per-pixel mix of painted and
    // raw pixels). the reference look wants painterly edges with slight
    // stroke OVERSHOOT instead, and depthWeight inside the kernel already
    // keeps sub-windows from averaging across depth silhouettes
    let color = mix(original, painterly, strength);
    return vec4<f32>(color, 1.0);
}
