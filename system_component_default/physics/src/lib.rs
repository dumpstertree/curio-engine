//! Physics plugin: reads `SysRecordCollider` (written by the collision crate's
//! collector habit) each tick, detects overlaps/contacts with rapier's geometry
//! layer (parry), and writes the results to `SysRecordCollision`.
//!
//! There is no rigid body simulation: transforms drive colliders, the plugin
//! only answers "what is touching what" plus world raycasts through the
//! [`raycast_first`] / [`raycast_all`] statics.

use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

use collision::{CollisionPair, ColliderShapeDef, ColliderSnapshot, Contact, SysRecordCollider, SysRecordCollision};
use curio_core::{Ledger, Matrix4x4, Nerve, PluginCommon, Vector3};
use rapier3d::{
    na::{self, Isometry3},
    parry::{
        query::{contact, intersection_test, Ray},
        shape::{Ball, Capsule, Cuboid, Shape},
    },
};

pub struct SystemComponentDefaultPhysics {
    // per instance (indexed by ledger index): pairs overlapping last frame,
    // keyed by (smaller guid, larger guid), used to compute enter/exit events
    previous_pairs: Vec<HashMap<(i32, i32), CollisionPair>>,
}
impl SystemComponentDefaultPhysics {
    pub fn new() -> Box<SystemComponentDefaultPhysics> {
        Box::new(SystemComponentDefaultPhysics { previous_pairs: vec![] })
    }
}
impl Default for SystemComponentDefaultPhysics {
    fn default() -> Self {
        SystemComponentDefaultPhysics { previous_pairs: vec![] }
    }
}
impl PluginCommon for SystemComponentDefaultPhysics {
    fn name(&self) -> String {
        "Physics".to_owned()
    }
    // after gameplay (5000) so colliders are collected, before rendering (9000)
    fn order(&self) -> i32 {
        6000
    }

    fn tick(&mut self, ledgers: &mut Vec<Ledger>, _event_queue: &mut Vec<Nerve>) {
        // rebuild the per-instance raycast registry from scratch this frame
        let mut registry = raycast_registry().lock().unwrap_or_else(|p| p.into_inner());
        registry.clear();

        for (instance_index, ledger) in ledgers.iter_mut().enumerate() {
            if self.previous_pairs.len() <= instance_index {
                self.previous_pairs.push(HashMap::new());
            }

            let snapshots = ledger.read::<SysRecordCollider>().colliders.clone();

            let prepared = prepare_colliders(&snapshots);
            let overlaps = overlaps_between(&prepared);

            // enter/exit events by diffing against last frame's pairs
            let previous = &mut self.previous_pairs[instance_index];
            let (started, ended) = diff_pairs(&overlaps, previous);
            *previous = pair_map(&overlaps);

            ledger.write::<SysRecordCollision>(|x| {
                x.overlaps = overlaps.clone();
                x.started = started.clone();
                x.ended = ended.clone();
            });

            // hand this instance's colliders to the raycast statics
            registry.push(prepared.clone());
        }
    }
}

/// Build rapier shapes from collider snapshots; degenerate colliders are dropped.
fn prepare_colliders(snapshots: &[ColliderSnapshot]) -> Vec<PreparedCollider> {
    let mut prepared: Vec<PreparedCollider> = Vec::with_capacity(snapshots.len());
    for snapshot in snapshots {
        if let Some(collider) = PreparedCollider::new(snapshot.clone()) {
            prepared.push(collider);
        }
    }
    prepared
}

