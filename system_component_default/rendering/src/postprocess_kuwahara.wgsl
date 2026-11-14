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
    viewport_count: f32, // number of valid vp rects (1..4)
    pad_x: f32, // scalar pads: uniform-space composites align to 16, so vp
    pad_y: f32, // lands at offset 64 on the GPU; the CPU mirror pads the same
    pad_z: f32, // way to stay byte-identical
    vp: array<vec4<f32>, 4>, // viewport rects in uv: minx, miny, maxx, maxy
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

// the viewport rect containing this uv (uv itself is inside a viewport);
// taps are then clamped to it so sub-windows never read across the split-
// screen seam. falls back to the full texture if nothing matches
fn viewportForUv(uv: vec2<f32>) -> vec4<f32> {
    let count = clamp(i32(round(kuwahara_params.viewport_count)), 1, 4);
    for (var i: i32 = 0; i < count; i = i + 1) {
        let r = kuwahara_params.vp[i];
        if (uv.x >= r.x && uv.x <= r.z && uv.y >= r.y && uv.y <= r.w) {
            return r;
        }
    }
    return vec4<f32>(0.0, 0.0, 1.0, 1.0);
}

fn clampToViewport(uv: vec2<f32>, r: vec4<f32>) -> vec2<f32> {
    return vec2<f32>(clamp(uv.x, r.x, r.z), clamp(uv.y, r.y, r.w));
}

// multiplicative falloff for depth discontinuities: a sample whose depth
// differs from the window center by dz gets exp(-strength * dz); strength = 0
// keeps the filter fully depth-ignorant
fn depthWeight(dz: f32, strength: f32) -> f32 {
    if (strength <= 0.0) {
        return 1.0;
    }
    // HARD chop now: samples jumping past 1/strength contribute nothing, so a
    // window mean never carries a blended grey halo across a silhouette (the
    // soft exp falloff bled a 1-cell smear along the canopy edges). windows
    // falling entirely on another depth layer are made unselectable in the
    // selection pass below
    if (abs(dz) * strength < 1.0) {
        return 1.0;
    }
    return 0.0;
}

// deterministic scalar noise at an arbitrary pixel position, with the same
// temporal re-randomization painterlyNoise applies
fn noiseAtPxTemporal(px: vec2<f32>) -> f32 {
    let base = ditherNoise(px, kuwahara_params.dither_mode, kuwahara_params.seed);
    let amt = clamp(kuwahara_params.temporal, 0.0, 1.0);
    if (amt <= 0.0) {
        return base;
    }
    let j = fract(kuwahara_params.frame * vec2<f32>(1.61803398, 0.61803398));
    let shifted = ditherNoise(
        px + j * 32.0,
        kuwahara_params.dither_mode,
        kuwahara_params.seed + kuwahara_params.frame,
    );
    return mix(base, shifted, amt);
}

// per-cell stable hash for the stroke-lattice jitter (salt picks independent
// streams: anchor x, anchor y, territory size)
fn cellHash(px: vec2<f32>, salt: f32) -> f32 {
    return ditherNoise(px, kuwahara_params.dither_mode, kuwahara_params.seed + salt);
}

