use curio_core::{Ledger, Nerve, NetworkModes};
use gameplay::{
    built_in::facet::transform::transform3d::Transform3D,
    context_3d::Context3D,
    traits::{facet_common::FacetCommon, habit::Habit, scope::Scope},
    traits_internal::world_context_common::ContextCommon,
};
use habit::habit;

use crate::{Collider, ColliderSnapshot, SysRecordCollider};

/// Copies every enabled collider in the world, together with its world matrix,
/// into `SysRecordCollider` once per frame. The physics plugin picks the record
/// up later in the same frame (its plugin order runs after gameplay).
#[habit]
pub struct ColliderCollector {}
impl Scope for ColliderCollector {
    fn is_enabled(&mut self, _ledger: &mut Ledger) -> bool {
        true
    }
    fn run_on_instance(&mut self, _ledger: &mut Ledger) -> Vec<NetworkModes> {
        NetworkModes::all()
    }
}
impl Habit for ColliderCollector {
    fn did_tick(&mut self, state: &mut Ledger, world: &mut Context3D, _: &mut Nerve) {
        let colliders: std::cell::RefCell<Vec<ColliderSnapshot>> = std::cell::RefCell::new(Vec::new());
        world.edit::<(&Collider, &Transform3D)>(|q| {
            for (_entity, (collider, transform)) in q {
                if !collider.form().enabled_in_hierachy() {
                    continue;
                }
                colliders.borrow_mut().push(ColliderSnapshot {
                    guid: collider.form().instance_id(),
                    matrix: *transform.get_world_matrix(world),
                    shape: collider.shape.clone(),
                    sensor: collider.sensor,
                    filter_group: collider.filter_group,
                    filter_mask: collider.filter_mask,
                });
            }
        });
        state.write::<SysRecordCollider>(|x| {
            x.colliders = std::mem::take(&mut colliders.borrow_mut());
        });
    }
}
