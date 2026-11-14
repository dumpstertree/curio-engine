// === Painterly stamping passes ===
// hybrid layer: the PREVIOUS stage in the chain (the flow-driven kuwahara) is
// the COLOUR GUIDE, not the final picture. pass 1 blits that guide to the
// output; pass 2 stamps feathered elliptical daubs over it with ALPHA_BLENDING.
//
// no stroke list exists: every daub's parameters (anchor, size, angle, colour)
// are recomputed in the vertex stage from its slot index, so there are no
// atomics, no compaction and no storage buffer. slot anchors are two
// independent hashes per slot; a low-discrepancy sequence was tried and its
// point SET shows diagonal band structure at any fixed N, which reads as a
// static screen-space lattice no probe fix can remove; slots that fail the
// keep test collapse to a degenerate quad (zero rasterised area).
//
// group(0) binding(0): sampler (blit pass only)
// group(0) binding(1): guide texture (the COLOUR source; only the stamp
//                       VERTEX samples it, plus the blit FRAGMENT, so this
//                       binding is visible to both stages)
// group(0) binding(2): StrokeParams uniform (15 scalars + 1 pad puts vp on
//                       offset 64, matching the #[repr(C)] CPU mirror; total
//                       128 bytes)
// group(0) binding(3): brush alpha texture (stamp FRAGMENT only; one
//                       procedural bristle mask shared by every daub)
// group(0) binding(4): ORIGINAL FRAME (stamp VERTEX only; every size and
//                       orientation probe runs on the continuous scene
//                       gradients, never on the guide: the guide is
//                       piecewise CONSTANT, so window differences over a
//                       flat block read exactly null and the whole stamp
//                       layer degenerates to a static screen-space lattice)

struct StrokeParams {
    size: f32, // long-axis daub length in the FLAT-area case, px
    size_min: f32, // fraction of size kept where local contrast is maxed
    mag_scale: f32, // contrast sensitivity: SLOPE (luminance per px) * this -> 0..1
    elongation: f32, // long/short aspect where the neighbourhood is anisotropic
    aspect_min: f32, // aspect where the neighbourhood has no edge to follow
    alpha: f32, // stamp opacity
    feather: f32, // 0 = hard-edged stamp, 1 = gradient all the way in
    keep: f32, // keep probability per slot (0..1); failures degenerate
    jitter: f32, // anchor jitter as a fraction of one slot cell
    wobble: f32, // angle jitter amplitude in radians
    seed: f32, // shifts every hash (moves the whole daub layout)
    viewport_count: f32, // number of valid vp rects (1..4)
    flow: f32, // how much the daub SHAPE follows the tensor's anisotropy
    bristle: f32, // how much the brush art drives the stamp alpha
    color_jitter: f32, // per-stamp colour jitter amplitude (0 = guide colour)
    pad_x: f32, // one pad only: 15 scalars now, vp is a composite and must
    vp: array<vec4<f32>, 4>, // viewport rects in uv: minx, miny, maxx, maxy
};

// the slot grid is compile-time because it sizes the draw range: one draw
// covers every slot, six vertices each (two triangles, no vertex buffer).
// 1024 slots left the picture under-painted: mean spacing was ~45 px at a
// 1920x1080 capture against a 22 px daub, so only busy edges caught any
// paint. 4096 puts mean spacing at ~22 px, i.e. one daub per coverage cell
const SLOT_COUNT: i32 = 4096;

@group(0) @binding(0)
var paintSampler: sampler;

@group(0) @binding(1)
var guideTex: texture_2d<f32>;

@group(0) @binding(2)
var<uniform> paint_params: StrokeParams;

@group(0) @binding(3)
var brushTex: texture_2d<f32>;

@group(0) @binding(4)
var sceneTex: texture_2d<f32>;

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
// scale and orientation come from WINDOW DIFFERENCES ON THE ORIGINAL FRAME,
// not on the guide (not from a shared flow texture either: the feature still
// owns no kuwahara resources). WHY THE GUIDE CANNOT BE THE PROBE SOURCE:
// the kuwahara output is piecewise constant, so a pair of tap windows
// standing inside one colour block differences to exactly zero most of the
// time, the tensor reads null, and every stamp falls back to the same
// default shape at a hashed angle - a static screen-space lattice smeared
// over the picture, which is what the earlier versions shipped. the RAW
// frame is continuous: its gradient varies with where the window sits and
// with the actual edge angle, so the tensor is meaningful everywhere the
// scene has detail. textureSample is fragment-only in WebGPU and these
// taps run in the VERTEX stage, so smoothing stays a tap-box average
fn colorAtGuide(px: vec2<i32>) -> vec3<f32> {
    let d = textureDimensions(guideTex);
    let c = vec2<i32>(
        clamp(px.x, 0, i32(d.x) - 1),
        clamp(px.y, 0, i32(d.y) - 1),
    );
    return textureLoad(guideTex, c, 0).rgb;
}