// temporal dither noise: with temporal > 0 the spatial dither noise is mixed
// with a copy whose lattice (pixel jitter for IGN) and seed (for the hash)
// change every frame. selection-boundary pixels then oscillate between the
// candidate sub-windows frame to frame, so temporally averaged output
// converges toward the weighted blend instead of a hard per-pixel decision
//
// the noise is sampled on a COARSE lattice (one value per radius-sized cell):
// whole stroke regions then pick the same candidate sub-window, giving crisp
// stroke-sized borders and stroke-scale overshoot at silhouettes instead of
// 1-px sawtooth fringe
fn painterlyNoise(uv: vec2<f32>) -> f32 {
    let dims = vec2<f32>(textureDimensions(myTexture));
    let block = max(1.0, round(kuwahara_params.radius));
    let cell = floor((uv * dims) / block);
    let px = (cell + 0.5) * block; // sample at the cell centre
    return noiseAtPxTemporal(px);
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

// shared frame computation from a raw flow tensor: (theta bucket, stretch,
// detail-shrink). theta is folded into [-45, 45] degrees (the four-quadrant
// kernel is symmetric under 90-degree rotation) and bucketed to four
// directions so a whole region shares one blob orientation. detail-shrink
// f = 1/(1 + lam1*40): where local variation is dense (distant trees, plank
// stacks, albedo detail) windows AND blob territories shrink together, so
// detail stays crisp small strokes instead of dissolving at full stroke size
fn flowFrame(tf: vec4<f32>) -> vec3<f32> {
    let t = Tensor2(tf.x, tf.z, tf.y);
    let halfTrace = 0.5 * (t.xx + t.yy);
    let disc = sqrt(0.25 * (t.xx - t.yy) * (t.xx - t.yy) + t.xy * t.xy);
    let lam1 = halfTrace + disc;
    let lam2 = max(halfTrace - disc, 0.0);
    let anisotropy = clamp(lam2 / max(lam1, 0.0001), 0.0, 1.0);
    var theta = 0.5 * atan2(2.0 * t.xy, t.xx - t.yy);
    if (theta > 0.7853981) {
        theta = theta - 1.5707962;
    }
    if (theta < -0.7853981) {
        theta = theta + 1.5707962;
    }
    let thetaBucket = round(theta / 0.3926991) * 0.3926991;
    let stretch = 1.0 + kuwahara_params.stroke * (1.0 - anisotropy);
    let f = 1.0 / (1.0 + max(lam1, 0.0) * 40.0);
    return vec3<f32>(thetaBucket, stretch, max(f, 0.2));
}

// --- Kuahara filter, anisotropic ---
// orientation comes from the smoothed flow field (E,F,G blurred gradient
// covariance of luminance + k*depth), so stroke direction is coherent across
// large areas instead of jittering with each pixel's own window. theta is the
// gradient direction ACROSS the edge; the kernel frame is rotated fully by it
// (no (1-A) fade needed: flat areas have near-null tensors AND square windows
// because stretch is gated by anisotropy below). in the rotated frame u is
// across the edge and v runs along it
//
// STROKE LATTICE: windows, orientation and the selection noise all anchor on
// the centre of the same radius-sized cell, so every pixel in a cell shares
// ONE window mean and ONE candidate: the stroke is a discrete flat patch with
// a hard border, like the reference paint, instead of a gradient that varies
// continuously from pixel to pixel
fn Kuahara(uv: vec2<f32>, radius: i32, sigma: f32, tau: f32) -> vec3<f32> {
    let dims = vec2<f32>(textureDimensions(myTexture));
    let texel = 1.0 / dims;

    // stroke lattice v3: jittered blobs in the STROKE frame. each base cell
    // hashes a jittered centre (rotated into its own theta-bucket frame) and
    // owns territory under ANISOTROPIC distance measured in its window
    // metric (across-edge tight, along-edge long) -> blobs become ellipses
    // stretched along the local edge, so patch borders run along strokes and
    // contours stop stair-stepping across them. the detail-shrink of dense
    // areas scales both blob and window, keeping far/detail features crisp
    let B = max(1.0, round(kuwahara_params.radius));
    let pxF = uv * dims;
    let cell0 = vec2<i32>(floor(pxF / B));
    var bestScore = 1000000.0;
    var anchorF = vec2<f32>(0.0);
    var winF = 1.0; // winning blob's detail-shrink: kernel windows follow it
    for (var oy: i32 = -1; oy <= 1; oy = oy + 1) {
        for (var ox: i32 = -1; ox <= 1; ox = ox + 1) {
            let c = cell0 + vec2<i32>(ox, oy);
            let cf = (vec2<f32>(c) + vec2<f32>(0.5, 0.5)) * B; // base centre px
            let cfPx = vec2<i32>(
                clamp(i32(round(cf.x)), 0, i32(dims.x) - 1),
                clamp(i32(round(cf.y)), 0, i32(dims.y) - 1),
            );
            let fr = flowFrame(textureLoad(flowTex, cfPx, 0));
            let frT = fr.x;
            let frStretch = fr.y;
            let frF = fr.z;
            let jx = cellHash(cf, 31.7) - 0.5;
            let jy = cellHash(cf, 57.3) - 0.5;
            let tw = 0.6 + 0.8 * cellHash(cf, 89.1);
            let rc = cos(frT);
            let rs = sin(frT);
            let jxR = rc * jx - rs * jy;
            let jyR = rs * jx + rc * jy;
            let ac = cf + vec2<f32>(jxR, jyR) * B * 0.5;
            // distance measured in the candidate's window metric: r_u across
            // the edge, r_v along it, both scaled by the detail shrink
            let dxp = pxF.x + 0.5 - ac.x;
            let dyp = pxF.y + 0.5 - ac.y;
            let du = (rc * dxp + rs * dyp) / max(B * frF / frStretch, 1.0);
            let dv = (rs * dxp - rc * dyp) / max(B * frF * frStretch, 1.0);
            let score = (du * du + dv * dv) / max(tw, 0.0001);
            if (score < bestScore) {
                bestScore = score;
                anchorF = ac;
                winF = frF;
            }
        }
    }
    let anchorPx = vec2<i32>(
        clamp(i32(round(anchorF.x)), 0, i32(dims.x) - 1),
        clamp(i32(round(anchorF.y)), 0, i32(dims.y) - 1),
    );
    let vpr = viewportForUv(uv);
    let auv = clampToViewport((anchorF + vec2<f32>(0.5, 0.5)) * texel, vpr);

    let tf = textureLoad(flowTex, anchorPx, 0);
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

    // depth of the window anchor: sub-windows straddle a depth silhouette when
    // their samples fall off by depthWeight, so they end up covering one
    // depth layer only
    let dCenter = depthAt(auv);

    // elongated facets, gated on the SAME anisotropy factor as theta: ~1 =
    // isotropic variation (flat or blobby) -> square windows; ~0 = strong
    // edge -> long window. unstretched-flat is what keeps smooth areas clean
    // big facets instead of stamping axis-aligned checkerboard rectangles
    // whose quadrant means differ whenever a shadow or silhouette falls
    // inside the window
    let stretch = 1.0 + kuwahara_params.stroke * (1.0 - anisotropy);
    // detail-aware: the winning blob's shrink scales the window with its
    // territory, so dense-variation areas sample fine and stay crisp
    let r_u = max(2, i32(round(f32(radius) * winF / stretch))); // across the edge
    let r_v = max(2, i32(round(f32(radius) * winF * stretch))); // along the edge
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
                let sample_uv = clampToViewport(auv + vec2<f32>(rx, ry) * texel, vpr);
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

    // variance-weighted selection over all four windows: score = 1/(1+(k*
    // relative variance)^8) with relative variance = the window's minor
    // eigenvalue over the mean of the four. a window straddling an edge or
    // silhouette blows its variance up and its share collapses toward zero,
    // so the pick is DETERMINISTIC there and silhouettes get clean stroke-
    // sized borders instead of speckle fringe; in flat areas the windows are
    // near-equivalent and the cell noise decides, so neighbouring cells
    // commit to different windows: the patchwork the references get their
    // variation from. dither_amount blends scores toward uniform (0 = pure
    // variance preference, 1 = fully stochastic roulette)
    var l2s: array<f32, 4>;
    var meanL2 = 0.0;
    var nValid = 0.0;
    for (var q: i32 = 0; q < 4; q = q + 1) {
        if (weight[q] < 0.0005) {
            // hard-chopped window: entirely another depth layer, unselectable
            l2s[q] = 1000000.0;
            continue;
        }
        let lw = max(sl[q], 0.0001);
        let mu = slu[q] / lw;
        let mv = slv[q] / lw;
        let cuu = max(suu[q] / lw - mu * mu, 0.0);
        let cvv = max(svv[q] / lw - mv * mv, 0.0);
        let cuv = suv[q] / lw - mu * mv;
        let tr = 0.5 * (cuu + cvv);
        let d = sqrt(0.25 * (cuu - cvv) * (cuu - cvv) + cuv * cuv);
        l2s[q] = max(tr - d, 0.0);
        meanL2 = meanL2 + l2s[q];
        nValid = nValid + 1.0;
    }
    meanL2 = meanL2 / max(nValid, 1.0);

    let amount = max(0.0, kuwahara_params.dither_amount);
    var scores: array<f32, 4>;
    var total = 0.0;
    for (var q: i32 = 0; q < 4; q = q + 1) {
        scores[q] = 0.0;
        if (weight[q] >= 0.0005) {
            let rel = l2s[q] / max(meanL2, 0.0001);
            let rel2 = rel * rel;
            let rel8 = rel2 * rel2 * rel2 * rel2;
            scores[q] = mix(1.0 / (1.0 + rel8), 1.0, amount);
        }
        total = total + scores[q];
    }

    // degenerate: every window chopped out (anchor sits on a lone texel of
    // its own depth layer) -> fall back to the raw texel, never a black patch
    if (total < 0.0001) {
        return textureSample(myTexture, mySampler, auv).rgb;
    }

    // weighted roulette on the winning blob's noise value (blob-coherent: the
    // anchor is identical for every pixel of the blob)
    let noise = noiseAtPxTemporal(anchorF);
    var picked: i32 = 3;
    var acc = 0.0;
    for (var q: i32 = 0; q < 4; q = q + 1) {
        acc = acc + scores[q] / max(total, 0.0001);
        if (noise < acc) {
            picked = q;
            break;
        }
    }

    return meanColor[picked] / max(weight[picked], 0.0001);
}

// --- fragment shader ---
@fragment
fn fs_fullscreen(in: VSOut) -> @location(0) vec4<f32> {
    let radius = max(1, i32(round(kuwahara_params.radius)));
    let sigma = max(0.0, kuwahara_params.sigma);
    let tau = max(0.0, kuwahara_params.tau);
    let original = textureSample(myTexture, mySampler, in.uv).rgb;
    let painterly = Kuahara(in.uv, radius, sigma, tau);
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
