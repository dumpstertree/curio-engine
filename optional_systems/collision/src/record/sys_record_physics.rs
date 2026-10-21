use curio_core::{FieldState, RecordOverride};
use record::record;

use crate::{CollisionPair, ColliderSnapshot};

/// Every collider in the game as of the current frame, written by the collector
/// habit in this crate and read by the physics plugin.
#[record(name = "Colliders", ownership = curio_core::RecordScope::Instance)]
pub struct SysRecordCollider {
    pub colliders: Vec<ColliderSnapshot>,
}

/// Collision results for the current frame, written by the physics plugin.
///
/// - `overlaps`: every pair of colliders intersecting right now
/// - `started`: pairs that started overlapping this frame (enter events)
/// - `ended`: pairs that stopped overlapping this frame (exit events)
#[record(name = "Collisions", ownership = curio_core::RecordScope::Instance)]
pub struct SysRecordCollision {
    pub overlaps: Vec<CollisionPair>,
    pub started: Vec<CollisionPair>,
    pub ended: Vec<CollisionPair>,
}

impl RecordOverride for SysRecordCollider {
    fn set_state(&mut self, _: &str, _: &str) {}
    fn get_state(&self) -> Vec<FieldState> {
        vec![FieldState::new("colliders", self.colliders.len().to_string())]
    }
}
impl RecordOverride for SysRecordCollision {
    fn set_state(&mut self, _: &str, _: &str) {}
    fn get_state(&self) -> Vec<FieldState> {
        vec![
            FieldState::new("overlaps", self.overlaps.len().to_string()),
            FieldState::new("started", self.started.len().to_string()),
            FieldState::new("ended", self.ended.len().to_string()),
        ]
    }
}
