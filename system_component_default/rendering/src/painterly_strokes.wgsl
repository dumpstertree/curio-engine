// === Painterly stamping passes ===
// hybrid layer: the PREVIOUS stage in the chain (the flow-driven kuwahara) is
// the COLOUR GUIDE, not the final picture. pass 1 blits that guide to the
// output; pass 2 stamps feathered elliptical daubs over it with ALPHA_BLENDING.
//
// no stroke list exists: every daub's parameters (anchor, size, angle, colour)
// are recomputed in the vertex stage from its slot index, so there are no
// atomics, no compaction and no storage buffer. slot anchors are points of the
// R2 low-discrepancy sequence plus hash jitter, which is why the look has no
// visible lattice at all; slots that fail the keep test collapse to a
// degenerate quad (zero rasterised area).
//
// group(0) binding(0): sampler (blit pass only)
// group(0) binding(1): guide texture (previous stage; the STAMP VERTEX stage
//                       sobels it for size and angle, so this binding is
//                       visible to both stages)
// group(0) binding(2): StrokeParams uniform (12 scalars + 4 pads puts vp on
//                       offset 64, matching the #[repr(C)] CPU mirror; total
//                       128 bytes)

struct StrokeParams {
    size: f32, // long-axis daub length in the FLAT-area case, px
    size_min: f32, // fraction of size kept where local contrast is maxed
    mag_scale: f32, // contrast sensitivity: gradient magnitude * this -> 0..1
    elongation: f32, // long/short aspect in flat areas (1 = round daub)
    aspect_min: f32, // aspect floor in busy areas (refs round small daubs)
    alpha: f32, // stamp opacity
    feather: f32, // 0 = hard-edged stamp, 1 = gradient all the way in
    keep: f32, // keep probability per slot (0..1); failures degenerate
    jitter: f32, // anchor jitter as a fraction of one slot cell
    wobble: f32, // angle jitter amplitude in radians
    seed: f32, // shifts every hash (moves the whole daub layout)
    viewport_count: f32, // number of valid vp rects (1..4)
    pad_x: f32, // scalar pads: uniform-space composites align to 16
    pad_y: f32, // vp is a composite, so it must land on offset 64
    pad_z: f32, // the CPU mirror pads identically
    pad_w: f32,
    vp: array<vec4<f32>, 4>, // viewport rects in uv: minx, miny, maxx, maxy
};

// the slot grid is compile-time because it sizes the draw range: one draw
// covers every slot, six vertices each (two triangles, no vertex buffer)
const SLOT_COUNT: i32 = 1024;

@group(0) @binding(0)
var paintSampler: sampler;

@group(0) @binding(1)
var guideTex: texture_2d<f32>;

@group(0) @binding(2)
var<uniform> paint_params: StrokeParams;

// --- blit pass: guide copied through untouched, so the daubs read as an
// overlay on a coherent painterly palette instead of raw scene colour ---
struct BlitOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_fullscreen(@builtin(vertex_index) i: u32) -> BlitOut {
    var pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>( 3.0,  1.0),
        vec2<f32>(-1.0,  1.0),
    );

    var out: BlitOut;
    out.pos = vec4<f32>(pos[i], 0.0, 1.0);

    // same uv convention as every other post pass: uv is texture space, so
    // ndc.y = 1 - 2*uv.y and texture taps are uv * dimensions directly
    out.uv = (out.pos.xy + vec2<f32>(1.0, 1.0)) * 0.5;
    out.uv.y = 1.0 - out.uv.y;

    return out;
}

@fragment
fn fs_blit(in: BlitOut) -> @location(0) vec4<f32> {
    return textureSample(guideTex, paintSampler, in.uv);
}

// --- stamp pass helpers ---
// one-tap-each-side central differences of the GUIDE (not of a shared flow
// texture: the guide is what this feature is guaranteed to own, which keeps
// the pass self-contained at the cost of a slightly less smooth orientation)
fn lumAtGuide(px: vec2<i32>) -> f32 {
    let d = textureDimensions(guideTex);
    let c = vec2<i32>(
        clamp(px.x, 0, i32(d.x) - 1),
        clamp(px.y, 0, i32(d.y) - 1),
    );
    let col = textureLoad(guideTex, c, 0).rgb;
    return dot(col, vec3<f32>(0.2126, 0.7152, 0.0722));
}

// daub colour is sampled once at the centre in the vertex stage: a FLAT
// colour per stamp is what makes the result read as discrete paint instead of
// a blur, which is the whole point of the stamp method
fn colorAtGuide(px: vec2<i32>) -> vec3<f32> {
    let d = textureDimensions(guideTex);
    let c = vec2<i32>(
        clamp(px.x, 0, i32(d.x) - 1),
        clamp(px.y, 0, i32(d.y) - 1),
    );
    return textureLoad(guideTex, c, 0).rgb;
}

// the viewport rect owning an anchor: stamps never spill across a split-screen
// seam, so the rect is baked into the varyings and the fragment discards
fn paintViewportForUv(uv: vec2<f32>) -> vec4<f32> {
    let count = clamp(i32(round(paint_params.viewport_count)), 1, 4);
    for (var i: i32 = 0; i < count; i = i + 1) {
        let r = paint_params.vp[i];
        if (uv.x >= r.x && uv.x <= r.z && uv.y >= r.y && uv.y <= r.w) {
            return r;
        }
    }
    return vec4<f32>(0.0, 0.0, 1.0, 1.0);
}

// --- stamp vertex: six vertices per slot, parameters derived from the slot ---
struct StampOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) local: vec2<f32>, // ellipse space: |local| <= 1 is inside
    @location(1) uv: vec2<f32>, // texture-space uv, for the seam discard
    @location(2) rect: vec4<f32>, // owning viewport rect (constant per stamp)
    @location(3) color: vec3<f32>, // flat guide colour at the daub centre
};

