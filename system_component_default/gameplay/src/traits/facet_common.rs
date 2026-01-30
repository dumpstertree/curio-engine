use crate::form::Form;
use hecs::Component;

pub trait FacetCommon: Component {
    fn set_ownership(&mut self, owner: Form);
    fn form(&self) -> Form;
}
