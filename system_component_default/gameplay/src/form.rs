use crate::{
    built_in::facet::transform::transform3d::Transform3D,
    form_ref::{FormRef, MutQuery},
    static_data::global_components::COMPONENT_REGISTRY,
    traits::{facet_common::FacetCommon, field_override::FieldOverride},
    traits_internal::world_context_common::{self, ContextCommon, spawn_prefab_recursive_internal},
};
use curio_core::{Composition, ObjectState, Quaternion, Vector3};
use hecs::{Entity, Query, World};
use num::NumCast;
use std::{cell::RefCell, hash::Hash, rc::Rc, sync::Arc};

/// Representation of an object in the world
#[derive(Clone, PartialEq, Eq)]
pub struct Form {
    form_ref: Rc<RefCell<FormRef>>,
}
// Constructors
impl Form {
    /// Create a new form. This should only be called by a context
    pub fn new(form_ref: Rc<RefCell<FormRef>>) -> Form {
        Form { form_ref }
    }
}
unsafe impl Send for Form {}
unsafe impl Sync for Form {}

// Public Methods
impl Form {
    /// Get the serialized state
    pub fn get_state(&self) -> ObjectState {
        let x = COMPONENT_REGISTRY.read().expect("msg");

        let mut data = Vec::new();
        for z in &x.get_state {
            let d = z.1(&self);
            if let Some(dd) = d {
                data.push(dd);
            }
        }

        ObjectState {
            object_name: self.name(),
            children: self.children().iter().map(|x| x.get_state()).collect(),
            components: data,
        }
    }
    pub fn set_enabled(&mut self, enabled: bool) {
        self.form_ref.borrow_mut().set_enabled(enabled);
    }
    pub fn enabled(&self) -> bool {
        self.form_ref.borrow().enabled()
    }
    pub fn enabled_in_hierachy(&self) -> bool {
        self.form_ref.borrow().enabled()
    }
    /// Get the instance ID of this Form
    pub fn instance_id(&self) -> i32 {
        self.form_ref.borrow().id()
    }
    /// Get the backing FormRef for this form
    pub fn form_ref(&self) -> Rc<RefCell<FormRef>> {
        self.form_ref.clone()
    }
    /// Get the name of the Form
    pub fn name(&self) -> String {
        self.form_ref.borrow().name().to_string()
    }
    /// Get the children Forms
    pub fn children(&self) -> Vec<Form> {
        self.form_ref.borrow().children()
    }
    /// Get the parent Form if it exists
    pub fn parent(&self) -> Option<Form> {
        let b = self.form_ref.borrow();
        let p = b.parent();
        drop(b);
        p
    }
    pub fn get_child(&self, path: &str) -> Option<Form> {
        let split = path.split("/");
        let mut cur_form = self.clone();
        for s in split {
            let mut children = cur_form.children();
            children.retain(|x| x.name() == s);
            if children.len() == 0 {
                panic!("no matches");
            } else if children.len() > 1 {
                panic!("too many matches");
            } else {
                cur_form = children.first().unwrap().clone();
            }
        }
        Some(cur_form)
    }
    /// Add a Facet 'T' using an instance.
    pub fn add_facet<T: FacetCommon>(self, value: T) -> Self {
        FormRef::add_facet(&self, value);
        self
    }
    pub fn try_edit_facets_in_child<T>(&self, path: &str, f: impl for<'a> FnOnce(<T::Query<'a> as Query>::Item<'a>))
    where
        T: MutQuery,
    {
        if let Some(c) = self.get_child(path) {
            c.form_ref.borrow_mut().try_edit_facet_group::<T>(f);
        } else {
            println!("failed");
        }
    }
    pub fn try_edit_facets<T>(&self, f: impl for<'a> FnOnce(<T::Query<'a> as Query>::Item<'a>))
    where
        T: MutQuery,
    {
        self.form_ref.borrow_mut().try_edit_facet_group::<T>(f);
    }
    pub fn edit_facets<T>(&self, f: impl for<'a> FnOnce(<T::Query<'a> as Query>::Item<'a>))
    where
        T: MutQuery,
    {
        self.form_ref.borrow_mut().edit_facet_group::<T>(f);
    }
    pub fn try_edit_facet_in_child<T: FacetCommon + 'static>(&self, path: &str, edit_fn: impl FnOnce(&mut T)) {
        if let Some(c) = self.get_child(path) {
            c.form_ref.borrow_mut().try_edit_facet::<T>(edit_fn);
        } else {
            println!("failed");
        }
    }
    /// Edit facet of type 'T'
    pub fn edit_facet<T: FacetCommon + 'static>(&self, edit_fn: impl FnOnce(&mut T)) {
        self.form_ref.borrow_mut().edit_facet(edit_fn);
    }
    /// Edit facet of type 'T'
    pub fn try_edit_facet<T: FacetCommon + 'static>(&self, edit_fn: impl FnOnce(&mut T)) {
        self.form_ref.borrow_mut().try_edit_facet(edit_fn);
    }
    /// Get facet of type 'T'
    pub fn get_facet<T: FacetCommon + Clone + 'static>(&self) -> Option<T> {
        self.form_ref.borrow().get_facet::<T>()
    }
    /// Has facet of type 'T'
    pub fn has_facet<T: FacetCommon + 'static>(&self) -> bool {
        self.form_ref.borrow().has_facet::<T>()
    }
    /// Set the parent
    pub fn set_parent(&self, parent_form: Option<Form>) {
        FormRef::set_parent(self.clone(), parent_form);
    }
    /// Destroy this Form
    pub fn destroy(&self) {
        self.form_ref.borrow().destroy();
    }
}
// Hash
impl Hash for Form {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.form_ref.borrow().hash(state);
    }
}
// To Deprecate
impl Form {
    /// Get the HECS Entity. This will eventually be made deprecated
    pub fn entity(&self) -> Entity {
        self.form_ref.borrow().entity()
    }
}

