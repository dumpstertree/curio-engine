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

use curio_core::{Ledger, Nerve, NetworkModes, Vector3};
use gameplay::{
    built_in::facet::transform::transform3d::Transform3D,
    context_3d::Context3D,
    traits::{facet_common::FacetCommon, habit::Habit, scope::Scope},
    traits_internal::world_context_common::ContextCommon,
};
use habit::habit;
use rendering_static::{PrimitiveResolution, Primitives};

use crate::PaintOil;
use crate::SysRecordOilPaint;
use crate::data::mesh_sampler::{sample_mesh_surface};

// ceiling on one screen's paint — area-based density normally keeps
// painted scenes well under this; beyond it the tail simply goes unpainted
const MAX_TOTAL_DABS: usize = 24000;

// one-shot demo authoring field (the sample curio predates this facet)
#[habit]
pub struct Instance {
    demo_done: bool,
}
impl Instance {
    pub fn new() -> Box<Instance> {
        Box::new(Instance {
            demo_done: false,
        })
    }
}
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
                spawn_demo_scene(world);
            }
        }

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
                        let painted = sample_mesh_surface(&m, paint.dab_size, paint.tint, seed_base, budget);
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
        });
    }
}

// the stand-in painted scene — procedural primitives need no asset names
// from the curio, and Transform3D places them where the sample camera
// looks (the same neighborhood as the phase-1 proof patch)
pub fn spawn_demo_scene(world: &mut Context3D) {
    let ground = world
        .spawner("oil_demo_ground")
        .facet(
            PaintOil::builder()
                .asset(Some(Primitives::cube(PrimitiveResolution::Medium)))
                // dab_size is object-LOCAL — the draw pass bakes size by
                // the matrix's mean column length (~9 for this slab), so
                // 0.035 lands at a ~0.3 m world stroke
                .dab_size(0.035)
                .build(),
        )
        .spawn();
    let mut t = Transform3D::default();
    t.position = Vector3::new(0.0, -0.09, 0.0);
    t.scale = Vector3::new(16.0, 0.06, 11.0);
    ground.add_facet(t);

    let ball = world
        .spawner("oil_demo_sphere")
        .facet(
            PaintOil::builder()
                .asset(Some(Primitives::sphere(PrimitiveResolution::Medium)))
                .dab_size(0.14)
                .build(),
        )
        .spawn();
    let mut t = Transform3D::default();
    t.position = Vector3::new(0.8, 1.05, -2.4);
    t.scale = Vector3::new(1.7, 1.7, 1.7);
    ball.add_facet(t);

    let column = world
        .spawner("oil_demo_cylinder")
        .facet(
            PaintOil::builder()
                .asset(Some(Primitives::cylinder(PrimitiveResolution::Medium)))
                .dab_size(0.16)
                .build(),
        )
        .spawn();
    let mut t = Transform3D::default();
    t.position = Vector3::new(-1.9, 0.78, -1.5);
    t.scale = Vector3::new(0.7, 1.55, 0.7);
    column.add_facet(t);
}
