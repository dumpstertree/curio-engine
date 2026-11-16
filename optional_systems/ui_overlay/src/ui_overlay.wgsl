// One quad per UI box plus glyphs from the 3D text's shared font atlas:
// positions arrive as normalized device coordinates the pass already
// resolved, colours as rgba, and flat boxes carry uv (-1, -1) as the
// "no texture" marker.

@group(0) @binding(0) var font_atlas: texture_2d<f32>;
@group(0) @binding(1) var font_sampler: sampler;

struct VSOut {
    // the position built-in is a vec4 in clip space — wgpu rejects a vec2
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tint: vec4<f32>,
    @location(1) uv: vec2<f32>,
};

@vertex
fn vs_main(
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) tint: vec4<f32>,
) -> VSOut {
    var out: VSOut;
    // positions are already NDC from the pass; z stays 0 and w stays 1
    out.clip_position = vec4<f32>(position, 0.0, 1.0);
    out.tint = tint;
    out.uv = uv;
    return out;
}

@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
    // the pipeline blends with the post-processed scene, so the box can
    // be partly transparent. text quads multiply the glyph texture by the
    // tint — the same convention as the unlit shader that paints 3D text
    let sampled = textureSample(font_atlas, font_sampler, in.uv);
    return select(in.tint, in.tint * sampled, in.uv.x >= 0.0);
}
