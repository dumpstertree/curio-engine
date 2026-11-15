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
// group(0) binding(2): StrokeParams uniform (20 scalars, then a 3-vec4 layers
//                       array at offset 80 and 4-vec4 vp at 128, matching the
//                       #[repr(C)] CPU mirror; total 192 bytes)
// group(0) binding(3): brush alpha texture (stamp FRAGMENT only; one
//                       procedural bristle mask shared by every daub)
// group(0) binding(4): ORIGINAL FRAME (stamp VERTEX only; every size and
//                       orientation probe runs on the continuous scene
//                       gradients, never on the guide: the guide is
//                       piecewise CONSTANT, so window differences over a
//                       flat block read exactly null and the whole stamp
//                       layer degenerates to a static screen-space lattice)
// group(0) binding(5): scene depth (stamp VERTEX: unprojects each slot
//                       anchor to a world point so strokes sit in the scene
//                       instead of sliding over the screen; full capture
//                       resolution)
// group(0) binding(6): WorldAnchorParams uniform (view_proj + its CPU inverse
//                       + camera position + focal length in px, per camera)
//
// temporal history WITHOUT a history texture: a slot's identity is its world
// anchor, which is stable across frames; a CONTINUOUS function of the rolling
// time (StrokeParams.time, advanced per frame on the CPU) keyed on a per-cell
// phase sways each stroke in place, which reads as accumulation-style
// breathing. hashing the time instead re-rolls the offset every frame, and a
// daub that hops 60 times a second reads as the whole field still moving.
// the same rolling clock drives birth/death: each stroke fades up and down
// on a life cycle of its own (StrokeParams.life, 0 = off), so what the
// camera reveals and retires lands mid-fade instead of popping at full
// alpha (there is no stroke list to AGE, so the cycle is periodic: the
// reference's brushwork accumulates and gets brushed out the same way)

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
    world_size: f32, // 0 = daub long axis stays in px; > 0 = WORLD units,
    //                     projected through the focal length so distant
    //                     strokes shrink with the scene
    time: f32, // rolling seconds (CPU-advanced each frame): drives the
    //                     temporal drift of world-anchored daubs
    world_cell: f32, // world-space lattice spacing the hit points snap to
    drift: f32, // world-anchor breathing as a fraction of one lattice cell
    //                     (0 = strokes pinned dead still; the default is a
    //                     sub-pixel sway, not a hop)
    life: f32, // birth/death cycle mean in seconds (0 = hard pops kept):
    //                     each stroke fades up and down on a phase keyed on
    //                     its own identity, softening camera pop-in
    //                     (20 scalars then the two vec4 arrays: 192 bytes)
    layers: array<vec4<f32>, 3>, // per layer (mass, mid, detail): long-axis
    //                     mult, alpha mult, contrast gate lo, gate hi; slot
    //                     index ranges draw mass..mid..detail in submission
    //                     order, so the mass layer sits UNDER the detail,
    //                     which is the reference structure: broad value
    //                     masses first, finer strokes on top, each layer
    //                     weighted (softly feathered) by its own contrast
    //                     band — never hard-culled, so no scene contrast
    //                     distribution can uncover a whole region (the
    //                     research fix for large daubs landing in detailed zones)
    vp: array<vec4<f32>, 4>, // viewport rects in uv: minx, miny, maxx, maxy
};

// the slot grid is compile-time because it sizes the draw range: one draw
// covers every slot, six vertices each (two triangles, no vertex buffer).
// 1024 slots left the picture under-painted: mean spacing was ~45 px at a
// 1920x1080 capture against a 22 px daub, so only busy edges caught any
// paint. 4096 puts mean spacing at ~22 px, i.e. one daub per coverage cell
const SLOT_COUNT: i32 = 4096;

// layer split by slot INDEX range: a slot's ray is a hash of its index, so
// ANY contiguous range is still uniform over the screen, and vertex
// submission order makes the ranges come back-to-front for free: mass is
// stamped under mid is stamped under detail. coverage scales as length
// squared, so the 3x-longer mass layer needs only ~1/9 the slots
const SLOT_MASS_END: u32 = 4096u; // each layer gets a FULL SLOT_COUNT grid:
const SLOT_MID_END: u32 = 8192u; // mass [0,4096), mid [4096,8192), detail the rest
const BAND_FEATHER: f32 = 0.15; // how wide the layer bands overlap: band
//                        membership feathers the layer alpha instead of
//                        hard-culling, so a scene whose contrast distribution
//                        misses the split knobs still paints everywhere

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

