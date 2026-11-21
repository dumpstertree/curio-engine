// Phase 3: mesh surface sampler. Triangles become dabs IN OBJECT-LOCAL
// space — the habit multiplies the form's matrix in, and phase 4 hands that
// matrix to the vertex shader instead so painted forms can move without
// re-sampling. Density is area-based (one dab per brush footprint), stroke
// flow follows the face tangent, and every random is hashed off
// (form, triangle, slot) so the paint is stable across frames —
// re-seeding per frame would make the whole form shimmer.

use curio_core::Color;
use ext_rendering::Mesh;

use crate::Dab;
use crate::feature::render_feature_draw_dabs::RenderFeatureDrawDabs;

use curio_core::Matrix4x4;

// one dab covers a brush footprint ≈ 0.6 × dab_size² of surface — strokes
// are anisotropic (across ≈ 0.3 × along, so real coverage ≈ 0.4 dab²) and
// 0.6 lands ~70 % dressed skin with a little overlap, the way patch's
// strokes lay themselves into paint; the old 2.1 left the floor 80 % bare.
// the per-triangle clamp only stops one fat triangle in a sparse mesh
// from hogging the budget — the global MAX_TOTAL_DABS is the real ceiling,
// so it sits far above any single face's want
const MAX_PER_TRIANGLE: usize = 16384;