type AddFacetFn = Box<dyn FnOnce(&Form)>;
pub struct FormBuilder3D {
    pub(crate) comp: Option<Arc<Composition>>,
    pub(crate) name: String,
    pub(crate) pos: Vector3,
    pub(crate) rot: Quaternion,
    pub(crate) scl: Vector3,
    pub(crate) enabled: bool,
    pub(crate) children: Vec<Form>,
    pub(crate) facets: Vec<AddFacetFn>,
    pub(crate) world: Rc<RefCell<World>>,
}
impl FormBuilder3D {
    // Add a Facet to the spawned Form
    pub fn facet<T: FacetCommon + FieldOverride>(mut self, value: T) -> Self {
        self.facets.push(Box::new(|x| {
            if x.has_facet::<T>() {
                x.edit_facet::<T>(|t| {
                    for s in value.get_state() {
                        t.apply(&s.field_name, &s.data.to_string());
                    }
                });
            } else {
                FormRef::add_facet(x, value);
            }
        }));
        self
    }

    /// Spawn an entire Composition as the base Form
    pub fn composition(mut self, composition: Option<Arc<Composition>>) -> Self {
        self.comp = composition;
        self
    }

    /// Sets local position of required Transform3D.
    pub fn position<X, Y, Z>(mut self, x: X, y: Y, z: Z) -> Self
    where
        X: NumCast,
        Y: NumCast,
        Z: NumCast,
    {
        self.pos = Vector3::new(NumCast::from(x).unwrap(), NumCast::from(y).unwrap(), NumCast::from(z).unwrap());
        self
    }

    /// Sets local rotation of required Transform3D. Input is expected to be euler angles
    pub fn rotation<X, Y, Z>(mut self, x: X, y: Y, z: Z) -> Self
    where
        X: NumCast,
        Y: NumCast,
        Z: NumCast,
    {
        self.rot = Quaternion::from_euler(Vector3::new(NumCast::from(x).unwrap(), NumCast::from(y).unwrap(), NumCast::from(z).unwrap()));
        self
    }
    /// Sets the local scale of the required Transforms3D.
    pub fn scale<X, Y, Z>(mut self, x: X, y: Y, z: Z) -> Self
    where
        X: NumCast,
        Y: NumCast,
        Z: NumCast,
    {
        self.scl = Vector3::new(NumCast::from(x).unwrap(), NumCast::from(y).unwrap(), NumCast::from(z).unwrap());
        self
    }

    /// Not Yet Implemented. Sets the world postion of the required Transforms3D.
    pub fn global_position<X, Y, Z>(self, _x: X, _y: Y, _z: Z) -> Self
    where
        X: NumCast,
        Y: NumCast,
        Z: NumCast,
    {
        todo!()
    }
    /// Not Yet Implemented. Sets the world rotation of the required Transforms3D. Input is expected to be euler angles
    pub fn global_rotation<X, Y, Z>(self, _x: X, _y: Y, _z: Z) -> Self
    where
        X: NumCast,
        Y: NumCast,
        Z: NumCast,
    {
        todo!()
    }
    pub fn global_scale<X, Y, Z>(self, _x: X, _y: Y, _z: Z) -> Self
    where
        X: NumCast,
        Y: NumCast,
        Z: NumCast,
    {
        todo!()
    }

    // hierachy
    pub fn child(mut self, form: Form) -> Self {
        self.children.push(form);
        self
    }

    // enable
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    // build
    pub fn spawn(self) -> Form {
        // create the form depending on the composition state
        let form = {
            // spawn using the passed in composition
            if let Some(comp) = self.comp {
                spawn_prefab_recursive_internal(self.world, &comp, self.name)
            }
            // spawn using a newly creaty form
            else {
                // generate an entity
                let entity = {
                    // borrow
                    let mut world = self.world.borrow_mut();
                    // spawn - dont know how to spawn with only a single tranform
                    world.spawn(())
                };

                // spawn the form
                FormRef::new(&self.name, self.world, entity)
            }
        };

        // if we dont already have a transform lets add one
        if !form.has_facet::<Transform3D>() {
            // generate transform
            let mut transform = Transform3D::default();
            transform.position = self.pos;
            transform.rotation = self.rot;
            transform.scale = self.scl;

            // add the tranform - must be added like this in order to properly set the owner
            FormRef::add_facet(&form, transform);
        } else {
            // edit the existing transform
            form.edit_facet::<Transform3D>(|transform| {
                transform.position = self.pos;
                transform.rotation = self.rot;
                transform.scale = self.scl;
            });
        }

        // add all facets - this does not check if it already exists. It probably should
        for add_facet in self.facets {
            add_facet(&form);
        }

        // return our new form
        form
    }
}
