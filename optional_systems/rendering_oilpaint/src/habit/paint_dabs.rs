// The paint habit: every PaintOil form gets its mesh sampled into dabs
// this tick, written into this screen's SysRecordOilPaint for the dab
// pass. Painted forms push NO DrawCall — the dab pass is a whole separate
// pass (see the rendering component's call after draw_3d_features) — and
// because nothing enters SysRecordRendering, they also cast no shadow;
// the shadow hook for painted forms, if it ever lands, is a cast_shadow
// DrawCall pushed here with its color draw suppressed.
//
// Sampling is deterministic per (form, triangle, slot): the dabs never
// re-roll between frames, they shimmer-zero.

use curio_core::{Color, Ledger, Nerve, NetworkModes, Vector3};
use gameplay::{
    built_in::facet::transform::transform3d::Transform3D,
    context_3d::Context3D,
    traits::{facet_common::FacetCommon, habit::Habit, scope::Scope},
    traits_internal::world_context_common::ContextCommon,
};
use habit::habit;
use rendering_static::{PrimitiveResolution, Primitives};

use crate::{PaintOil, SysRecordOilPaint};
use camera::{camera_snapshot::CameraSnapshot, SysRecordCamera};
use crate::data::mesh_sampler::{sample_mesh_surface};

// ceiling on one screen's paint — area-based density normally keeps
// painted scenes well under this; beyond it the tail simply goes unpainted
const MAX_TOTAL_DABS: usize = 32768;

// one-shot demo authoring field (the sample curio predates this facet)
#[habit]
pub struct Instance {
    demo_done: bool,
}
// the macro derives Default — demo_done starts false, which is all the
// per-screen instance needs
impl Scope for Instance {
    fn is_enabled(&mut self, _ledger: &mut Ledger) -> bool {
        true
    }
    fn run_on_instance(&mut self, _ledger: &mut Ledger) -> Vec<NetworkModes> {
        NetworkModes::all()
    }
}
impl Habit for Instance {
    fn did_tick(&mut self, state: &mut Ledger, world: &mut Context3D, _: &mut Nerve) {
        // demo scene, once: the prebuilt sample curio authors no PaintOil,
        // so the first tick with nothing painted spawns a small painted
        // trio near where that curio's camera looks. A curio that authors
        // its own PaintOil forms never sees the demo.
        if !self.demo_done {
            self.demo_done = true;
            let mut authored = 0;
            world.edit::<&PaintOil>(|query| {
                for (_, paint) in query {
                    if paint.enabled {
                        authored += 1;
                    }
                }
            });
            if authored == 0 {
                // camera-placed — this habit also serves foreign curios
                // whose cameras sit nowhere near the origin
                let snap = &state.read::<SysRecordCamera>().cameras;
                spawn_demo_scene(world, snap);
            }
        }

        // the demo floor slab rides along under the walking camera in x/z
        // (its y stays baked at floor height) so the painted floor keeps
        // covering the view wherever the curio's camera roams — the phase-4
        // rigid pin makes its dabs follow the matrix for free
        let cam_pos = state.read::<SysRecordCamera>().cameras.position;
        let cam_x = cam_pos.x;
        let cam_z = cam_pos.z;

        state.write::<SysRecordOilPaint>(|x| {
            x.dabs.clear();
            x.models.clear();

            world.edit::<(&PaintOil, &Transform3D)>(|query| {
                for (_, (paint, transform)) in query {
                    // guard - disabled (same hierarchy check the static renderer uses)
                    if !paint.enabled || !paint.form().enabled_in_hierachy() {
                        continue;
                    }

                    // guard - no mesh
                    let Some(asset) = &paint.asset else {
                        continue;
                    };

                    let matrix = transform.get_matrix();
                    // stable per-form seed — the dabs never re-roll
                    let seed_base = (paint.form().instance_id() as u64) * 0x9E3779B97F4A7C15;

                    for m in &asset.mesh {
                        // the sampler returns this mesh's dabs — budget is
                        // what's left in this screen's cap
                        let budget = if MAX_TOTAL_DABS > x.dabs.len() { MAX_TOTAL_DABS - x.dabs.len() } else { 0 };
                        let painted = sample_mesh_surface(&m, paint.dab_size, paint.tint, seed_base, budget, matrix);
                        let painted_len = painted.len();
                        x.dabs.extend(painted);
                        // parallel matrices: phase 3 bakes them in the draw
                        // pass, phase 4 uploads them as the second instance
                        // attribute
                        for _ in 0..painted_len {
                            x.models.push(matrix);
                        }
                    }
                }
            });

            // ground follows the camera (x/z only)
            world.edit::<(&PaintOil, &mut Transform3D)>(|query| {
                for (_, (paint, transform)) in query {
                    if paint.form().name() == "oil_demo_ground" {
                        transform.position.x = cam_x;
                        transform.position.z = cam_z;
                    }
                }
            });
        });
    }
}

