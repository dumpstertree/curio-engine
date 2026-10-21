//! Same-frame pull queries against the live ECS.
//!
//! The [`crate::SystemComponentDefaultPhysics`] plugin *reports* collisions through
//! `SysRecordCollision` at the end of the frame, so gameplay reading that record sees
//! collisions from one frame earlier. A [`PhysicsQuery`] instead builds its colliders
//! from the live world right now: colliders spawned, moved or destroyed earlier in the
//! same tick are visible to it. Use it when a reaction must happen in the frame the
//! touch happens (bounces, hitscan, line-of-sight); use the record for deferred or
//! networked consumers (score sensors, editor overlays).
//!
//! ```ignore
//! let hits = physics::PhysicsQuery::new(ctx).overlaps_form(&my_form);
//! ```
//!
//! Reentrancy rule: a query borrows the world immutably while building. Never call
//! [`PhysicsQuery::new`] from inside a `context.edit::<..>`/`form.edit_facet::<..>`
//! closure - move first, then query.

use collision::{CollisionPair, Collider, ColliderSnapshot};
use curio_core::Matrix4x4;
use gameplay::{
    built_in::facet::transform::transform3d::Transform3D,
    context_3d::Context3D,
    form::Form,
    traits::facet_common::FacetCommon,
    traits_internal::world_context_common::ContextCommon,
};

use crate::{overlaps_between, prepare_colliders, rays_against, PreparedCollider, RayHit, RayQuery};

/// The world's colliders, prepared from live facets, ready for immediate queries.
pub struct PhysicsQuery {
    prepared: Vec<PreparedCollider>,
}
impl PhysicsQuery {
    /// Snapshot every enabled collider in `context` using each form's *current*
    /// transform (not the world matrix cached at the last `did_tick`).
    pub fn new(context: &Context3D) -> PhysicsQuery {
        PhysicsQuery {
            prepared: collect_live(context),
        }
    }

    /// Number of non-degenerate colliders captured by this query.
    #[inline]
    pub fn collider_count(&self) -> usize {
        self.prepared.len()
    }

    /// Every filter-allowed overlapping pair in the world, contacts included
    /// (sensors report `contact: None`), like `SysRecordCollision.overlaps` but current.
    pub fn overlaps(&self) -> Vec<CollisionPair> {
        overlaps_between(&self.prepared)
    }

    /// Overlapping pairs involving the collider owned by `form`.
    pub fn overlaps_form(&self, form: &Form) -> Vec<CollisionPair> {
        self.overlaps_guid(form.instance_id())
    }

    /// Overlapping pairs involving the collider with this `instance_id()`.
    pub fn overlaps_guid(&self, guid: i32) -> Vec<CollisionPair> {
        self.overlaps().into_iter().filter(|pair| pair.involves(guid)).collect()
    }

    /// Closest hit against the live colliders; see [`crate::raycast_first`].
    pub fn raycast_first(&self, query: &RayQuery) -> Option<RayHit> {
        // raycast_all already returns hits sorted by distance
        self.raycast_all(query).into_iter().next()
    }

    /// Every hit against the live colliders sorted by distance; see [`crate::raycast_all`].
    pub fn raycast_all(&self, query: &RayQuery) -> Vec<RayHit> {
        rays_against(&self.prepared, query)
    }
}

fn collect_live(context: &Context3D) -> Vec<PreparedCollider> {
    let mut snapshots: Vec<ColliderSnapshot> = Vec::new();
    for collider in context.get::<Collider>() {
        let form = collider.form();
        if !form.enabled_in_hierachy() {
            continue;
        }
        let Some(matrix) = live_world_matrix(&form) else {
            continue;
        };
        snapshots.push(ColliderSnapshot {
            guid: form.instance_id(),
            matrix,
            shape: collider.shape.clone(),
            sensor: collider.sensor,
            filter_group: collider.filter_group,
            filter_mask: collider.filter_mask,
        });
    }
    prepare_colliders(&snapshots)
}

/// Compose the world matrix from the current local transforms along the parent chain,
/// mirroring what `update_transform3d` caches at `did_tick` but reading this frame's
/// values. Returns None if any ancestor lacks a `Transform3D`.
fn live_world_matrix(form: &Form) -> Option<Matrix4x4> {
    let mut chain: Vec<Matrix4x4> = Vec::new();
    let mut current = form.clone();
    loop {
        chain.push(current.get_facet::<Transform3D>()?.get_matrix());
        match current.parent() {
            Some(parent) => current = parent,
            None => break,
        }
    }
    let mut world_matrix = *chain.last()?;
    for local in chain.iter().rev().skip(1) {
        world_matrix = Matrix4x4::multiply(&world_matrix, local);
    }
    Some(world_matrix)
}
