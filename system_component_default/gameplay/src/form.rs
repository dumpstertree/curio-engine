use crate::{
    built_in::facet::transform::{transform2d::Transform2D, transform3d::Transform3D},
    form_ref::{FormRef, MutQuery},
    static_data::global_components::COMPONENT_REGISTRY,
    traits::facet_common::FacetCommon,
};
use curio_core::{FieldState, ObjectState, Quaternion, Vector2, Vector3};
use hecs::{Entity, Query};
use num::NumCast;
use std::{cell::RefCell, hash::Hash, rc::Rc};

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
    /*
    c.spawn( "my_name" )
        .position( 0, 0, 0 )
        .rotation( 0, 0, 0 )
        .scale( 0, 0, 0 )
        .facet( RendererStatic::default()
            .set_opacity( 1.0)
            .set_asset( my_asset )
        )
        .child( c.spawn( "my_name" )
            .global_position( 0, 0, 0 )
            .facet( RendererStatic::default()
                .set_opacity( 1.0)
                .set_asset( my_asset )
            )
        )
        .child( c.spawn( "my_name" )
            .global_position_vec( Vector3::new( 0, 0, 0) )
            .facet( RendererStatic::default()
                .set_opacity( 1.0)
                .set_asset( my_asset )
            )
        )
        .child( c.spawn( "my_name" )
            .global_position( Vector3::new( 0, 0, 0) )
            .facet( RendererStatic::default()
                .set_opacity( 1.0)
                .set_asset( my_asset )
            )
        );
    */

    // set local values

    /// Sets local position of required Transform3D.
    pub fn position<X, Y, Z>(self, x: X, y: Y, z: Z) -> Self
    where
        X: NumCast,
        Y: NumCast,
        Z: NumCast,
    {
        self.edit_facet::<Transform3D>(|t| t.position = Vector3::new(NumCast::from(x).unwrap(), NumCast::from(y).unwrap(), NumCast::from(z).unwrap()));
        self
    }

    /// Sets local rotation of required Transform3D. Input is expected to be euler angles
    pub fn rotation<X, Y, Z>(self, x: X, y: Y, z: Z) -> Self
    where
        X: NumCast,
        Y: NumCast,
        Z: NumCast,
    {
        self.edit_facet::<Transform3D>(|t| t.rotation = Quaternion::from_euler(Vector3::new(NumCast::from(x).unwrap(), NumCast::from(y).unwrap(), NumCast::from(z).unwrap())));
        self
    }
    /// Sets the local scale of the required Transforms3D.
    pub fn scale<X, Y, Z>(self, x: X, y: Y, z: Z) -> Self
    where
        X: NumCast,
        Y: NumCast,
        Z: NumCast,
    {
        self.edit_facet::<Transform3D>(|t| t.scale = Vector3::new(NumCast::from(x).unwrap(), NumCast::from(y).unwrap(), NumCast::from(z).unwrap()));
        self
    }

    /// Not Yet Implemented. Sets the world postion of the required Transforms3D.
    pub fn position_world<X, Y, Z>(self, x: X, y: Y, z: Z) -> Self
    where
        X: NumCast,
        Y: NumCast,
        Z: NumCast,
    {
        todo!()
    }
    /// Not Yet Implemented. Sets the world rotation of the required Transforms3D. Input is expected to be euler angles
    pub fn rotation_world<X, Y, Z>(self, x: X, y: Y, z: Z) -> Self
    where
        X: NumCast,
        Y: NumCast,
        Z: NumCast,
    {
        todo!()
    }

    /// Sets local position of required Transform3D using a pre-built Vector3
    pub fn position_vec(self, pos: Vector3) -> Self {
        self.edit_facet::<Transform3D>(|t| t.position = pos);
        self
    }
    /// Sets local rotation of required Transform3D using a pre-built Quaternion
    pub fn rotation_qat(self, rot: Quaternion) -> Self {
        self.edit_facet::<Transform3D>(|t| t.rotation = rot);
        self
    }
    /// Sets local scale of required Transform3D using a pre-built Vector3
    pub fn scale_vec(self, scl: Vector3) -> Self {
        self.edit_facet::<Transform3D>(|t| t.scale = scl);
        self
    }

    // set world values with a vec
    pub fn position_world_vec(&self) {}
    pub fn rotation_world_qat(&self) {}
    pub fn scale_world_vec(&self) {}

    // add a facet
    pub fn facet<T: FacetCommon>(self, facet: T) -> Self {
        self.add_facet(facet)
    }
    pub fn facet_default<T: FacetCommon + Default>(self) -> Self {
        // self.add_facet_default<T>()
        self
    }

    // add a child
    pub fn child(self, child: Form) -> Self {
        child.set_parent(Some(self.clone()));
        self
    }

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
    /// Get the instance ID of this Form
    pub fn instance_id(&self) -> i32 {
        self.form_ref.borrow().instance_id()
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
    /// Add a Facet 'T' using its default value.
    pub fn add_facet_default<T: FacetCommon + Default>(self) -> Self {
        FormRef::add_facet_default::<T>(&self);
        self
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
    /// Get the HECS Entity. This wille eventually be made deprecated
    pub fn entity(&self) -> Entity {
        self.form_ref.borrow().entity()
    }
}