// budget caps the dabs this call may produce. density is WORLD-aware: the
// matrix maps local area to world area, and the vertex shader grows strokes
// by the mean column length, so want = world face area ÷ world footprint —
// a 30 m floor slab gets thousands of strokes per face while its 6 cm
// sides get a handful, which local-only counting got badly wrong
pub fn sample_mesh_surface(mesh: &Mesh, dab_size: f32, tint: Color, seed_base: u64, budget: usize, matrix: Matrix4x4) -> Vec<Dab> {
    let mut out = Vec::<Dab>::new();
    let m4 = &matrix.model;
    // mean column length — the same stroke scale the vertex shader applies
    let s0 = (m4[0][0] * m4[0][0] + m4[1][0] * m4[1][0] + m4[2][0] * m4[2][0]).sqrt();
    let s1 = (m4[0][1] * m4[0][1] + m4[1][1] * m4[1][1] + m4[2][1] * m4[2][1]).sqrt();
    let s2 = (m4[0][2] * m4[0][2] + m4[1][2] * m4[1][2] + m4[2][2] * m4[2][2]).sqrt();
    let stroke_scale = if (s0 + s1 + s2) < 0.0001 { 1.0 } else { (s0 + s1 + s2) / 3.0 };
    let brush = if dab_size < 0.01 { 0.01 } else { dab_size };
    let footprint = brush * brush * stroke_scale * stroke_scale * 0.6;

    // tint acts as a hue bias: saturated tints pin the palette slot, a
    // white/gray tint lets every dab draw its own
    let (hue_base, sat) = hue_of(tint);

    let tri_count = mesh.indicies.len() / 3;
    let mut tri = 0;
    while tri < tri_count && out.len() < budget {
        let ia = mesh.indicies[tri * 3] as usize;
        let ib = mesh.indicies[tri * 3 + 1] as usize;
        let ic = mesh.indicies[tri * 3 + 2] as usize;
        tri += 1;
        if ia >= mesh.verticies.len() || ib >= mesh.verticies.len() || ic >= mesh.verticies.len() {
            continue;
        }

        let a = mesh.verticies[ia];
        let b = mesh.verticies[ib];
        let c = mesh.verticies[ic];
        let ax = a.position[0];
        let ay = a.position[1];
        let az = a.position[2];
        let bx = b.position[0];
        let by = b.position[1];
        let bz = b.position[2];
        let cx = c.position[0];
        let cy = c.position[1];
        let cz = c.position[2];

        // face frame: e1,e2 edges; n = e1×e2 (length = 2×area)
        let e1x = bx - ax;
        let e1y = by - ay;
        let e1z = bz - az;
        let e2x = cx - ax;
        let e2y = cy - ay;
        let e2z = cz - az;
        let nx = e1y * e2z - e1z * e2y;
        let ny = e1z * e2x - e1x * e2z;
        let nz = e1x * e2y - e1y * e2x;
        let nlen = (nx * nx + ny * ny + nz * nz).sqrt();
        if nlen < 0.000001 {
            continue; // degenerate — no paint on a pinched sliver
        }
        // world-aware density + downward skip: world-space area drives how
        // many strokes the face can hold; the orientation is the same
        // cross product pushed through the matrix's 3x3 block
        let e1wx = m4[0][0] * e1x + m4[1][0] * e1y + m4[2][0] * e1z;
        let e1wy = m4[0][1] * e1x + m4[1][1] * e1y + m4[2][1] * e1z;
        let e1wz = m4[0][2] * e1x + m4[1][2] * e1y + m4[2][2] * e1z;
        let e2wx = m4[0][0] * e2x + m4[1][0] * e2y + m4[2][0] * e2z;
        let e2wy = m4[0][1] * e2x + m4[1][1] * e2y + m4[2][1] * e2z;
        let e2wz = m4[0][2] * e2x + m4[1][2] * e2y + m4[2][2] * e2z;
        let nwx = e1wy * e2wz - e1wz * e2wy;
        let nwy = e1wz * e2wx - e1wx * e2wz;
        let nwz = e1wx * e2wy - e1wy * e2wx;
        let nwlen = (nwx * nwx + nwy * nwy + nwz * nwz).sqrt();
        if nwlen < 0.000001 {
            continue;
        }
        if nwy / nwlen < -0.4 {
            continue; // surface faces the ground — paint there reads mud
        }
        let area = nwlen * 0.5;
        let nnx = nx / nlen;
        let nny = ny / nlen;
        let nnz = nz / nlen;

        // base flow = the world axis LEAST aligned with n, crossed into the
        // face plane: ground strokes run one way, walls run their contour
        let ref_is_x = nnx.abs() <= nny.abs() && nnx.abs() <= nnz.abs();
        let ref_is_y = !ref_is_x && nny.abs() <= nnz.abs();
        let (tx, ty, tz) = if ref_is_x {
            (0.0, nnz, -nny) // n × X
        } else if ref_is_y {
            (-nnz, 0.0, nnx) // n × Y
        } else {
            (nny, -nnx, 0.0) // n × Z
        };
        let tlen = (tx * tx + ty * ty + tz * tz).sqrt();
        let (tx, ty, tz) = if tlen < 0.0001 {
            (1.0, 0.0, 0.0)
        } else {
            (tx / tlen, ty / tlen, tz / tlen)
        };
        // n × t — the in-plane perpendicular for the Rodrigues swirl
        let bx_ = nny * tz - nnz * ty;
        let by_ = nnz * tx - nnx * tz;
        let bz_ = nnx * ty - nny * tx;

        let tri_i = tri as u64;
        let want = ((area / footprint) + 0.5) as usize;
        let n_want = if want < 1 { 1 } else if want > MAX_PER_TRIANGLE { MAX_PER_TRIANGLE } else { want };
        for k in 0..n_want {
            if out.len() >= budget {
                break;
            }
            // one hash chain per (form, triangle, slot) — the stable seed
            let mix = seed_base + tri_i * 0x1000193 + k as u64;
            let r0 = RenderFeatureDrawDabs::hash01(mix, 11);
            let r1 = RenderFeatureDrawDabs::hash01(mix, 23);
            let r2 = RenderFeatureDrawDabs::hash01(mix, 37);
            let r3 = RenderFeatureDrawDabs::hash01(mix, 53);
            let r4 = RenderFeatureDrawDabs::hash01(mix, 71);

            // uniform barycentric pick (sqrt keeps area even)
            let s = r0.sqrt();
            let w1 = 1.0 - s;
            let w2 = s * (1.0 - r1);
            let w3 = s * r1;
            let px = w1 * ax + w2 * bx + w3 * cx;
            let py = w1 * ay + w2 * by + w3 * cy;
            let pz = w1 * az + w2 * bz + w3 * cz;

            // thick paint sits a hair proud of the skin it replaces
            let lift = 0.005 + r2 * 0.007;
            let pinx = px + nnx * lift;
            let piny = py + nny * lift;
            let pink = pz + nnz * lift;

            // stroke runs along the face tangent, swirled ±30° about n
            let ang = (r3 - 0.5) * 1.05;
            let ca = ang.cos();
            let sa = ang.sin();
            let mut dx = tx * ca + bx_ * sa;
            let mut dy = ty * ca + by_ * sa;
            let mut dz = tz * ca + bz_ * sa;
            let dlen = (dx * dx + dy * dy + dz * dz).sqrt();
            if dlen > 0.0001 {
                dx = dx / dlen;
                dy = dy / dlen;
                dz = dz / dlen;
            }

            let along = brush * (0.8 + r2 * 0.55);
            let across = along * (0.26 + r3 * 0.2);

            // saturated tint pins the palette slot near the tint's hue;
            // gray lets the dab draw its own
            let hue = if sat < 0.15 { r4 } else { hue_base + (r3 - 0.5) * 0.1 };

            out.push(
                Dab {
                    pin: [pinx, piny, pink],
                    dir: [dx, dy, dz],
                    size: [across, along],
                    seed: [hue, r1, r2, r4],
                },
            );
        }
    }

    return out;
}

// rgb → (hue 0-1, saturation 0-1) — same hue sectors the shader's ramp
// draws, so a tint authored as a composition string lands on the right
// palette slot
pub fn hue_of(color: Color) -> (f32, f32) {
    let r = color.as_r_01();
    let g = color.as_g_01();
    let b = color.as_b_01();
    let max = if r > g { if r > b { r } else { b } } else { if g > b { g } else { b } };
    let min = if r < g { if r < b { r } else { b } } else { if g < b { g } else { b } };
    let sat = if max < 0.001 { 0.0 } else { (max - min) / max };
    let d = max - min;
    if d < 0.001 {
        return (0.0, sat);
    }
    let mut hue = if max == r {
        (g - b) / d
    } else if max == g {
        2.0 + (b - r) / d
    } else {
        4.0 + (r - g) / d
    };
    hue = hue / 6.0;
    if hue < 0.0 {
        hue += 1.0;
    }
    return (hue, sat);
}
