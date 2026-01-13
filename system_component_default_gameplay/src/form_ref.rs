use core::random::Random;
use hecs::{Entity, World};
use std::{any::type_name, cell::RefCell, hash::Hash, rc::Rc};

use crate::form::{FacetCommon, Form};

/// Representation of an object in the world
pub struct FormRef {
    world: Rc<RefCell<World>>,
    entity: Entity,
    name: String,
    children: Vec<Form>,
    parent: Vec<Form>,
    instance_id: i32,
}
impl FormRef {
    /// Createa a new form. This should only be called by a context
    pub fn new(name: &str, world: Rc<RefCell<World>>, entity: Entity) -> Form {
        Form::new(Rc::new(RefCell::new(FormRef {
            world,
            entity,
            name: name.to_string(),
            children: vec![],
            parent: vec![],
            instance_id: Random::range_int(-99999, 9999),
        })))
    }
    pub fn instance_id(&self) -> i32 {
        self.instance_id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn children(&self) -> Vec<Form> {
        self.children.clone()
    }
    pub fn parent(&self) -> Option<Form> {
        self.parent.get(0).cloned()
    }
    /// Get the HECS entity. This will eventually be made deprecated
    pub fn entity(&self) -> Entity {
        self.entity
    }
    /// Set the parent - reparenting is currently shaky at best
    pub fn set_parent(child_form: Form, parent_form: Option<Form>) {
        // if we already have a parent remove from old parent's children
        if let Some(old_parent) = child_form.parent() {
            // get the old parent ref
            let old_parent_ref = old_parent.form_ref();
            // borrow mut
            let mut borrow = old_parent_ref.borrow_mut();
            // remove this as child
            borrow
                .children
                .retain(|child| child.instance_id() != child_form.instance_id());
        }
        // if we were given a new parent set it
        if let Some(parent_form) = parent_form {
            // add as child to parent
            parent_form
                .form_ref()
                .borrow_mut()
                .children
                .push(child_form.clone());
            // add as parent to child
            child_form.form_ref().borrow_mut().parent = vec![parent_form];
        } else {
            // add as parents as empty
            child_form.form_ref().borrow_mut().parent = vec![];
        }
    }

    /// Add a component T using its default value.
    pub fn add_facet_default<T>(form: &Form)
    where
        T: FacetCommon + Default,
    {
        Self::add_facet(form, T::default());
    }

    /// Add a specific component instance.
    pub fn add_facet<T>(form: &Form, value: T)
    where
        T: FacetCommon,
    {
        let mut value = value;
        value.set_ownership(form.clone());
        let form_ref = form.form_ref();
        let form_ref_borrow = form_ref.borrow();
        form_ref_borrow
            .world
            .borrow_mut()
            .insert_one(form_ref_borrow.entity, value)
            .expect("Failed to insert component");
    }

    /// Modify a component in-place.
    pub fn edit_facet<T: FacetCommon + 'static>(&self, edit_fn: impl FnOnce(&mut T)) {
        let world = self.world.borrow_mut();
        let mut borrow = world
            .get::<&mut T>(self.entity)
            .unwrap_or_else(|_| panic!("Form '{}' does not contain Facet {}", self.name, type_name::<T>(),));
        edit_fn(&mut *borrow);
    }

    /// Get a cloned component value (Unity style).
    pub fn get_facet<T: FacetCommon + Clone + 'static>(&self) -> Option<T> {
        let cloned = {
            let world = self.world.borrow();
            let borrow = world.get::<&T>(self.entity).ok()?;
            let cloned = (*borrow).clone();
            drop(borrow);
            cloned
        }; // borrow ends here

        Some(cloned)
    }

    /// Returns true if the object contains component T.
    pub fn has_facet<T: FacetCommon + 'static>(&self) -> bool {
        self.world.borrow().get::<&T>(self.entity).is_ok()
    }

    /// Destroy this form and remove it from the world
    pub fn destroy(&self) {
        let _ = self.world.borrow_mut().despawn(self.entity);
    }
}
impl Eq for FormRef {}
impl PartialEq for FormRef {
    fn eq(&self, other: &Self) -> bool {
        // currently doesnt check world
        self.entity == other.entity && self.name == other.name
    }
}
impl Hash for FormRef {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.entity.hash(state);
        self.name.hash(state);
    }
}