@group(0) @binding(5)
var sceneDepthTex: texture_depth_2d;

// world anchor plumbing: the exact view_proj the 3D pass bound, plus its CPU
// inverse. unprojecting the depth buffer through the inverse and re-projecting
// the snapped world point needs no znear/zfar anywhere: the depth texture
// already stores the same non-linear [0, 1] values the projection wrote, and
// the matrices invert and re-apply them as a pair
struct WorldAnchorParams {
    view_proj: mat4x4<f32>, // world -> clip (column-major, as every pass binds it)
    inv_view_proj: mat4x4<f32>, // clip -> world
    cam_pos: vec4<f32>, // world-space camera position (kept for distance fades)
    focal_px: f32, // capture focal length in pixels (y): px per world unit at z = 1
    _pad_x: f32,
    _pad_y: f32,
    _pad_z: f32,
};

@group(0) @binding(6)
var<uniform> world_anchor: WorldAnchorParams;

// capture-space uv (uv.y = 0 at the TOP row, the blit pass' convention) at a
// stored depth value, as a clip-space position for the inverse matrix
fn uvToClip(uv: vec2<f32>, depth: f32) -> vec4<f32> {
    return vec4<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0, depth, 1.0);
}

// depth at a uv of the capture: the depth texture keeps the FULL capture
// resolution while the guide may be half-res, so the uv must scale by the
// texture's own dimensions, never by the guide's
fn sceneDepthUv(uv: vec2<f32>) -> f32 {
    let d = textureDimensions(sceneDepthTex);
    let c = vec2<i32>(
        clamp(i32(uv.x * f32(d.x)), 0, i32(d.x) - 1),
        clamp(i32(uv.y * f32(d.y)), 0, i32(d.y) - 1),
    );
    return textureLoad(sceneDepthTex, c, 0);
}

fn unprojectWorld(uv: vec2<f32>, depth: f32) -> vec3<f32> {
    let w4 = world_anchor.inv_view_proj * uvToClip(uv, depth);
    return w4.xyz / w4.w;
}