fn lumAtScene(px: vec2<i32>) -> f32 {
    let d = textureDimensions(sceneTex);
    let c = vec2<i32>(
        clamp(px.x, 0, i32(d.x) - 1),
        clamp(px.y, 0, i32(d.y) - 1),
    );
    let col = textureLoad(sceneTex, c, 0).rgb;
    return dot(col, vec3<f32>(0.2126, 0.7152, 0.0722));
}

// mean luminance over a 3x3 tap box on the original frame, so the
// gradient varies with WHERE the window sits instead of with which axis
// a single tap pair straddles
fn lumBoxScene(cx: f32, cy: f32) -> f32 {
    let bx = i32(round(cx));
    let by = i32(round(cy));
    var sum = 0.0;
    for (var oy: i32 = -1; oy <= 1; oy = oy + 1) {
        for (var ox: i32 = -1; ox <= 1; ox = ox + 1) {
            sum = sum + lumAtScene(vec2<i32>(bx + ox, by + oy));
        }
    }
    return sum * (1.0 / 9.0);
}

// window-mean derivative of the ORIGINAL FRAME along ONE unit direction:
// central difference of two 3x3 tap boxes divided by the displacement, so
// the result is a slope (luminance per pixel) instead of a raw jump and
// stays comparable across daub sizes
fn dirDeriv(px: vec2<f32>, d: vec2<f32>, r: f32) -> f32 {
    let a = lumBoxScene(px.x + d.x * r, px.y + d.y * r);
    let b = lumBoxScene(px.x - d.x * r, px.y - d.y * r);
    return (a - b) / max(2.0 * r, 0.0001);
}

