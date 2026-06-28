use crate::built_in::facet::transform::transform3d::Transform3D;
use crate::form::{Form, FormBuilder3D};
use crate::form_ref::FormRef;
use crate::traits_internal::world_context_common::ContextCommon;
use curio_core::{Composition, Quaternion, Vector3};
use hecs::World;
use std::cell::RefCell;
use std::rc::Rc;

/// 3D access to the world
pub struct Context3D {
    pub world: Rc<RefCell<World>>,
}

impl ContextCommon for Context3D {
    fn hecs_world(&self) -> Rc<RefCell<World>> {
        self.world.clone()
    }
}
impl Context3D {
    /// Create a new context
    pub fn new(world: Rc<RefCell<World>>) -> Self {
        Self { world }
    }

    /// Spawn a Form inside the Context
    pub fn spawner(&mut self, name: &str) -> FormBuilder3D {
        FormBuilder3D {
            comp: None,
            name: name.to_owned(),
            pos: Vector3::zero(),
            rot: Quaternion::identity(),
            scl: Vector3::one(),
            enabled: true,
            children: Vec::new(),
            facets: Vec::new(),
            world: self.world.clone(),
        }
    }
}
