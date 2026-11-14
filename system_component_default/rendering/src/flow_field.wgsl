// === flow field passes ===
// written once per frame before the kuwahara pass; the painterly kernel then
// reads the smoothed orientation with a single textureLoad instead of
// estimating a tensor over its own window per pixel.
// RUNS ON THE HALF-RES GUIDE GRID: the sobel reads the downsampled source
// (bound by the CPU wrapper) so its taps span 2 capture px each, and the
// fixed 7-tap blur widens accordingly on screen — quarter of the pixels for
// an orientation field the kernel only samples once
//
// group(0) binding(0): source color texture (textureLoad; sobel pass)
//                     / blurred tensor texture (blur pass, same slot)
// group(0) binding(1): scene depth (textureLoad, sobel pass only)
// group(0) binding(2): FlowParams uniform

struct FlowParams {
    dir: vec2<f32>, // blur axis; (1,0) horizontal, (0,1) vertical, unused by sobel
    k_depth: f32, // weight of the depth-gradient tensor (0 = color only)
    pad: f32,
};

@group(0) @binding(0)
var srcTex: texture_2d<f32>;

@group(0) @binding(1)
var srcDepth: texture_depth_2d;

@group(0) @binding(2)
var<uniform> flow_params: FlowParams;

// --- fullscreen triangle ---
@vertex
fn vs_flow(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    var pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>( 3.0,  1.0),
        vec2<f32>(-1.0,  1.0),
    );
    return vec4<f32>(pos[i], 0.0, 1.0);
}

// --- sobel pass: per-pixel gradient covariance (E, F, G) ---
fn lumAt(px: vec2<i32>) -> f32 {
    let d = vec2<u32>(textureDimensions(srcTex));
    let c = vec2<i32>(
        clamp(px.x, 0, i32(d.x) - 1),
        clamp(px.y, 0, i32(d.y) - 1),
    );
    let col = textureLoad(srcTex, c, 0).rgb;
    return dot(col, vec3<f32>(0.2126, 0.7152, 0.0722));
}

// srcTex is the half-res guide grid while srcDepth stays at full capture res:
// raw guide coordinates would index the wrong quarter of the depth buffer and
// halve its gradients, so the sobel's depth term fought the colour term out
// of phase. map the guide texel through normalized uv into depth texels
fn depthAtPx(px: vec2<i32>) -> f32 {
    let sd = textureDimensions(srcDepth);
    let cd = textureDimensions(srcTex);
    let sx = f32(sd.x) / f32(cd.x);
    let sy = f32(sd.y) / f32(cd.y);
    let c = vec2<i32>(
        clamp(i32((f32(px.x) + 0.5) * sx), 0, i32(sd.x) - 1),
        clamp(i32((f32(px.y) + 0.5) * sy), 0, i32(sd.y) - 1),
    );
    return textureLoad(srcDepth, c, 0);
}

@fragment
fn fs_sobel(@builtin(position) in: vec4<f32>) -> @location(0) vec4<f32> {
    let px = vec2<i32>(i32(in.x), i32(in.y));

    // central differences: one tap each side per axis; the separable blur
    // that follows supplies the smoothing, so no 3x3 stencil is needed
    let gx = lumAt(px + vec2<i32>(1, 0)) - lumAt(px - vec2<i32>(1, 0));
    let gy = lumAt(px + vec2<i32>(0, 1)) - lumAt(px - vec2<i32>(0, 1));

    let dzx = depthAtPx(px + vec2<i32>(1, 0)) - depthAtPx(px - vec2<i32>(1, 0));
    let dzy = depthAtPx(px + vec2<i32>(0, 1)) - depthAtPx(px - vec2<i32>(0, 1));
    // window depth is non-linear: scale the jump by inverse window depth so
    // near and far silhouettes carry comparable gradient strength
    let ds = 1.0 / max(depthAtPx(px), 0.02);
    let w = flow_params.k_depth * ds * ds;

    // blended covariance tensor: luminance gradients + k * depth gradients.
    // depth part is what makes strokes follow geometry instead of albedo
    let ex = gx * gx + w * dzx * dzx;
    let ey = gy * gy + w * dzy * dzy;
    let exy = gx * gy + w * dzx * dzy;
    return vec4<f32>(ex, exy, ey, 0.0);
}

// --- separable gaussian blur of the tensor, 7 taps along flow_params.dir ---
@fragment
fn fs_blur(@builtin(position) in: vec4<f32>) -> @location(0) vec4<f32> {
    let px = vec2<i32>(i32(in.x), i32(in.y));
    let d = vec2<u32>(textureDimensions(srcTex));
    var acc = vec4<f32>(0.0);
    var sum = 0.0;
    let ws = array<f32, 7>(1.0, 3.0, 5.0, 7.0, 5.0, 3.0, 1.0);
    for (var o: i32 = -3; o <= 3; o = o + 1) {
        let c = vec2<i32>(
            clamp(px.x + i32(flow_params.dir.x * f32(o)), 0, i32(d.x) - 1),
            clamp(px.y + i32(flow_params.dir.y * f32(o)), 0, i32(d.y) - 1),
        );
        let wgt = ws[o + 3];
        acc = acc + textureLoad(srcTex, c, 0) * wgt;
        sum = sum + wgt;
    }
    return acc / sum;
}
