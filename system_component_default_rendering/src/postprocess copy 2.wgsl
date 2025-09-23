@group(0) @binding(1) var myTex: texture_2d<f32>;
@group(0) @binding(0) var mySampler: sampler;

struct VSOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_fullscreen(@builtin(vertex_index) i: u32) -> VSOut {
    var pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>( 3.0,  1.0),
        vec2<f32>(-1.0,  1.0),
    );
    var out: VSOut;
    out.pos = vec4<f32>(pos[i], 0.0, 1.0);
    out.uv = (out.pos.xy + vec2(1.0)) * 0.5;
    return out;
}

@fragment
fn fs_fullscreen(in: VSOut) -> @location(0) vec4<f32> {
    let color = textureSample(myTex, mySampler, in.uv);
    // apply post effect here
    return vec4<f32>(1.0 - color.rgb, 1.0); // simple invert effect
}