/// Narrow phase: test every filter-allowed pair of colliders.
fn overlaps_between(prepared: &[PreparedCollider]) -> Vec<CollisionPair> {
    let mut overlaps: Vec<CollisionPair> = Vec::new();
    for i in 0..prepared.len() {
        for j in (i + 1)..prepared.len() {
            let a = &prepared[i];
            let b = &prepared[j];
            if a.guid == b.guid {
                continue;
            }
            if !a.snapshot.filter_allows(&b.snapshot) {
                continue;
            }

            let contact = if a.sensor || b.sensor {
                // sensors overlap but never report contact data
                let Ok(intersecting) = intersection_test(&a.isometry, a.shape.as_shape(), &b.isometry, b.shape.as_shape()) else {
                    continue;
                };
                if !intersecting {
                    continue;
                }
                None
            } else {
                let Ok(Some(c)) = contact(&a.isometry, a.shape.as_shape(), &b.isometry, b.shape.as_shape(), 0.0) else {
                    continue;
                };
                Some(Contact {
                    point: Vector3::new(
                        (c.point1.x + c.point2.x) * 0.5,
                        (c.point1.y + c.point2.y) * 0.5,
                        (c.point1.z + c.point2.z) * 0.5,
                    ),
                    normal_a: Vector3::new(c.normal1.x, c.normal1.y, c.normal1.z),
                    normal_b: Vector3::new(c.normal2.x, c.normal2.y, c.normal2.z),
                })
            };

            overlaps.push(CollisionPair {
                collider_a: a.snapshot.clone(),
                collider_b: b.snapshot.clone(),
                contact,
            });
        }
    }
    overlaps
}

fn pair_map(overlaps: &[CollisionPair]) -> HashMap<(i32, i32), CollisionPair> {
    let mut current: HashMap<(i32, i32), CollisionPair> = HashMap::with_capacity(overlaps.len());
    for pair in overlaps {
        current.insert(pair_key(&pair.collider_a.guid, &pair.collider_b.guid), pair.clone());
    }
    current
}

/// Pairs that started overlapping (in `overlaps` but not `previous`) and pairs
/// that ended (in `previous` but not `overlaps`).
fn diff_pairs(overlaps: &[CollisionPair], previous: &HashMap<(i32, i32), CollisionPair>) -> (Vec<CollisionPair>, Vec<CollisionPair>) {
    let current = pair_map(overlaps);
    let started: Vec<CollisionPair> = overlaps
        .iter()
        .filter(|pair| !previous.contains_key(&pair_key(&pair.collider_a.guid, &pair.collider_b.guid)))
        .cloned()
        .collect();
    let ended: Vec<CollisionPair> = previous
        .iter()
        .filter(|(key, _)| !current.contains_key(key))
        .map(|(_, pair)| pair.clone())
        .collect();
    (started, ended)
}

fn pair_key(a: &i32, b: &i32) -> (i32, i32) {
    if a <= b {
        (*a, *b)
    } else {
        (*b, *a)
    }
}

// ────────────────────────────── shapes ──────────────────────────────

#[derive(Clone)]
enum PreparedShape {
    Box(Cuboid),
    Sphere(Ball),
    Capsule(Capsule),
}
impl PreparedShape {
    fn as_shape(&self) -> &dyn Shape {
        match self {
            PreparedShape::Box(s) => s,
            PreparedShape::Sphere(s) => s,
            PreparedShape::Capsule(s) => s,
        }
    }
    /// Build a rapier shape from a collider def, baking the world scale into the
    /// shape (rapier isometries carry no scale). Returns `None` for degenerate
    /// (zero/negative) dimensions.
    fn from_def(def: &ColliderShapeDef, scale: Vector3) -> Option<PreparedShape> {
        let scale_positive = scale.x > 0.0 && scale.y > 0.0 && scale.z > 0.0;
        if !scale_positive {
            return None;
        }
        match def {
            ColliderShapeDef::Box { size } => {
                let half = Vector3::new(size.x * scale.x * 0.5, size.y * scale.y * 0.5, size.z * scale.z * 0.5);
                if half.x <= 0.0 || half.y <= 0.0 || half.z <= 0.0 {
                    return None;
                }
                Some(PreparedShape::Box(Cuboid::new(na::Vector3::new(half.x, half.y, half.z))))
            }
            ColliderShapeDef::Sphere { diameter } => {
                // non uniform scale is approximated with the average scale
                let average_scale = (scale.x + scale.y + scale.z) / 3.0;
                let radius = diameter * 0.5 * average_scale;
                if radius <= 0.0 {
                    return None;
                }
                Some(PreparedShape::Sphere(Ball::new(radius)))
            }
            ColliderShapeDef::Capsule { radius, height } => {
                let scaled_radius = radius * scale.x.max(scale.z);
                // def height is the full height, rapier's is the cylindrical section only
                let half_height = (height * scale.y * 0.5 - scaled_radius).max(0.0);
                if scaled_radius <= 0.0 {
                    return None;
                }
                Some(PreparedShape::Capsule(Capsule::new_y(half_height, scaled_radius)))
            }
        }
    }
}