// world -> uv.xy of the capture, clip z in .z (still the non-linear [0, 1]
// depth space the texture stores, so it compares straight against a tap) and
// the view distance in .w (<= 0 is behind the camera)
fn projectUv(world: vec3<f32>) -> vec4<f32> {
    let clip = world_anchor.view_proj * vec4<f32>(world, 1.0);
    let ndc = clip.xyz / clip.w;
    return vec4<f32>(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5, ndc.z, clip.w);
}

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
    @location(4) fade: f32, // birth/death ramp, constant per stamp
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

    // layer from the slot range (see SLOT_MASS_END/SLOT_MID_END): mass strokes
    // are submitted FIRST so they rasterise UNDER the mid and detail layers,
    // which is the reference's back-to-front structure for free
    var layer_i: u32 = 2;
    if (slot < SLOT_MASS_END) {
        layer_i = 0;
    } else if (slot < SLOT_MID_END) {
        layer_i = 1;
    }
    let L = paint_params.layers[layer_i];

    // keep test first: a rejected slot degenerates the whole quad, which costs
    // nothing in the rasteriser and needs no list compaction
    let keepNoise = ditherHash(vec2<f32>(s, 137.0 + seed), seed);
    if (keepNoise >= paint_params.keep) {
        out.pos = vec4<f32>(0.0, 0.0, 0.0, 1.0);
        out.local = vec2<f32>(0.0, 0.0);
        out.uv = vec2<f32>(0.0, 0.0);
        out.rect = vec4<f32>(0.0, 0.0, 1.0, 1.0);
        out.color = vec3<f32>(0.0, 0.0, 0.0);
        out.fade = 0.0;
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
    // clumping is fine and reads as grouped brushwork. INDEPENDENT means
    // NO SHARED INPUT COMPONENT: ditherHash returns hash(x) ^ hash(y), so
    // feeding both anchors the same slot coordinate would tie anchor.y to
    // anchor.x by one fixed xor - every point on one digital line, only
    // 1/8 of any cell grid ever receiving a daub (the sparse-scatter bug)
    let cell_px = 1.0 / sqrt(f32(SLOT_COUNT));
    var rayUv = vec2<f32>(
        ditherHash(vec2<f32>(s, 3.0 + seed), seed),
        ditherHash(vec2<f32>(97.0 + seed, s), seed),
    );
    let jx = ditherHash(vec2<f32>(s, 11.0 + seed), seed) - 0.5;
    let jy = ditherHash(vec2<f32>(71.0 + seed, s), seed) - 0.5; // inputs swapped:
    // sharing p.x = s would tie jy to jx by a fixed xor again (sub-cell
    // amplitude, so cosmetic, but the same structural trap)
    rayUv = rayUv + vec2<f32>(jx, jy) * paint_params.jitter * cell_px;
    rayUv = rayUv - floor(rayUv); // wrap back into the texture

    let guideDims = textureDimensions(guideTex); // vec2<u32>
    let dims = vec2<f32>(f32(guideDims.x), f32(guideDims.y));

    // the WORLD anchor (what reference painterly does and the last missing
    // piece here): the hashed uv is only a RAY into the scene. its hit point
    // snaps to a world lattice cell - floor(world / cell_world) is camera-
    // invariant - the daub sits at a cell-hash position inside that cell, and
    // the point is re-projected through the 3D pass' view_proj. anchors then
    // translate, scale and rotate WITH the geometry instead of sliding over
    // the screen, and every hash below keys on the CELL identity, so a stroke
    // keeps its colour and angle while the camera moves: the shimmer of a
    // purely screen-space anchor field disappears. cells stay implicit (no
    // world stroke list): the slot only chooses which ray to shoot, and
    // surfaces the camera reveals between old frames simply get hit later
    let hitDepth = sceneDepthUv(rayUv);
    var anchor = rayUv; // uv of the anchor, whatever the source was
    var hashKey = s; // identity for the angle/colour hashes below
    var viewW = 0.0; // view-space distance (clip.w); 0 = screen fallback
    if (hitDepth < 0.9999) { // a real surface under the ray
        let world = unprojectWorld(rayUv, hitDepth);
        // the lattice scales WITH the layer's stroke length: mass daubs sit
        // one per larger cell instead of many stacking on the same fine cell,
        // and each layer's cells stay camera-invariant identity-wise
        let cell_world = paint_params.world_cell * L.x;
        let cell = floor(world / cell_world);
        // three DIFFERENT cell-coordinate pairs per hash: ditherHash xors a
        // p.x term with a p.y term, so two calls sharing a coordinate pair
        // would be tied by a constant xor, the same structural trap that
        // made the old screen anchor scatter sparse
        let hxy = ditherHash(vec2<f32>(cell.x, cell.y), 0.0);
        let hzx = ditherHash(vec2<f32>(cell.z, cell.x + 17.0), 0.0);
        let hyz = ditherHash(vec2<f32>(cell.y + 41.0, cell.z), 0.0);
        let t = paint_params.time;
        // the drift must be CONTINUOUS in time: ditherHash(cell, t) is a step
        // function, so hashing the clock re-rolls a fresh random offset every
        // frame and each daub HOPS inside its cell 60 times a second (and at
        // 0.1 of a cell that is several px close up), which reads exactly as
        // "the strokes still move". a sinusoid keyed on a per-cell PHASE (a
        // hash of the cell alone, so stable per stroke) sways continuously and
        // returns to its start, which is the breathing accumulation painterly
        // has instead of crawling
        let phase = vec3<f32>(hxy, hyz, hzx) * 6.2831853;
        let drift = vec3<f32>(
            sin(t * 0.7 + phase.x),
            sin(t * 0.9 + phase.y * 0.5),
            sin(t * 1.1 + phase.z * 0.25),
        );
        let anchorWorld = (cell + vec3<f32>(hxy, hyz, hzx) + drift * paint_params.drift) * cell_world;
        let proj = projectUv(anchorWorld);
        // behind the camera: degenerate quad, free
        if (proj.w <= 0.01) {
            out.pos = vec4<f32>(0.0, 0.0, 0.0, 1.0);
            out.local = vec2<f32>(0.0, 0.0);
            out.uv = vec2<f32>(0.0, 0.0);
            out.rect = vec4<f32>(0.0, 0.0, 1.0, 1.0);
            out.color = vec3<f32>(0.0, 0.0, 0.0);
            out.fade = 0.0;
            return out;
        }
        // depth-occlusion: the stored depth at the PROJECTED pixel against
        // the anchor's own clip z (both the same non-linear [0, 1] space, so
        // they compare straight). an anchor floating behind real geometry
        // must not paint over it; the small bias absorbs the non-linearity
        let occ = sceneDepthUv(proj.xy);
        if (occ + 0.002 < proj.z) {
            out.pos = vec4<f32>(0.0, 0.0, 0.0, 1.0);
            out.local = vec2<f32>(0.0, 0.0);
            out.uv = vec2<f32>(0.0, 0.0);
            out.rect = vec4<f32>(0.0, 0.0, 1.0, 1.0);
            out.color = vec3<f32>(0.0, 0.0, 0.0);
            out.fade = 0.0;
            return out;
        }
        // the snapped lattice point may sit up to one (coarse) cell off the
        // SURFACE: the mass layer's 3x cells let a daub float a quarter meter
        // in front of a wall, and such strokes parallax across silhouette
        // edges and paint the wrong geometry (the mush reading over every
        // seam). the cell keeps the IDENTITY; the position re-snaps to the
        // surface at the anchor's own projected pixel — occ already holds
        // that pixel's depth from the occlusion test — so a stroke always
        // sits ON the geometry it was assigned to, and its world size then
        // uses the surface distance, not the float's
        let surf = unprojectWorld(proj.xy, occ);
        let surf_proj = projectUv(surf);
        anchor = surf_proj.xy;
        viewW = surf_proj.w;
        // cell-hash identity: continuous in value but constant per cell
        hashKey = hxy * 131.0 + hzx * 17.0 + hyz;
    }
    // sky / far: no surface to sit on, keep the plain screen anchor there

    // birth/death: every stroke carries its own life cycle keyed on its
    // identity (hashKey: the world cell, or the slot in the sky fallback), so
    // what the camera reveals and retires lands mid-fade instead of popping
    // at full alpha, and silhouette occlusion toggles blink less. there is no
    // stroke list to age, so the cycle is periodic rather than a one-shot
    // birth: the field breathes, which is how the reference's brushwork
    // accumulates and gets brushed out anyway. life = 0 keeps hard pops.
    // checked BEFORE the contrast probes so faded strokes pay no probe cost
    var fade = 1.0;
    if (paint_params.life > 0.0) {
        // per-stroke cycle length around the mean (half to 1.5x) so the
        // cycles never align into one global pulse
        let life_len = max(paint_params.life * (0.5 + ditherHash(vec2<f32>(hashKey, 811.0 + seed), seed)), 0.05);
        let cyc = fract(paint_params.time / life_len + ditherHash(vec2<f32>(hashKey, 977.0 + seed), seed));
        // triangle ramp with a plateau: UP over the first 30 % of a cycle,
        // full through the middle 40 %, DOWN over the last 30 %
        fade = smoothstep(0.0, 0.3, cyc) * (1.0 - smoothstep(0.7, 1.0, cyc));
    }
    // the layer's alpha rides the same interpolator as the birth/death ramp,
    // so an alpha-0 layer degenerates every one of its slots for free
    fade = fade * L.y;
    if (fade < 0.02) {
        out.pos = vec4<f32>(0.0, 0.0, 0.0, 1.0);
        out.local = vec2<f32>(0.0, 0.0);
        out.uv = vec2<f32>(0.0, 0.0);
        out.rect = vec4<f32>(0.0, 0.0, 1.0, 1.0);
        out.color = vec3<f32>(0.0, 0.0, 0.0);
        out.fade = 0.0;
        return out;
    }

    let center = anchor * dims; // texture pixels
    let pxi = vec2<i32>(i32(center.x), i32(center.y));

    // local contrast AND local anisotropy of the RAW FRAME drive the stamp:
    // the frame is continuous where the guide is not, so these magnitudes
    // vary with the real surface detail (cracks, grout, studs) instead of
    // collapsing to zero over every kuwahara block, and the tensor is read
    // over the extent the stamp covers, so a daub always belongs to the
    // edge it sits on
    // the probe covers the extent the stamp draws, so it scales with the
    // layer's long axis too
    let gradRadius = clamp(paint_params.size * L.x * 0.35, 1.5, 24.0);
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

    // the layer's contrast band: a SOFT weight, not a hard reject. hard
    // band-culls make the layer split discontinuous in knob space (a scene
    // whose contrast lands in the wrong band paints NOTHING there, which is
    // how walls came back raw), so band membership feathers the layer's
    // alpha across a feather window instead: mass falls out on busy walls,
    // detail falls out on flat floors, and no contrast value is ever
    // uncovered by all three layers at once
    // the feather only works BETWEEN two layers: mass owns contrast 0 and
    // detail owns 1, so feathering those outer boundaries inward dims flat
    // walls (the mass layer at half weight where the old regime drew at
    // full) and busy edges, and breaks the partition's sum-to-one. only
    // the interior boundaries at the split knobs may feather
    var band_lo = 1.0;
    if (L.z > BAND_FEATHER) {
        band_lo = smoothstep(L.z - BAND_FEATHER, L.z + BAND_FEATHER, contrast);
    }
    var band_hi = 1.0;
    if (L.w < 1.0 - BAND_FEATHER) {
        band_hi = 1.0 - smoothstep(L.w - BAND_FEATHER, L.w + BAND_FEATHER, contrast);
    }
    let band_weight = band_lo * band_hi;
    fade = fade * band_weight;
    // the band weight rides out.fade into the fragment, and a layer weighted
    // to nothing degenerates its slots, keeping the culled layers free
    if (fade < 0.02) {
        out.pos = vec4<f32>(0.0, 0.0, 0.0, 1.0);
        out.local = vec2<f32>(0.0, 0.0);
        out.uv = vec2<f32>(0.0, 0.0);
        out.rect = vec4<f32>(0.0, 0.0, 1.0, 1.0);
        out.color = vec3<f32>(0.0, 0.0, 0.0);
        out.fade = 0.0;
        return out;
    }

    var longPx = mix(paint_params.size, paint_params.size * paint_params.size_min, contrast);
    if (paint_params.world_size > 0.0 && viewW > 0.0) {
        // world-sized daub: a fixed world length through the capture's focal
        // length, so distant strokes shrink WITH the scene; the contrast mix
        // still modulates, busy neighbourhoods take their fraction of it
        longPx = mix(1.0, paint_params.size_min, contrast)
            * world_anchor.focal_px * paint_params.world_size / max(viewW, 0.01);
    }
    // layer size mult: mass is a longer brush over the same contrast mix,
    // in BOTH the px and the world-sized mode
    longPx = longPx * L.x;

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
    // what stops the paint from sitting as a fixed screen-space pattern. the
    // angle keys on the world CELL, so a stroke turns with the geometry
    // instead of re-rolling per screen position
    var angle = (ditherHash(vec2<f32>(hashKey, 211.0 + seed), seed) - 0.5) * 6.2831853;
    if (lam1 > 0.000001) {
        angle = 0.5 * atan2(2.0 * tf.y, tf.x - tf.z) + 1.5707963;
    }
    angle = angle + (ditherHash(vec2<f32>(hashKey, 307.0 + seed), seed) - 0.5) * paint_params.wobble;

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
        let hr = ditherHash(vec2<f32>(hashKey, 541.0 + seed), seed) * 2.0 - 1.0;
        let hl = ditherHash(vec2<f32>(hashKey, 613.0 + seed), seed) * 2.0 - 1.0;
        let hs = ditherHash(vec2<f32>(hashKey, 727.0 + seed), seed) * 2.0 - 1.0;
        // select not sign: sign(0.0) is 0, which would let a rare slot land on
        // the guide colour exactly
        let dr = select(-1.0, 1.0, hr >= 0.0) * (0.6 + 0.4 * abs(hr));
        let dl = select(-1.0, 1.0, hl >= 0.0) * (0.6 + 0.4 * abs(hl));
        let ds = select(-1.0, 1.0, hs >= 0.0) * (0.6 + 0.4 * abs(hs));
        col = col * vec3<f32>(1.0 + cj * dr, 1.0 + cj * dl, 1.0 + cj * ds)
            + vec3<f32>(dr * cj * 0.25, dl * cj * 0.25, ds * cj * 0.25);
    }
    out.color = col;
    out.fade = fade;

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

    return vec4<f32>(in.color, cover * paint_params.alpha * in.fade);
}
