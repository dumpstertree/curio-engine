// === Bindings ===
// group(0) binding(0): sampler
// group(0) binding(1): color texture
// group(0) binding(2): depth texture

@group(0) @binding(0)
var mySampler: sampler;

@group(0) @binding(1)
var myTexture: texture_2d<f32>;

@group(0) @binding(2)
var myDepthTex: texture_depth_2d;

// --- Editable constants ---
const LUMA_WEIGHTS: vec3<f32>       = vec3<f32>(0.3, 0.59, 0.11);
const EDGE_STRENGTH_SCALE: f32     = 0.25;   // scales Sobel intensity
const EDGE_THRESHOLD: f32          = 0.25;   // discard below this combined strength
const DEPTH_EDGE_SCALE: f32        = 25.0;   // scales depth-edge visibility
const DEPTH_THRESHOLD: f32         = 0.1;    // min linear depth diff considered an edge
const CAMERA_NEAR: f32             = 0.1;    // MUST match your camera near
const CAMERA_FAR: f32              = 100.0;  // MUST match your camera far

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

    // flip Y if your offscreen texture is upside down
    out.uv.y = 1.0 - out.uv.y;

    return out;
}

// --- Linearize depth (depth in [0,1] -> linear view-space depth) ---
fn linearizeDepth(depth: f32, near: f32, far: f32) -> f32 {
    let z_ndc: f32 = depth * 2.0 - 1.0;
    return (2.0 * near * far) / (far + near - z_ndc * (far - near));
}

// --- Sobel edge detection (color-based) ---
fn Sobel(uv: vec2<f32>) -> f32 {
    let dims_u: vec2<u32> = textureDimensions(myTexture); // returns u32
    let dims_f: vec2<f32> = vec2<f32>(f32(dims_u.x), f32(dims_u.y));
    let texel: vec2<f32> = 1.0 / dims_f;

    var gx: f32 = 0.0;
    var gy: f32 = 0.0;

    for (var xi: i32 = -1; xi <= 1; xi = xi + 1) {
        for (var yi: i32 = -1; yi <= 1; yi = yi + 1) {
            let offset = vec2<f32>(f32(xi), f32(yi)) * texel;
            let c = textureSample(myTexture, mySampler, uv + offset).rgb;
            let lum = dot(c, LUMA_WEIGHTS);

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

// --- Depth-based edge detection (uses texel fetch / textureLoad) ---
fn DepthEdge(uv: vec2<f32>) -> f32 {
    // textureDimensions for depth returns vec2<u32>
    let dims_u: vec2<u32> = textureDimensions(myDepthTex);
    let max_coord: vec2<i32> = vec2<i32>(i32(dims_u.x) - 1, i32(dims_u.y) - 1);

    // convert uv -> integer texel coords (clamp to valid range)
    var coord: vec2<i32> = vec2<i32>(i32(uv.x * f32(dims_u.x)), i32(uv.y * f32(dims_u.y)));
    coord = clamp(coord, vec2<i32>(0, 0), max_coord);

    // fetch center depth (textureLoad returns f32 for depth textures)
    let centerRaw: f32 = textureLoad(myDepthTex, coord, 0);
    let centerLinear: f32 = linearizeDepth(centerRaw, CAMERA_NEAR, CAMERA_FAR);

    var maxDiff: f32 = 0.0;
    for (var xi: i32 = -1; xi <= 1; xi = xi + 1) {
        for (var yi: i32 = -1; yi <= 1; yi = yi + 1) {
            if (xi == 0 && yi == 0) { continue; }
            var neighborCoord: vec2<i32> = coord + vec2<i32>(xi, yi);
            neighborCoord = clamp(neighborCoord, vec2<i32>(0, 0), max_coord);

            let neighborRaw: f32 = textureLoad(myDepthTex, neighborCoord, 0);
            let neighborLinear: f32 = linearizeDepth(neighborRaw, CAMERA_NEAR, CAMERA_FAR);

            let diff: f32 = abs(centerLinear - neighborLinear);
            maxDiff = max(maxDiff, diff);
        }
    }

    // threshold + scale
    return step(DEPTH_THRESHOLD, maxDiff) * clamp(maxDiff * DEPTH_EDGE_SCALE, 0.0, 1.0);
}

// --- fragment shader ---
@fragment
fn fs_fullscreen(in: VSOut) -> @location(0) vec4<f32> {
    // color sample (if you want to overlay later)
    let color = textureSample(myTexture, mySampler, in.uv);

    let sobelStrength: f32 = clamp(Sobel(in.uv) * EDGE_STRENGTH_SCALE, 0.0, 1.0);
    let depthStrength: f32 = DepthEdge(in.uv);
    let combined: f32 = max(sobelStrength, depthStrength);

    if (combined < EDGE_THRESHOLD) {
        return vec4<f32>(color.rgb, 1.0);
    }

    // return black outline; change to mix(color, ...) to overlay
    return vec4<f32>(color.rgb * 0.3, 1.0);
}
