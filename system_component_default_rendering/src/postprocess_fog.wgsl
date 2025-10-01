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

// --- Human-editable constants ---
const FOG_COLOR: vec3<f32>   = vec3<f32>(0.7, 0.8, 0.9); // light blue fog
const FOG_NEAR: f32          = 15.0;  // start applying fog
const FOG_FAR: f32           = 100.0; // fully fogged
const CAMERA_NEAR: f32       = 0.1;  // must match your camera near
const CAMERA_FAR: f32        = 512.0; // must match your camera far
const DEPTH_EPSILON: f32     = 0.999; // threshold for "no object" (far plane)

// --- Vertex output ---
struct VSOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

// --- Fullscreen triangle ---
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
    // flip Y if offscreen texture is upside down
    out.uv.y = 1.0 - out.uv.y;

    return out;
}

// --- Linearize depth ---
fn linearizeDepth(depth: f32, near: f32, far: f32) -> f32 {
    let z_ndc: f32 = depth * 2.0 - 1.0;
    return (2.0 * near * far) / (far + near - z_ndc * (far - near));
}

// --- Fragment shader: apply fog only if object exists ---
@fragment
fn fs_fullscreen(in: VSOut) -> @location(0) vec4<f32> {
    // fetch color
    let color = textureSample(myTexture, mySampler, in.uv);

    // fetch depth
    let dims_u: vec2<u32> = textureDimensions(myDepthTex);
    let coord: vec2<i32> = vec2<i32>(i32(in.uv.x * f32(dims_u.x)), i32(in.uv.y * f32(dims_u.y)));
    let depthRaw: f32 = textureLoad(myDepthTex, clamp(coord, vec2<i32>(0,0), vec2<i32>(i32(dims_u.x)-1, i32(dims_u.y)-1)), 0);

    // skip fog if depth is at far plane (no geometry)
    // if (depthRaw >= DEPTH_EPSILON) {
    //     return color;
    // }

    // linearize depth
    let linearDepth: f32 = linearizeDepth(depthRaw, CAMERA_NEAR, CAMERA_FAR);

    // compute fog factor (0 = near, 1 = far)
    let fogFactor: f32 = clamp((linearDepth - FOG_NEAR) / (FOG_FAR - FOG_NEAR), 0.0, 1.0);

    // mix scene color with fog color
    let finalColor: vec3<f32> = mix(color.rgb, FOG_COLOR, fogFactor);

    return vec4<f32>(finalColor, color.a);
}
// @fragment
// fn fs_fullscreen(in: VSOut) -> @location(0) vec4<f32> {
//         // let color = textureSample(myTexture, mySampler, in.uv);

//     // return vec4<f32>(color.rgb, 1.0);
//     return vec4<f32>(1.0, 0.0, 1.0, 1.0); // bright magenta
// }