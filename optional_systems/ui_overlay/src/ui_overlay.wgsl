// One flat-coloured quad per UI box. The pass did all the maths:
// positions arrive as normalized device coordinates, colours as rgba.

struct VSOut {
    @builtin(position) clip_position: vec2<f32>,
    @location(0) tint: vec4<f32>,
};

@vertex
fn vs_main(
    @location(0) position: vec2<f32>,
    @location(1) tint: vec4<f32>,
) -> VSOut {
    var out: VSOut;
    out.clip_position = position;
    out.tint = tint;
    return out;
}

@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
    // the pipeline blends with the post-processed scene, so the box can
    // be partly transparent
    return in.tint;
}
