use crate::ComponentState;

#[unsafe(no_mangle)]
pub extern "C" fn peek_curio() -> Box<Vec<ComponentState>> {
    Box::new(vec![ComponentState::default(), ComponentState::default(), ComponentState::default()])
}
