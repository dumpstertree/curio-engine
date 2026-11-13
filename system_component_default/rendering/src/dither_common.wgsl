// === Shared dither helpers ===
// Prefix: dither_ so these can be safely concatenated in front of any pass's
// shader source (engine idiom: include_str! the pass file, prepend this file).
// All noise values are in [0, 1).

// hash-based value noise, close to a blue-noise look when tiled; seed shifts
// the lattice without changing the distribution
fn ditherHash(p: vec2<f32>, seed: f32) -> f32 {
    var v: vec2<u32> = vec2<u32>(p * vec2<f32>(1867.47, 1899.51))
        + vec2<u32>(u32(seed * 32767.0) + 11u, 2167263077u);
    var v3 = v * v * v + vec2<u32>(1268297u, 2971830941u); // u32-wrapped constants
    return f32((v3.x ^ v3.y) % 65536u) / 65536.0;
}

// interleaved gradient noise: cheap ordered-ish blue noise, no seed
fn ditherInterleavedGradient(p: vec2<f32>) -> f32 {
    let ip = vec2<i32>(floor(p));
    let m = 0.06711056 * f32(ip.x) + 0.00583715 * f32(ip.y);
    return m - floor(m);
}

// pick the noise type: mode < 0.5 selects the interleaved gradient,
// anything else the hashed blue noise
fn ditherNoise(p: vec2<f32>, mode: f32, seed: f32) -> f32 {
    if (mode < 0.5) {
        return ditherInterleavedGradient(p);
    }
    return ditherHash(p, seed);
}

// stochastic weighted picks (naga's WGSL frontend has no user-defined
// generics, so one variant per type)
fn ditherPickF32(f1: f32, f2: f32, w: f32, t: f32) -> f32 {
    return select(f1, f2, t > w);
}

fn ditherPickVec3(f1: vec3<f32>, f2: vec3<f32>, w: f32, t: f32) -> vec3<f32> {
    return select(f1, f2, t > w);
}