@vertex
fn vs_stamp(@builtin(vertex_index) i: u32) -> StampOut {
    let corners = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>(-1.0,  1.0),
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(-1.0,  1.0),
        vec2<f32>( 1.0,  1.0),
    );

    var out: StampOut;
    let slot = i / 6u;
    let s = f32(slot);
    let seed = paint_params.seed;

    // keep test first: a rejected slot degenerates the whole quad, which costs
    // nothing in the rasteriser and needs no list compaction
    let keepNoise = ditherHash(vec2<f32>(s, 137.0 + seed), seed);
    if (keepNoise >= paint_params.keep) {
        out.pos = vec4<f32>(0.0, 0.0, 0.0, 1.0);
        out.local = vec2<f32>(0.0, 0.0);
        out.uv = vec2<f32>(0.0, 0.0);
        out.rect = vec4<f32>(0.0, 0.0, 1.0, 1.0);
        out.color = vec3<f32>(0.0, 0.0, 0.0);
        return out;
    }

    // R2 low-discrepancy anchors: the pair (frac(i*phi), frac(i*psi)) has
    // star-like distribution, so no cell structure exists to see; jitter then
    // breaks the near-regular rows the sequence alone still shows
    var anchor = vec2<f32>(
        fract(s * 1.61803399),
        fract(s * 1.20606794),
    );
    let cell = 1.0 / sqrt(f32(SLOT_COUNT));
    let jx = ditherHash(vec2<f32>(s, 11.0 + seed), seed) - 0.5;
    let jy = ditherHash(vec2<f32>(s, 71.0 + seed), seed) - 0.5;
    anchor = anchor + vec2<f32>(jx, jy) * paint_params.jitter * cell;
    anchor = anchor - floor(anchor); // wrap back into the texture

    let guideDims = textureDimensions(guideTex); // vec2<u32>
    let dims = vec2<f32>(f32(guideDims.x), f32(guideDims.y));
    let center = anchor * dims; // texture pixels
    let pxi = vec2<i32>(i32(center.x), i32(center.y));

    // local contrast decides BOTH scale and shape, which is the reference
    // behaviour: a flat area gets a big long daub, a busy area gets a small
    // round one. the guide is already smoothed, so these magnitudes are much
    // gentler than raw-scene sobel magnitudes would be
    let gx = lumAtGuide(pxi + vec2<i32>(1, 0)) - lumAtGuide(pxi - vec2<i32>(1, 0));
    let gy = lumAtGuide(pxi + vec2<i32>(0, 1)) - lumAtGuide(pxi - vec2<i32>(0, 1));
    let e = gx * gx + gy * gy;
    let contrast = clamp(sqrt(e) * paint_params.mag_scale, 0.0, 1.0);

    let longPx = mix(paint_params.size, paint_params.size * paint_params.size_min, contrast);
    let aspect = max(mix(paint_params.elongation, paint_params.aspect_min, contrast), 1.0);
    let halfLong = longPx * 0.5;
    let halfShort = halfLong / aspect;

    // draw ALONG the edge: the gradient (gx, gy) points across the fastest
    // change, so the tangent is (-gy, gx); a near-flat neighbourhood has no
    // meaningful tangent, and hashed angle there keeps flat areas varied
    var angle = (ditherHash(vec2<f32>(s, 211.0 + seed), seed) - 0.5) * 6.2831853;
    if (e > 0.000001) {
        angle = atan2(gx, -gy);
    }
    angle = angle + (ditherHash(vec2<f32>(s, 307.0 + seed), seed) - 0.5) * paint_params.wobble;

    let ca = cos(angle);
    let sa = sin(angle);

    // corner offset in texture pixels (column, row), then converted to ndc;
    // rows point down while ndc.y points up, hence the negated row
    let c = corners[i % 6u];
    let ox = c.x * halfLong;
    let oy = c.y * halfShort;
    let dx = (ca * ox - sa * oy) / dims.x;
    let dy = (sa * ox + ca * oy) / dims.y;

    out.pos = vec4<f32>(
        (center.x / dims.x + dx) * 2.0 - 1.0,
        1.0 - (center.y / dims.y + dy) * 2.0,
        0.0,
        1.0,
    );

    // this uv comes from the CORNER POSITION, not the anchor: it interpolates
    // to the real texture coordinate of each fragment, so the seam discard
    // trims a daub that straddles a viewport boundary (baking the anchor here
    // would make the test constant per stamp and let it bleed across)
    out.uv = vec2<f32>(
        (out.pos.x + 1.0) * 0.5,
        1.0 - (out.pos.y + 1.0) * 0.5,
    );
    out.local = c;
    out.rect = paintViewportForUv(anchor);
    out.color = colorAtGuide(pxi);

    return out;
}

@fragment
fn fs_stamp(in: StampOut) -> @location(0) vec4<f32> {
    // seam clip: the daub belongs to one viewport rect, so it must not bleed
    // into the neighbouring screen
    let r = in.rect;
    if (in.uv.x < r.x || in.uv.x > r.z || in.uv.y < r.y || in.uv.y > r.w) {
        discard;
    }

    // feathered elliptical brush alpha: analytic for now, and this is the one
    // slot where the game can later swap in real brush art
    let d = sqrt(dot(in.local, in.local));
    let soft = clamp(1.0 - paint_params.feather, 0.0, 0.999);
    let cover = 1.0 - smoothstep(soft, 1.0, d);
    if (cover <= 0.0) {
        discard;
    }

    return vec4<f32>(in.color, cover * paint_params.alpha);
}