/// A collider prepared for rapier queries: shape + world pose.
#[derive(Clone)]
struct PreparedCollider {
    guid: i32,
    sensor: bool,
    snapshot: ColliderSnapshot,
    isometry: Isometry3<f32>,
    shape: PreparedShape,
}
impl PreparedCollider {
    fn new(snapshot: ColliderSnapshot) -> Option<PreparedCollider> {
        let scale = world_scale(&snapshot.matrix);
        let shape = PreparedShape::from_def(&snapshot.shape, scale)?;
        let position = snapshot.matrix.extract_position();
        let rotation = snapshot.matrix.extract_rotation();
        let rotation = na::Unit::new_normalize(na::Quaternion::new(rotation.w, rotation.x, rotation.y, rotation.z));
        Some(PreparedCollider {
            guid: snapshot.guid,
            sensor: snapshot.sensor,
            isometry: Isometry3::from_parts(
                na::Vector3::new(position.x, position.y, position.z).into(),
                rotation,
            ),
            shape,
            snapshot,
        })
    }
}

/// The engine stores scale in the row vectors of the (row major) matrix.
fn world_scale(matrix: &Matrix4x4) -> Vector3 {
    let row_length = |row: usize| {
        let x = matrix.model[row][0];
        let y = matrix.model[row][1];
        let z = matrix.model[row][2];
        (x * x + y * y + z * z).sqrt()
    };
    Vector3::new(row_length(0), row_length(1), row_length(2))
}

// ────────────────────────────── raycast ──────────────────────────────

static RAYCAST_REGISTRY: OnceLock<Mutex<Vec<Vec<PreparedCollider>>>> = OnceLock::new();
fn raycast_registry() -> &'static Mutex<Vec<Vec<PreparedCollider>>> {
    RAYCAST_REGISTRY.get_or_init(|| Mutex::new(Vec::new()))
}

/// The result of a raycast hitting one collider.
#[derive(Clone, Copy, Debug)]
pub struct RayHit {
    /// `instance_id()` of the form that owns the hit collider
    pub guid: i32,
    /// whether the hit collider was a sensor
    pub hit_sensor: bool,
    /// distance along the query direction from the origin to the hit
    pub distance: f32,
    pub point: Vector3,
    /// surface normal of the hit collider at the hit point
    pub normal: Vector3,
}

/// A ray used by [`raycast_first`] / [`raycast_all`].
#[derive(Clone, Debug)]
pub struct RayQuery {
    origin: Vector3,
    direction: Vector3,
    max_distance: f32,
    include_sensors: bool,
    solid: bool,
    ignore_guids: Vec<i32>,
}
impl RayQuery {
    /// `direction` does not need to be normalized; its length is preserved in
    /// reported distances only via `max_distance` (distances are measured in
    /// world units along the normalized direction).
    pub fn new(origin: Vector3, direction: Vector3) -> RayQuery {
        RayQuery {
            origin,
            direction,
            max_distance: f32::MAX,
            include_sensors: false,
            solid: true,
            ignore_guids: vec![],
        }
    }
    pub fn max_distance(mut self, max_distance: f32) -> Self {
        self.max_distance = max_distance;
        self
    }
    /// also hit sensor colliders (default false)
    pub fn include_sensors(mut self, include_sensors: bool) -> Self {
        self.include_sensors = include_sensors;
        self
    }
    /// a ray starting inside a collider counts as a hit at distance 0 (default true)
    pub fn solid(mut self, solid: bool) -> Self {
        self.solid = solid;
        self
    }
    /// skip colliders owned by the form with this `instance_id()` (e.g. self)
    pub fn ignore(mut self, guid: i32) -> Self {
        self.ignore_guids.push(guid);
        self
    }
}

