use crate::form_ref::FormRef;
use hecs::{Component, Entity};
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
// Public Methods
impl Form {
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
    /// Edit facet of type 'T'
    pub fn edit_facet<T: FacetCommon + 'static>(&self, edit_fn: impl FnOnce(&mut T)) {
        self.form_ref.borrow_mut().edit_facet(edit_fn);
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
    pub fn set_parent(&mut self, parent_form: Option<Form>) {
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
pub trait FacetCommon: Component {
    fn set_ownership(&mut self, owner: Form);
    fn form(&self) -> Form;
}
