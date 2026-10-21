use curio_core::{FieldState, Matrix4x4, Vector3};
use facet::facet;
use gameplay::traits::field_override::FieldOverride;

/// A collider attached to a form. The collector habit in this crate copies every
/// collider (with its world transform) into `SysRecordCollider` each tick and the
/// physics plugin reads that record to detect collisions and answer raycasts.
///
/// Shapes are defined at unit space around the form's origin and are scaled by the
/// form's world scale:
/// - `Box`: `size` is the full edge length (a box spans +/- size/2)
/// - `Sphere`: `diameter` is the full width (radius = diameter / 2)
/// - `Capsule`: `height` is the full height including the rounded caps, oriented
///   along the local +Y axis
#[facet]
pub struct Collider {
    pub shape: ColliderShapeDef,
    /// Sensors overlap other colliders (and raise events) but produce no contact data.
    pub sensor: bool,
    /// The group bits this collider belongs to. Default `0` = the only group.
    pub filter_group: u32,
    /// Which group bits this collider interacts with. `0` (the default) means
    /// "interact with everything".
    pub filter_mask: u32,
}
impl Collider {
    pub fn builder() -> ColliderBuilder {
        ColliderBuilder {
            shape: ColliderShapeDef::default(),
            sensor: false,
            filter_group: 0,
            filter_mask: 0,
        }
    }

    /// Does this collider's filter allow it to interact with `other`?
    pub fn filter_allows(&self, other: &Collider) -> bool {
        filters_match(self.filter_group, self.filter_mask, other.filter_group, other.filter_mask)
    }
}

/// Two colliders interact iff each one's mask allows the other's group.
/// A mask of `0` acts as "all groups".
pub fn filters_match(group_a: u32, mask_a: u32, group_b: u32, mask_b: u32) -> bool {
    let a_ok = mask_a == 0 || (mask_a & group_b) != 0;
    let b_ok = mask_b == 0 || (mask_b & group_a) != 0;
    a_ok && b_ok
}
impl FieldOverride for Collider {
    fn apply(&mut self, field: &str, value: &str) {
        let value = value.trim();
        match field {
            "shape" => {
                if let Some(shape) = ColliderShapeDef::decode(value) {
                    self.shape = shape;
                }
            }
            "sensor" => self.sensor = value.parse().unwrap_or(self.sensor),
            "group" => self.filter_group = value.parse().unwrap_or(self.filter_group),
            "mask" => self.filter_mask = value.parse().unwrap_or(self.filter_mask),
            _ => {}
        }
    }
    fn get_state(&self) -> Vec<FieldState> {
        vec![
            FieldState::new("shape", self.shape.encode()),
            FieldState::new("sensor", self.sensor),
            FieldState::new("group", self.filter_group),
            FieldState::new("mask", self.filter_mask),
        ]
    }
}

pub struct ColliderBuilder {
    shape: ColliderShapeDef,
    sensor: bool,
    filter_group: u32,
    filter_mask: u32,
}
impl ColliderBuilder {
    pub fn shape(mut self, shape: ColliderShapeDef) -> Self {
        self.shape = shape;
        self
    }
    pub fn box_shape(mut self, size: Vector3) -> Self {
        self.shape = ColliderShapeDef::Box { size };
        self
    }
    pub fn sphere_shape(mut self, diameter: f32) -> Self {
        self.shape = ColliderShapeDef::Sphere { diameter };
        self
    }
    pub fn capsule_shape(mut self, radius: f32, height: f32) -> Self {
        self.shape = ColliderShapeDef::Capsule { radius, height };
        self
    }
    pub fn sensor(mut self, sensor: bool) -> Self {
        self.sensor = sensor;
        self
    }
    pub fn filter_group(mut self, filter_group: u32) -> Self {
        self.filter_group = filter_group;
        self
    }
    pub fn filter_mask(mut self, filter_mask: u32) -> Self {
        self.filter_mask = filter_mask;
        self
    }
    pub fn build(self) -> Collider {
        Collider {
            shape: self.shape,
            sensor: self.sensor,
            filter_group: self.filter_group,
            filter_mask: self.filter_mask,
            owner: None,
        }
    }
}

/// The geometric description of a collider shape.
#[derive(Clone, Debug, PartialEq)]
pub enum ColliderShapeDef {
    Box { size: Vector3 },
    Sphere { diameter: f32 },
    Capsule { radius: f32, height: f32 },
}
impl Default for ColliderShapeDef {
    fn default() -> Self {
        ColliderShapeDef::Box { size: Vector3::one() }
    }
}
impl ColliderShapeDef {
    pub fn encode(&self) -> String {
        match self {
            ColliderShapeDef::Box { size } => format!("box {}", size),
            ColliderShapeDef::Sphere { diameter } => format!("sphere {}", diameter),
            ColliderShapeDef::Capsule { radius, height } => {
                format!("capsule {} {}", radius, height)
            }
        }
    }
    pub fn decode(value: &str) -> Option<ColliderShapeDef> {
        let mut words = value.split_whitespace();
        match words.next()?.to_ascii_lowercase().as_str() {
            "box" => {
                let size = words.next()?.parse().ok()?;
                Some(ColliderShapeDef::Box { size })
            }
            "sphere" => {
                let diameter = words.next()?.parse().ok()?;
                Some(ColliderShapeDef::Sphere { diameter })
            }
            "capsule" => {
                let radius = words.next()?.parse().ok()?;
                let height = words.next()?.parse().ok()?;
                Some(ColliderShapeDef::Capsule { radius, height })
            }
            _ => None,
        }
    }
}

/// A collider as it existed at one point in time: the shape plus the world
/// matrix it was last seen at. Written to the ledger by the collector habit and
/// consumed by the physics plugin. `guid` is the owning form's `instance_id()`.
#[derive(Clone)]
pub struct ColliderSnapshot {
    pub guid: i32,
    pub matrix: Matrix4x4,
    pub shape: ColliderShapeDef,
    pub sensor: bool,
    pub filter_group: u32,
    pub filter_mask: u32,
}
impl ColliderSnapshot {
    /// Does this snapshot's filter allow it to interact with `other`?
    pub fn filter_allows(&self, other: &ColliderSnapshot) -> bool {
        filters_match(
            self.filter_group,
            self.filter_mask,
            other.filter_group,
            other.filter_mask,
        )
    }
}

/// A single contact point between two solid colliders.
#[derive(Clone, Copy, Debug)]
pub struct Contact {
    pub point: Vector3,
    /// Surface normal of collider_a at the contact point (points away from a's interior).
    pub normal_a: Vector3,
    /// Surface normal of collider_b at the contact point (points away from b's interior).
    pub normal_b: Vector3,
}

/// Two colliders that currently overlap. `contact` is `None` when either
/// collider is a sensor.
#[derive(Clone)]
pub struct CollisionPair {
    pub collider_a: ColliderSnapshot,
    pub collider_b: ColliderSnapshot,
    pub contact: Option<Contact>,
}
impl CollisionPair {
    pub fn involves(&self, guid: i32) -> bool {
        self.collider_a.guid == guid || self.collider_b.guid == guid
    }
    /// Given one of the two guids in the pair, return the other one.
    pub fn other_guid(&self, guid: i32) -> Option<i32> {
        if self.collider_a.guid == guid {
            Some(self.collider_b.guid)
        } else if self.collider_b.guid == guid {
            Some(self.collider_a.guid)
        } else {
            None
        }
    }
}