/// Cast a ray and return the closest hit. `instance` is the ledger/player index
/// (0 for single player). Reads the colliders the physics plugin prepared during
/// the last tick.
pub fn raycast_first(instance: usize, query: &RayQuery) -> Option<RayHit> {
    // raycast_all already returns hits sorted by distance
    raycast_all(instance, query).into_iter().next()
}

/// Cast a ray and return every hit sorted by distance. `instance` is the
/// ledger/player index (0 for single player).
pub fn raycast_all(instance: usize, query: &RayQuery) -> Vec<RayHit> {
    let registry = raycast_registry().lock().unwrap_or_else(|p| p.into_inner());
    let Some(colliders) = registry.get(instance) else {
        return vec![];
    };
    rays_against(colliders, query)
}

/// Cast against an explicit set of colliders (used by [`raycast_all`] and tests).
fn rays_against(colliders: &[PreparedCollider], query: &RayQuery) -> Vec<RayHit> {
    let direction = normalize(query.direction);
    if length(query.direction) <= f32::EPSILON || query.max_distance <= 0.0 {
        return vec![];
    }

    let origin = na::Point3::new(query.origin.x, query.origin.y, query.origin.z);
    let dir = na::Vector3::new(direction.x, direction.y, direction.z);

    let mut hits = Vec::new();
    for collider in colliders {
        if collider.sensor && !query.include_sensors {
            continue;
        }
        if query.ignore_guids.contains(&collider.guid) {
            continue;
        }
        // move the ray into the collider's local space
        let local_origin = collider.isometry.inverse_transform_point(&origin);
        let local_dir = collider.isometry.inverse_transform_vector(&dir);
        let ray = Ray::new(local_origin, local_dir);

        if let Some(hit) = collider.shape.as_shape().cast_local_ray_and_get_normal(&ray, query.max_distance, query.solid) {
            let normal = collider.isometry.transform_vector(&hit.normal);
            hits.push(RayHit {
                guid: collider.guid,
                hit_sensor: collider.sensor,
                distance: hit.time_of_impact,
                point: Vector3::new(
                    query.origin.x + direction.x * hit.time_of_impact,
                    query.origin.y + direction.y * hit.time_of_impact,
                    query.origin.z + direction.z * hit.time_of_impact,
                ),
                normal: Vector3::new(normal.x, normal.y, normal.z),
            });
        }
    }
    hits.sort_by(|a, b| a.distance.total_cmp(&b.distance));
    hits
}