// the stand-in painted scene — procedural primitives need no asset names
// from the curio, and Transform3D places them where the sample camera
// looks. the phase-4 proof ran on hall-proto, whose camera sits far from
// the origin, so placement reads CameraSnapshot: ground under the eye,
// sphere and column a few strides ahead along the view axis. each form
// also gets a painterly TINT — the gray default lets every dab roll its
// own hue, which made the first demo look like confetti
pub fn spawn_demo_scene(world: &mut Context3D, cam: &CameraSnapshot) {
    let p = cam.position;
    let m = cam.calc_matrix();
    // row i of the rotation block is the camera basis axis i (view is
    // look_to_rh: rows are right, up, BACKWARD)
    let rx = m[0][0];
    let rz = m[2][0];
    let fx = -m[0][2];
    let fz = -m[2][2];
    let flen = (fx * fx + fz * fz).sqrt();
    let fx = if flen < 0.0001 { 0.0 } else { fx / flen };
    let fz = if flen < 0.0001 { 0.0 } else { fz / flen };
    let ground_y = p.y - 1.5; // eye height above the floor

    let ground = world
        .spawner("oil_demo_ground")
        .facet(
            PaintOil::builder()
                .asset(Some(Primitives::cube(PrimitiveResolution::Medium)))
                // dab_size is object-LOCAL — the sampler counts wants by
                // world area and the shader grows strokes by the mean
                // column length (~19 for this slab), so 0.016 lands at a
                // ~0.3 m world stroke across 32×24 m of floor
                .dab_size(0.016)
                .tint(Color::new_hex("8a7355"))
                .build(),
        )
        .spawn();
    let mut t = Transform3D::default();
    t.position = Vector3::new(p.x, ground_y, p.z);
    t.scale = Vector3::new(32.0, 0.06, 24.0);
    ground.add_facet(t);

    let ball = world
        .spawner("oil_demo_sphere")
        .facet(
            PaintOil::builder()
                .asset(Some(Primitives::sphere(PrimitiveResolution::Medium)))
                .dab_size(0.14)
                .tint(Color::new_hex("c46b2f"))
                .build(),
        )
        .spawn();
    let mut t = Transform3D::default();
    t.position = Vector3::new(p.x + fx * 4.0 + rx * 1.2, ground_y + 0.85, p.z + fz * 4.0 + rz * 1.2);
    t.scale = Vector3::new(1.7, 1.7, 1.7);
    ball.add_facet(t);

    let column = world
        .spawner("oil_demo_cylinder")
        .facet(
            PaintOil::builder()
                .asset(Some(Primitives::cylinder(PrimitiveResolution::Medium)))
                .dab_size(0.16)
                .tint(Color::new_hex("93552f"))
                .build(),
        )
        .spawn();
    let mut t = Transform3D::default();
    t.position = Vector3::new(p.x + fx * 2.6 - rx * 1.6, ground_y + 0.78, p.z + fz * 2.6 - rz * 1.6);
    t.scale = Vector3::new(0.7, 1.55, 0.7);
    column.add_facet(t);
}