// the frame's local second moment (E, F, G) of its gradient, recovered from
// FOUR directional probes at 0/45/90/135 degrees. that quartet is a tight
// frame, so the squared slopes pin the tensor exactly: E = q(0), G = q(90)
// and F = (q(45) - q(135)) / 2, since q(t) = E cos^2 + 2F cos sin + G sin^2.
// this is the SAME covariance the kuwahara accumulates per quad over its
// sobel flow textures, estimated here at the daub's own extent with tap
// boxes in place of its blur passes; taken at the extent the stamp covers,
// so a daub always belongs to the edge it sits on, and continuous because
// it probes the raw frame rather than the piecewise-constant guide
fn sceneTensor(px: vec2<f32>, radius: f32) -> vec3<f32> {
    var q0 = dirDeriv(px, vec2<f32>(1.0, 0.0), radius);
    q0 = q0 * q0;
    var q90 = dirDeriv(px, vec2<f32>(0.0, 1.0), radius);
    q90 = q90 * q90;
    var q45 = dirDeriv(px, vec2<f32>(0.7071068, 0.7071068), radius);
    q45 = q45 * q45;
    var q135 = dirDeriv(px, vec2<f32>(-0.7071068, 0.7071068), radius);
    q135 = q135 * q135;
    return vec3<f32>(q0, 0.5 * (q45 - q135), q90);
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

    // hashed anchors, NOT a low-discrepancy sequence: R2 looked like it
    // spread evenly, but its 1024-point SET has band structure (the
    // three-distance theorem: consecutive points lie on three families of
    // parallel lines, slope psi/phi), and that reads exactly as the
    // "static screenspace diagonal lattice" seen in every build - the
    // pattern never depended on the scene, so no probe fix could ever
    // remove it. two independent per-slot hashes have no ordering
    // structure at all: evenness comes from the count and the keep test,
    // clumping is fine and reads as grouped brushwork
    var anchor = vec2<f32>(
        ditherHash(vec2<f32>(s, 3.0 + seed), seed),
        ditherHash(vec2<f32>(s, 97.0 + seed), seed),
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

    // local contrast AND local anisotropy of the RAW FRAME drive the stamp:
    // the frame is continuous where the guide is not, so these magnitudes
    // vary with the real surface detail (cracks, grout, studs) instead of
    // collapsing to zero over every kuwahara block, and the tensor is read
    // over the extent the stamp covers, so a daub always belongs to the
    // edge it sits on
    let gradRadius = clamp(paint_params.size * 0.35, 1.5, 24.0);
    let tf = sceneTensor(center + vec2<f32>(0.5, 0.5), gradRadius);

    // a stamp sitting exactly on a grout or crack line straddles it
    // SYMMETRICALLY, so every probe window pairs into matching blocks and
    // the tensor reads near null - which used to mean "flat area, take the
    // big daub", painting dark smoke blobs along every grout line. compare
    // the centre against a ring of window boxes at half the probe radius:
    // a dark centre with a lighter ring is a line, and a line is busy, so
    // the stamp shrinks to roughly the line's width instead of blooming
    let midR = gradRadius * 0.5;
    let cx = center.x + 0.5;
    let cy = center.y + 0.5;
    let centerLum = lumBoxScene(cx, cy);
    let ringMean = (
        lumBoxScene(cx + midR, cy) +
        lumBoxScene(cx - midR, cy) +
        lumBoxScene(cx, cy + midR) +
        lumBoxScene(cx, cy - midR)
    ) * 0.25;
    // signed on purpose: only DARK centres bloom (grout and cracks), a
    // bright specular dot keeps the tensor's own verdict
    let lineContrast = clamp((ringMean - centerLum) * 6.0, 0.0, 1.0);

    // eigen decomposition of [[E, F], [F, G]], the same form the kuwahara's
    // flowFrame uses: lam1 is the dominant variance (how STEEP the change is,
    // in luminance per pixel now that dirDeriv divides out the displacement)
    // and lam2/lam1 says how much of it is directional
    let halfTrace = 0.5 * (tf.x + tf.z);
    let disc = sqrt(0.25 * (tf.x - tf.z) * (tf.x - tf.z) + tf.y * tf.y);
    let lam1 = halfTrace + disc;
    let lam2 = max(halfTrace - disc, 0.0);
    let anisotropy = clamp(lam2 / max(lam1, 0.000001), 0.0, 1.0); // 1 = isotropic
    let contrast = clamp(max(sqrt(lam1) * paint_params.mag_scale, lineContrast), 0.0, 1.0);

    let longPx = mix(paint_params.size, paint_params.size * paint_params.size_min, contrast);
    // SHAPE follows the flow: a strongly directional neighbourhood (a real
    // edge, anisotropy near 0) carries the full elongation, a near-isotropic
    // one falls back to aspect_min. flow = 0 keeps one constant elongation
    // for every stamp, flow = 1 makes daubs round wherever there is no edge
    let tensorAspect = paint_params.aspect_min + (paint_params.elongation - paint_params.aspect_min) * (1.0 - anisotropy);
    let aspect = max(mix(paint_params.elongation, tensorAspect, paint_params.flow), 1.0);
    let halfLong = longPx * 0.5;
    let halfShort = halfLong / aspect;

    // theta = 0.5*atan2(2F, E - G) is the direction of MAXIMUM variance, i.e.
    // ACROSS the edge, so the long axis is theta + 90 degrees. the hashed
    // fallback now fires only on genuinely empty regions (bare floor, fog),
    // because the raw frame is never piecewise constant: over the walls the
    // surface detail gives a real tangent for almost every stamp, which is
    // what stops the paint from sitting as a fixed screen-space pattern
    var angle = (ditherHash(vec2<f32>(s, 211.0 + seed), seed) - 0.5) * 6.2831853;
    if (lam1 > 0.000001) {
        angle = 0.5 * atan2(2.0 * tf.y, tf.x - tf.z) + 1.5707963;
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
    // the painted layer MUST deviate from the guide or coverage is invisible
    // by construction: inside a kuwahara block the guide is constant, so a
    // stamp carrying that exact centre colour paints nothing and the flat
    // areas of every earlier build read as bare guide with the kuahara dither
    // artefacts showing through - the coverage was there geometrically, it
    // simply could not be seen. so the deviation is FORCED, not sampled from a
    // range that contains zero: each channel moves by at least 0.6 * amplitude
    // (sign from the hash, magnitude floored) which removes the near-zero
    // samples the earlier uniform spread wasted half its range on. an
    // additive term rides the same sign so the shift survives near-black and
    // near-white regions where a pure multiply collapses to nothing;
    // color_jitter = 0 still reproduces the exact guide colour
    var col = colorAtGuide(pxi);
    let cj = paint_params.color_jitter;
    if (cj > 0.0) {
        let hr = ditherHash(vec2<f32>(s, 541.0 + seed), seed) * 2.0 - 1.0;
        let hl = ditherHash(vec2<f32>(s, 613.0 + seed), seed) * 2.0 - 1.0;
        let hs = ditherHash(vec2<f32>(s, 727.0 + seed), seed) * 2.0 - 1.0;
        // select not sign: sign(0.0) is 0, which would let a rare slot land on
        // the guide colour exactly
        let dr = select(-1.0, 1.0, hr >= 0.0) * (0.6 + 0.4 * abs(hr));
        let dl = select(-1.0, 1.0, hl >= 0.0) * (0.6 + 0.4 * abs(hl));
        let ds = select(-1.0, 1.0, hs >= 0.0) * (0.6 + 0.4 * abs(hs));
        col = col * vec3<f32>(1.0 + cj * dr, 1.0 + cj * dl, 1.0 + cj * ds)
            + vec3<f32>(dr * cj * 0.25, dl * cj * 0.25, ds * cj * 0.25);
    }
    out.color = col;

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

    // brush alpha: the analytic falloff carries the SILHOUETTE, then the
    // procedural bristle mask breaks that silhouette and hollows the body, so
    // the daub reads as the drag of real bristles instead of a clean gradient.
    // bristle = 0 reproduces the old analytic stamp exactly
    let d = sqrt(dot(in.local, in.local));
    let soft = clamp(1.0 - paint_params.feather, 0.0, 0.999);
    var cover = 1.0 - smoothstep(soft, 1.0, d);

    // local is the axis-aligned quad, so the mask stretches along the daub's
    // long axis: the bristle streaks run WITH the stroke instead of across it
    let mask = textureSample(brushTex, paintSampler, (in.local + vec2<f32>(1.0, 1.0)) * 0.5).r;
    cover = cover * mix(1.0, mask, paint_params.bristle);

    if (cover <= 0.0) {
        discard;
    }

    return vec4<f32>(in.color, cover * paint_params.alpha);
}