fn normalize(v: Vector3) -> Vector3 {
    let len = length(v);
    if len <= f32::EPSILON {
        return Vector3::new(0.0, 0.0, 0.0);
    }
    Vector3::new(v.x / len, v.y / len, v.z / len)
}
fn length(v: Vector3) -> f32 {
    (v.x * v.x + v.y * v.y + v.z * v.z).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use collision::ColliderShapeDef;
    use curio_core::Quaternion;

    fn snapshot(guid: i32, position: Vector3, shape: ColliderShapeDef) -> ColliderSnapshot {
        snapshot_with(guid, position, shape, false, 0, 0)
    }
    fn snapshot_with(
        guid: i32,
        position: Vector3,
        shape: ColliderShapeDef,
        sensor: bool,
        filter_group: u32,
        filter_mask: u32,
    ) -> ColliderSnapshot {
        ColliderSnapshot {
            guid,
            matrix: Matrix4x4::new(position, Quaternion::identity(), Vector3::one()),
            shape,
            sensor,
            filter_group,
            filter_mask,
        }
    }
    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < 0.001
    }

    #[test]
    fn overlapping_boxes_are_detected_with_contact() {
        let snapshots = vec![
            snapshot(1, Vector3::new(0.0, 0.0, 0.0), ColliderShapeDef::Box { size: Vector3::new(1.0, 1.0, 1.0) }),
            snapshot(2, Vector3::new(0.5, 0.0, 0.0), ColliderShapeDef::Box { size: Vector3::new(1.0, 1.0, 1.0) }),
        ];
        let prepared = prepare_colliders(&snapshots);
        let overlaps = overlaps_between(&prepared);
        assert_eq!(overlaps.len(), 1);
        assert!(overlaps[0].involves(1) && overlaps[0].involves(2));
        let contact = overlaps[0].contact.expect("solid pair should have a contact");
        // box a faces -x towards b... contact sits between them, normals along x
        assert!(approx(contact.normal_a.x.abs(), 1.0));
        assert!(contact.point.x > 0.0 && contact.point.x < 1.0);
    }

    #[test]
    fn separated_boxes_are_not_overlapping() {
        let snapshots = vec![
            snapshot(1, Vector3::new(0.0, 0.0, 0.0), ColliderShapeDef::Box { size: Vector3::new(1.0, 1.0, 1.0) }),
            snapshot(2, Vector3::new(3.0, 0.0, 0.0), ColliderShapeDef::Box { size: Vector3::new(1.0, 1.0, 1.0) }),
        ];
        let prepared = prepare_colliders(&snapshots);
        assert!(overlaps_between(&prepared).is_empty());
    }

    #[test]
    fn sphere_box_and_capsule_are_supported() {
        let snapshots = vec![
            // box at origin, sphere reaching into it, capsule next to the sphere
            snapshot(1, Vector3::new(0.0, 0.0, 0.0), ColliderShapeDef::Box { size: Vector3::new(1.0, 1.0, 1.0) }),
            snapshot(2, Vector3::new(0.8, 0.0, 0.0), ColliderShapeDef::Sphere { diameter: 1.0 }),
            snapshot(3, Vector3::new(1.5, 0.0, 0.0), ColliderShapeDef::Capsule { radius: 0.4, height: 2.0 }),
        ];
        let prepared = prepare_colliders(&snapshots);
        let overlaps = overlaps_between(&prepared);
        // box<->sphere (0.8 < 0.5 + 0.5) and sphere<->capsule (0.7 < 0.5 + 0.4)
        assert_eq!(overlaps.len(), 2);
        assert!(overlaps.iter().any(|p| p.involves(1) && p.involves(2)));
        assert!(overlaps.iter().any(|p| p.involves(2) && p.involves(3)));
    }

    #[test]
    fn sensors_overlap_without_contact_data() {
        let snapshots = vec![
            snapshot(1, Vector3::new(0.0, 0.0, 0.0), ColliderShapeDef::Box { size: Vector3::new(1.0, 1.0, 1.0) }),
            snapshot_with(2, Vector3::new(0.5, 0.0, 0.0), ColliderShapeDef::Box { size: Vector3::new(1.0, 1.0, 1.0) }, true, 0, 0),
        ];
        let prepared = prepare_colliders(&snapshots);
        let overlaps = overlaps_between(&prepared);
        assert_eq!(overlaps.len(), 1);
        assert!(overlaps[0].contact.is_none());
    }

    #[test]
    fn filters_block_mismatched_groups() {
        // a only interacts with group 1, b is group 2 and only interacts with group 2
        let snapshots = vec![
            snapshot_with(1, Vector3::new(0.0, 0.0, 0.0), ColliderShapeDef::Box { size: Vector3::new(1.0, 1.0, 1.0) }, false, 1, 1),
            snapshot_with(2, Vector3::new(0.5, 0.0, 0.0), ColliderShapeDef::Box { size: Vector3::new(1.0, 1.0, 1.0) }, false, 2, 2),
        ];
        let prepared = prepare_colliders(&snapshots);
        assert!(overlaps_between(&prepared).is_empty());
    }

    #[test]
    fn started_and_ended_events_across_frames() {
        let touching = vec![
            snapshot(1, Vector3::new(0.0, 0.0, 0.0), ColliderShapeDef::Box { size: Vector3::new(1.0, 1.0, 1.0) }),
            snapshot(2, Vector3::new(0.5, 0.0, 0.0), ColliderShapeDef::Box { size: Vector3::new(1.0, 1.0, 1.0) }),
        ];
        let previous: HashMap<(i32, i32), CollisionPair> = HashMap::new();

        // frame 1: pair starts
        let overlaps = overlaps_between(&prepare_colliders(&touching));
        let (started, ended) = diff_pairs(&overlaps, &previous);
        assert_eq!((started.len(), ended.len()), (1, 0));
        let previous = pair_map(&overlaps);

        // frame 2: still touching, no events
        let (started, ended) = diff_pairs(&overlaps, &previous);
        assert_eq!((started.len(), ended.len()), (0, 0));

        // frame 3: separated, pair ends
        let mut moved = touching.clone();
        moved[1].matrix = Matrix4x4::new(Vector3::new(3.0, 0.0, 0.0), Quaternion::identity(), Vector3::one());
        let overlaps = overlaps_between(&prepare_colliders(&moved));
        let (started, ended) = diff_pairs(&overlaps, &previous);
        assert_eq!((started.len(), ended.len()), (0, 1));
        assert_eq!(ended[0].other_guid(1), Some(2));
    }

    #[test]
    fn ray_hits_sphere_at_expected_distance_and_normal() {
        let snapshots = vec![snapshot(7, Vector3::new(0.0, 0.0, -5.0), ColliderShapeDef::Sphere { diameter: 2.0 })];
        let prepared = prepare_colliders(&snapshots);
        let query = RayQuery::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, -1.0));
        let hit = rays_against(&prepared, &query).into_iter().next().expect("should hit");
        assert_eq!(hit.guid, 7);
        assert!(approx(hit.distance, 4.0));
        assert!(approx(hit.normal.z, 1.0));
        assert!(approx(hit.point.z, -4.0));
    }

    #[test]
    fn ray_respects_max_distance_and_ignore_list() {
        let snapshots = vec![snapshot(7, Vector3::new(0.0, 0.0, -5.0), ColliderShapeDef::Sphere { diameter: 2.0 })];
        let prepared = prepare_colliders(&snapshots);
        let query = RayQuery::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, -1.0)).max_distance(2.0);
        assert!(rays_against(&prepared, &query).is_empty());

        let query = RayQuery::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, -1.0)).ignore(7);
        assert!(rays_against(&prepared, &query).is_empty());
    }

    #[test]
    fn ray_sensor_inclusion() {
        let snapshots = vec![snapshot_with(
            7,
            Vector3::new(0.0, 0.0, -5.0),
            ColliderShapeDef::Box { size: Vector3::new(1.0, 1.0, 1.0) },
            true,
            0,
            0,
        )];
        let prepared = prepare_colliders(&snapshots);
        let query = RayQuery::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, -1.0));
        assert!(rays_against(&prepared, &query).is_empty());

        let query = RayQuery::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, -1.0)).include_sensors(true);
        let hits = rays_against(&prepared, &query);
        assert_eq!(hits.len(), 1);
        assert!(hits[0].hit_sensor);
    }

    #[test]
    fn ray_hits_capsule_side_and_box() {
        let snapshots = vec![
            snapshot(1, Vector3::new(0.0, 0.0, -5.0), ColliderShapeDef::Capsule { radius: 0.5, height: 2.0 }),
            snapshot(2, Vector3::new(3.0, 0.0, -5.0), ColliderShapeDef::Box { size: Vector3::new(1.0, 1.0, 1.0) }),
        ];
        let prepared = prepare_colliders(&snapshots);
        let query = RayQuery::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, -1.0));
        let hit = rays_against(&prepared, &query).into_iter().next().expect("should hit capsule");
        assert_eq!(hit.guid, 1);
        assert!(approx(hit.distance, 4.5));

        let query = RayQuery::new(Vector3::new(3.0, 0.0, 0.0), Vector3::new(0.0, 0.0, -1.0));
        let hit = rays_against(&prepared, &query).into_iter().next().expect("should hit box");
        assert_eq!(hit.guid, 2);
        assert!(approx(hit.distance, 4.5));
    }

    #[test]
    fn degenerate_colliders_are_dropped() {
        let mut snapshot = snapshot(1, Vector3::new(0.0, 0.0, 0.0), ColliderShapeDef::Box { size: Vector3::new(1.0, 1.0, 1.0) });
        snapshot.matrix = Matrix4x4::new(Vector3::zero(), Quaternion::identity(), Vector3::zero());
        assert!(prepare_colliders(&[snapshot]).is_empty());
    }
}
